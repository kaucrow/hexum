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
    async fn get_game_ids_by_query(
        &self,
        query: &str,
    ) -> Result<Vec<Uuid>, InternalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let games = sqlx::query_as::<_, InternalIdDbRow>(sql(&QUERIES.game.get_ids_by_query))
                .bind(query)
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|row| row.id)
                .collect();

            Ok(games)
        }.await;

        res.map_err(Into::into)
    }

    async fn get_games_by_ids(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<GameSearchResultItem>, InternalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let games = sqlx::query_as::<_, GameSearchResultDbRow>(sql(&QUERIES.game.get_games_by_ids))
                .bind(ids)
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(GameSearchResultItem::from)
                .collect();

            Ok(games)
        }.await;

        res.map_err(Into::into)
    }

    async fn get_external_ids_by_internal_ids(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<u64>, InternalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let ids = sqlx::query_as::<_, ExternalIdDbRow>(sql(&QUERIES.game.get_external_ids_by_internal_ids))
                .bind(ids)
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

#[derive(FromRow)]
struct InternalIdDbRow {
    pub id: Uuid,
}

#[derive(FromRow)]
struct ExternalIdDbRow {
    pub external_id: i32,
}