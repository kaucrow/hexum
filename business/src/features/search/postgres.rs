use crate::{
    prelude::*,
    postgres::*,
    features::base::pagination,
};
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

impl PaginatedInternalRepository for PostgresAdapter {}

// ─── search::output::InternalRepository ─────────────────────

#[async_trait]
impl InternalRepository for PostgresAdapter {
    async fn get_games_by_ids(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<GameResultItem>, InternalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let games = sqlx::query_as::<_, GameResultDbRow>(sql(&QUERIES.game.get_games_by_ids))
                .bind(ids)
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(GameResultItem::from)
                .collect();

            Ok(games)
        }.await;

        res.map_err(Into::into)
    }

    async fn get_game_ids_by_criteria(
        &self,
        search: &GameSearch,
    ) -> Result<Vec<Uuid>, InternalRepositoryError> {
        let res: Result<_, LocalError> = async {
            use sqlx::QueryBuilder;

            let has_query = search.query.as_ref().map(|q| !q.is_empty()).unwrap_or(false);
            let has_platforms = search.platforms.as_ref().map(|p| !p.is_empty()).unwrap_or(false);

            if !has_query && !has_platforms {
                return Ok(vec![]);
            }

            let mut builder = QueryBuilder::new("SELECT g.id FROM game.game g");

            let mut where_added = false;

            if has_query {
                let q = search.query.as_deref().unwrap();
                builder.push(" WHERE g.game_name % ");
                builder.push_bind(q);
                where_added = true;
            }

            if has_platforms {
                let platform_uuids = search.platforms.as_ref().unwrap();

                if where_added {
                    builder.push(" AND ");
                } else {
                    builder.push(" WHERE ");
                }

                builder.push("EXISTS (SELECT 1 FROM game.game_platform gp WHERE gp.game_id = g.id AND gp.platform_id = ANY(");
                builder.push_bind(platform_uuids);
                builder.push("))");
            }

            if has_query {
                let q = search.query.as_deref().unwrap();
                builder.push(" ORDER BY similarity(g.game_name, ");
                builder.push_bind(q);
                builder.push(") DESC, g.id ASC");
            } else {
                builder.push(" ORDER BY g.game_name ASC, g.id ASC");
            }

            let games = builder
                .build_query_as::<InternalIdDbRow>()
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|row| row.id)
                .collect();

            Ok(games)
        }.await;

        res.map_err(Into::into)
    }

    async fn get_external_platform_ids_by_internal_ids(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<u64>, InternalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let external_ids = sqlx::query_as::<_, ExternalIdDbRow>(sql(&QUERIES.platform.get_external_ids_by_internal_ids))
                .bind(ids)
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|row| row.external_id as u64)
                .collect();

            Ok(external_ids)
        }.await;

        res.map_err(Into::into)
    }
}

// ─── pagination::InternalRepository ─────────────────────────

#[async_trait]
impl pagination::InternalRepository for PostgresAdapter {
    async fn get_external_ids_by_internal_ids(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<u64>, pagination::InternalRepositoryError> {
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

impl From<LocalError> for pagination::InternalRepositoryError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::Sqlx(e) => pagination::InternalRepositoryError::Internal(e.to_string()),
        }
    }
}

#[derive(FromRow)]
pub struct GameResultDbRow {
    pub id: Uuid,
    pub external_id: i32,
    pub game_name: String,
}

impl From<GameResultDbRow> for GameResultItem {
    fn from(row: GameResultDbRow) -> Self {
        Self {
            id: Some(row.id),
            external_id: row.external_id as u64,
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