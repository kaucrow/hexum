use chrono::{Utc, NaiveTime, Duration};
use tokio::time::sleep;

use crate::{
    BusinessState,
    get_config,
    prelude::*,
    features::*,
};

pub async fn init(
    pool: sqlx::PgPool,
    redis_conn: redis::aio::ConnectionManager,
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
    let igdb_port = videogame_api::IgdbAdapter::new(http_client.clone(), auth_data);

    let internal_search_adapter = Arc::new(search::PostgresAdapter::new(pool.clone()));
    let external_search_adapter = Arc::new(search::IgdbAdapter::new(igdb_port.clone()));

    let search_service = Arc::new(search::Service::new(
        internal_search_adapter,
        external_search_adapter,
        pagination_cache_repo,
    ));

    // ─── Platform ─────────────────────────────────────────────────────
    let internal_platform_adapter = Arc::new(platform::PostgresAdapter::new(pool));
    let external_platform_adapter = Arc::new(platform::IgdbAdapter::new(igdb_port));

    let platform_service = Arc::new(platform::Service::new(
        internal_platform_adapter,
        external_platform_adapter,
    ));

    start_cron_db_sync(platform_service.clone());

    Ok(BusinessState {
        config,
        base: base_service,
        search: search_service,
        platform: platform_service,
    })
}


pub fn start_cron_db_sync(
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