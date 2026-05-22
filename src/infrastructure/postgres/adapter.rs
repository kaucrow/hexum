use sqlx::{
    PgPool,
    postgres::PgPoolOptions,
};
use anyhow::{Result, Context};

use crate::prelude::*;
use crate::Config;
use super::queries;

#[derive(Clone)]
pub struct PostgresAdapter {
    pub pool: PgPool,
}

impl PostgresAdapter {
    pub async fn new(config: &Config) -> Result<Self> {
        queries::init()?;

        let pool = PgPoolOptions::new()
            .max_connections(config.postgres.pool_max_conn)
            .connect(&config.postgres.url())
            .await
            .context("Failed to connect to PostgreSQL database.")?;

        info!("Postgres migrations ran successfully.");

        sqlx::migrate!("postgres/migrations")
            .run(&pool)
            .await?;

        Ok(Self { pool })
    }
}