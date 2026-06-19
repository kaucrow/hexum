use async_trait::async_trait;
use uuid::Uuid;
use thiserror::Error;
use anyhow::Result;

use crate::postgres::*;
use super::*;

#[derive(FromRow)]
struct ExternalIdRow {
    pub external_id: i32,
}

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
    async fn search_for_game(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<GameSearchResultItem>, InternalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let games = sqlx::query_as::<_, GameSearchResultDbRow>(sql(&QUERIES.game.search))
                .bind(query)
                .bind(limit as i64)
                .bind(offset as i64)
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(GameSearchResultItem::from)
                .collect();

            Ok(games)
        }.await;

        res.map_err(Into::into)
    }

    async fn get_external_ids_for_search(
        &self,
        query: &str,
    ) -> Result<Vec<u64>, InternalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let ids = sqlx::query_as::<_, ExternalIdRow>(sql(&QUERIES.game.get_external_ids_for_search))
                .bind(query)
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|row| row.external_id as u64)
                .collect();

            Ok(ids)
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
pub struct GameSearchResultDbRow {
    pub id: Uuid,
    pub external_id: i32,
    pub game_name: String,
}

impl From<GameSearchResultDbRow> for GameSearchResultItem {
    fn from(row: GameSearchResultDbRow) -> Self {
        Self {
            id: row.id,
            external_id: Some(row.external_id.into()),
            name: row.game_name,
        }
    }
}