use async_trait::async_trait;
use thiserror::Error;

use crate::postgres::*;
use super::*;

#[derive(Clone)]
pub struct PostgresAdapter {
    pub pool: PgPool,
}

impl PostgresAdapter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn do_ping_db(&self) -> Result<(), LocalError> {
        let _ = sqlx::query_as::<_, PingDbRow>(sql(&QUERIES.base.ping))
            .fetch_one(&self.pool)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl Port for PostgresAdapter {
    async fn ping_db(&self) -> Result<(), PortError> {
        Ok(self.do_ping_db().await?)
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

impl From<LocalError> for PortError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::Sqlx(e) => PortError::Internal(e.to_string()),
        }
    }
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
pub struct PingDbRow {
    pub value: i32,
}