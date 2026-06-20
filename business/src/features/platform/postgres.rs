use async_trait::async_trait;
use uuid::Uuid;
use thiserror::Error;
use anyhow::Result;

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
}

#[async_trait]
impl InternalRepository for PostgresAdapter {
    async fn upsert_platforms(&self, platforms: &[Platform]) -> Result<(), InternalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let mut ids = Vec::with_capacity(platforms.len());
            let mut external_ids = Vec::with_capacity(platforms.len());
            let mut names = Vec::with_capacity(platforms.len());
            let mut generations = Vec::with_capacity(platforms.len());

            for p in platforms {
                ids.push(p.id);
                external_ids.push(p.external_id.map(|ext_id| ext_id as i64));
                names.push(&p.name);
                generations.push(p.generation.map(|generation| generation as i32))
            }

            sqlx::query(sql(&QUERIES.platform.upsert_many))
                .bind(&ids)
                .bind(&external_ids)
                .bind(&names)
                .bind(&generations)
                .execute(&self.pool)
                .await?;

            Ok(())
        }.await;

        res.map_err(Into::into)
    }

    async fn get_platforms(&self) -> Result<Vec<Platform>, InternalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let platforms = sqlx::query_as::<_, PlatformDbRow>(sql(&QUERIES.platform.get_all))
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(Platform::from)
                .collect();

            Ok(platforms)
        }.await;

        res.map_err(Into::into)
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

impl From<LocalError> for InternalRepositoryError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::Sqlx(e) => InternalRepositoryError::Internal(e.to_string()),
        }
    }
}

#[derive(FromRow)]
pub struct PlatformDbRow {
    id: Uuid,
    external_id: Option<i32>,
    platform_name: String,
    generation: Option<i32>,
}

impl From<PlatformDbRow> for Platform {
    fn from(row: PlatformDbRow) -> Self {
        Self {
            id: row.id,
            external_id: row.external_id.map(|ext_id| ext_id as u64),
            name: row.platform_name,
            generation: row.generation.map(|generation| generation as u8),
        }
    }
}