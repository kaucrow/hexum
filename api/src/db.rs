use std::time::Duration;

use sqlx::postgres::{PgPool, PgPoolOptions};
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