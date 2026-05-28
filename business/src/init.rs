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

    // ───── Recipes Service ─────
    let pg_local_repo = Arc::new(recipe::PostgresAdapter::new(pool));
    let mealdb_external_repo = Arc::new(recipe::MealdbAdapter::new(
        config.external_api.url.clone(), config.external_api.key.clone(),
    ));
    let redis_cache_repo = Arc::new(recipe::RedisCacheAdapter::new(redis_conn));

    let recipe_service = recipe::Service::new(
        pg_local_repo, mealdb_external_repo, redis_cache_repo,
    );

    Ok(BusinessState {
        base: Arc::new(base_service),
        recipe: Arc::new(recipe_service),
    })
}