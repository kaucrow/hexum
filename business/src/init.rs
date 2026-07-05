use chrono::{Utc, NaiveTime, Duration};
use tokio::time::sleep;

use ::platform as platform_crate;
use platform_crate::features::{
    user::{User, UserAuthenticator, Password},
    security::PasetoAdapter,
};

use crate::{
    BusinessState,
    get_config,
    postgres,
    prelude::*,
    features::*,
};

pub async fn init(
    pool: sqlx::PgPool,
    redis_conn: redis::aio::ConnectionManager,
    auth_service: Arc<dyn platform_crate::features::auth::UseCase>,
) -> Result<BusinessState, anyhow::Error> {
    let config = Arc::new(get_config()?);

    // ─── Base ─────────────────────────────────────────────────────────
    let pg_base_adapter = Arc::new(base::PostgresAdapter::new(pool.clone()));
    let base_service = Arc::new(base::Service::new(pg_base_adapter));

    let pagination_cache_repo = Arc::new(base::pagination::RedisAdapter::new(redis_conn));

    // ─── Search ───────────────────────────────────────────────────────
    let http_client = reqwest::Client::new();

    // Videogame API adapter. Holds auth credentials + access token,
    // provides `request<T>()` with 401 -> re-auth -> retry logic.
    let auth_data = videogame_api::VideogameApiAuthData {
        url: config.videogame_api.auth.url.clone(),
        client_id: config.videogame_api.auth.client_id.clone(),
        client_secret: config.videogame_api.auth.client_secret.clone(),
    };

    let igdb_client = Arc::new(videogame_api::IgdbClient::new(http_client.clone(), auth_data));

    let search_internal_repo = Arc::new(search::PostgresAdapter::new(pool.clone()));

    let search_service = Arc::new(search::Service::new(
        search_internal_repo,
        pagination_cache_repo,
        igdb_client.clone(),
    ));

    // ─── Platform ─────────────────────────────────────────────────────
    let internal_platform_adapter = Arc::new(platform::PostgresAdapter::new(pool.clone()));
    let external_platform_adapter = Arc::new(platform::IgdbAdapter::new(igdb_client.clone()));

    let platform_service = Arc::new(platform::Service::new(
        internal_platform_adapter,
        external_platform_adapter,
    ));

    // ─── Game ─────────────────────────────────────────────────────────
    let internal_game_adapter = Arc::new(game::PostgresAdapter::new(pool.clone()));
    let external_game_adapter = Arc::new(game::IgdbAdapter::new(igdb_client));

    let game_service = Arc::new(game::Service::new(
        internal_game_adapter,
        external_game_adapter,
    ));

    // ─── Critic ───────────────────────────────────────────────────────
    let pg_critic_adapter = Arc::new(critic::PostgresAdapter::new(pool.clone()));
    let critic_service = Arc::new(critic::Service::new(pg_critic_adapter));

    // ─── Review ───────────────────────────────────────────────────────
    let pg_review_adapter = Arc::new(review::PostgresAdapter::new(pool.clone()));
    let review_service = Arc::new(review::Service::new(pg_review_adapter));

    // ─── Admin Seed ───────────────────────────────────────────────────
    seed_admin_user(&pool).await?;

    // ─── Cron Jobs ────────────────────────────────────────────────────

    start_cron_db_sync(platform_service.clone());

    Ok(BusinessState {
        config,
        base: base_service,
        search: search_service,
        platform: platform_service,
        game: game_service,
        auth: auth_service,
        critic: critic_service,
        review: review_service,
    })
}

/// Seeds an Admin user into the database if none exists.
///
/// Uses a Postgres transaction-level advisory lock to prevent races
/// across parallel processes.
///
/// Credentials:
/// - Username: `admin`
/// - Email: `admin@hexum.local`
/// - Password: `AdminP@ssword123!`
async fn seed_admin_user(pool: &sqlx::PgPool) -> Result<(), anyhow::Error> {
    // Open a transaction to scope the advisory lock
    let mut tx = pool
        .begin()
        .await
        .context("Failed to begin transaction for admin seed")?;

    sqlx::query("SELECT pg_advisory_xact_lock(987654321)")
        .execute(&mut *tx)
        .await
        .context("Failed to acquire advisory lock for admin seed")?;

    let has_admin: (bool,) = sqlx::query_as(
        postgres::sql(&postgres::QUERIES.user.has_any_admin),
    )
    .fetch_one(&mut *tx)
    .await
    .context("Failed to check for existing admin users")?;

    if has_admin.0 {
        info!("Admin user already exists. Skipping seed.");
        return Ok(());
    }

    info!("No admin user found. Seeding default admin user...");

    let security = PasetoAdapter::new().context("Failed to create security adapter for admin seed")?;
    use platform_crate::features::security::Port;

    let admin_user = User::new("admin", "admin@hexum.local")
        .context("Failed to create admin user")?;

    let password = Password::new("AdminP@ssword123!".to_string())
        .context("Failed to create admin password")?;

    let passwd_hash = security
        .hash_password(&password)
        .map_err(|e| anyhow::anyhow!("Failed to hash admin password: {}", e))?;

    let admin_user_id = admin_user.id;

    sqlx::query(postgres::sql(&postgres::QUERIES.user.insert))
        .bind(admin_user_id)
        .bind(admin_user.username.as_str())
        .bind(admin_user.email.as_str())
        .bind(vec!["Admin".to_string()])
        .bind(true)
        .execute(&mut *tx)
        .await
        .context("Failed to insert admin user")?;

    let authenticator = UserAuthenticator::new_local(admin_user_id, passwd_hash.clone());

    sqlx::query(postgres::sql(&postgres::QUERIES.user.insert_authenticator))
        .bind(authenticator.id)
        .bind(authenticator.user_id)
        .bind(&passwd_hash)
        .bind(true) // is_verified = true, so admin can login immediately
        .execute(&mut *tx)
        .await
        .context("Failed to insert admin authenticator")?;

    tx.commit()
        .await
        .context("Failed to commit admin seed transaction")?;

    info!("Admin user seeded successfully (admin@hexum.local).");

    Ok(())
}

fn start_cron_db_sync(
    platform_service: Arc<dyn platform::UseCase>
) {
    tokio::spawn(async move {
        info!("Automated DB sync scheduled for 03:00 AM (UTC-4) every night.");

        loop {
            // Calculate how long to wait until the next 3:00 AM (UTC-4)
            let now = Utc::now();
            let target_time = NaiveTime::from_hms_opt(7, 0, 0).expect("Invalid time configuration");

            // Generate today's target timestamp
            let mut next_run = now.date_naive().and_time(target_time).and_local_timezone(Utc).unwrap();

            // If it's already past 3:00 AM today, point to 3:00 AM tomorrow
            if now >= next_run {
                next_run = next_run + Duration ::days(1);
            }

            // Convert the delta into a Duration for Tokio
            let duration_until_target = (next_run - now).to_std().unwrap_or(std::time::Duration::from_secs(0));

            info!(
                "Next scheduled sync will execute at: {} UTC. Sleeping for {} hours and {} minutes.",
                next_run.format("%d-%m-%Y %H:%M:%S"),
                duration_until_target.as_secs() / 3600,
                (duration_until_target.as_secs() % 3600) / 60
            );

            // Sleep until the scheduled time boundary hits
            sleep(duration_until_target).await;

            // Run the execution task
            info!("It is 3:00 AM. Starting scheduled DB data ingestion...");

            info!("Syncing platforms...");
            match platform_service.sync_db_platforms().await {
                Ok(()) => info!("Scheduled platform sync completed successfully."),
                Err(e) => error!("Scheduled platform sync job failed with error: {:?}", e),
            }
        }
    });
}