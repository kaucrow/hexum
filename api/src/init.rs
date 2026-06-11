use std::time::Duration;

use sqlx::postgres::{PgPool, PgPoolOptions};
use redis::aio::ConnectionManager as RedisConnManager;
use anyhow::{Result, Context};
use platform::Config;

pub async fn init_postgres_pool(config: &Config) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.postgres.pool_max_conn)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&config.postgres.url())
        .await
        .context("Failed to connect to PostgreSQL database.")?;

    // Run crate migrations sequentially
    platform::postgres::run_migrations(&pool).await?;
    business::postgres::run_migrations(&pool).await?;

    Ok(pool)
}

pub async fn init_redis_conn(config: &Config) -> Result<RedisConnManager> {
    let client = redis::Client::open(config.redis.url())?;

    let conn = RedisConnManager::new(client)
        .await
        .context("Failed to connect to Redis database.")?;

    Ok(conn)
}