use chrono::{Local, NaiveTime, Duration};
use tokio::time::sleep;
use crate::features::data_ingestion;

use platform::Config;

use crate::{
    BusinessState,
    prelude::*,
    features::*,
};

pub async fn init(
    pool: sqlx::PgPool,
    redis_conn: redis::aio::ConnectionManager,
    config: Arc<Config>,
) -> Result<BusinessState, anyhow::Error>
{
    // ───── Base Service ─────
    let pg_base_adapter = Arc::new(base::PostgresAdapter::new(pool.clone()));
    let base_service = base::Service::new(pg_base_adapter);

    // ───── Data Ingestion Service ─────
    let pg_data_ingestion_repo = Arc::new(data_ingestion::PostgresAdapter::new(pool.clone()));
    let mealdb_data_ingestion_repo = Arc::new(data_ingestion::MealdbAdapter::new(
        config.external_api.url.clone(), config.external_api.key.clone(),
    ));

    let data_ingestion_service = Arc::new(data_ingestion::Service::new(
        pg_data_ingestion_repo, mealdb_data_ingestion_repo
    ));

    // ───── Recipes Service ─────
    let pg_recipe_repo = Arc::new(recipe::PostgresAdapter::new(pool));
    let mealdb_recipe_repo = Arc::new(recipe::MealdbAdapter::new(
        config.external_api.url.clone(), config.external_api.key.clone(),
    ));
    let redis_recipe_repo = Arc::new(recipe::RedisCacheAdapter::new(redis_conn));

    let recipe_service = recipe::Service::new(
        pg_recipe_repo, mealdb_recipe_repo, redis_recipe_repo,
    );

    start_cron_recipes_sync(data_ingestion_service.clone());

    Ok(BusinessState {
        base: Arc::new(base_service),
        data_ingestion: data_ingestion_service,
        recipe: Arc::new(recipe_service),
    })
}

pub fn start_cron_recipes_sync(data_ingestion_service: Arc<dyn data_ingestion::UseCase>) {
    tokio::spawn(async move {
        info!("Automated recipe sync scheduled for 03:00 AM every night.");

        loop {
            // Calculate how long to wait until the next 3:00 AM
            let now = Local::now();
            let target_time = NaiveTime::from_hms_opt(3, 0, 0).expect("Invalid time configuration");

            // Generate today's target timestamp
            let mut next_run = now.date_naive().and_time(target_time).and_local_timezone(Local).unwrap();

            // If it's already past 3:00 AM today, point to 3:00 AM tomorrow
            if now >= next_run {
                next_run = next_run + Duration ::days(1);
            }

            // Convert the delta into a Duration for Tokio
            let duration_until_target = (next_run - now).to_std().unwrap_or(std::time::Duration::from_secs(0));

            info!(
                "Next scheduled sync will execute at: {}. Sleeping for {} hours and {} minutes.",
                next_run.format("%d-%m-%Y %H:%M:%S"),
                duration_until_target.as_secs() / 3600,
                (duration_until_target.as_secs() % 3600) / 60
            );

            // Sleep until the scheduled time boundary hits
            sleep(duration_until_target).await;

            // Run the execution task
            info!("It is 3:00 AM. Starting scheduled recipe ingestion...");
            match data_ingestion_service.sync_data().await {
                Ok(()) => info!("Scheduled recipe ingestion sync completed successfully."),
                Err(e) => error!("Scheduled recipe ingestion job failed with error: {:?}", e),
            }
        }
    });
}