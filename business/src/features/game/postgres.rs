use crate::{
    prelude::*,
    postgres::*,
    features::platform::postgres::PlatformDbRow,
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

#[async_trait]
impl InternalRepository for PostgresAdapter {
    async fn get_game(&self, id: &Uuid) -> Result<Option<Game>, InternalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let game = sqlx::query_as::<_, GameDbRow>(sql(&QUERIES.game.get_full_data))
                .bind(id)
                .fetch_optional(&self.pool)
                .await?
                .map(Game::from);

            Ok(game)
        }.await;

        res.map_err(Into::into)
    }

    async fn sync_db_game_from_external(&self, game: &Game) -> Result<Uuid, InternalRepositoryError> {
        let res: Result<_, LocalError> = async {
            // Resolve IGDB platform external IDs to internal UUIDs
            let platform_uuids = if game.platforms.is_empty() {
                vec![]
            } else {
                let ext_ids: Vec<i32> = game.platforms
                    .iter()
                    .filter_map(|p| p.external_id.map(|eid| eid as i32))
                    .collect();

                if ext_ids.is_empty() {
                    vec![]
                } else {
                    sqlx::query_as::<_, InternalIdDbRow>(sql(&QUERIES.platform.get_internal_ids_by_external_ids))
                        .bind(&ext_ids)
                        .fetch_all(&self.pool)
                        .await?
                        .into_iter()
                        .map(|row| row.id)
                        .collect()
                }
            };

            // Upsert game and re-link platforms
            let game_internal_id = sqlx::query_as::<_, InternalIdDbRow>(sql(&QUERIES.game.upsert_from_external))
                .bind(game.id)
                .bind(game.external_id.map(|eid| eid as i64))
                .bind(&game.name)
                .bind(&platform_uuids)
                .fetch_one(&self.pool)
                .await?
                .id;

            Ok(game_internal_id)
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
pub struct InternalIdDbRow {
    pub id: Uuid,
}

#[derive(FromRow)]
pub struct GameDbRow {
    id: Uuid,
    external_id: Option<i32>,
    game_name: String,
    #[sqlx(json)]
    platforms: Vec<PlatformDbRow>,
}

impl From<GameDbRow> for Game {
    fn from(row: GameDbRow) -> Self {
        let platforms =
            row.platforms
            .into_iter()
            .map(|platform_row| Platform::from(platform_row))
            .collect();

        Self {
            id: row.id,
            external_id: row.external_id.map(|ext_id| ext_id as u64),
            name: row.game_name,
            platforms,
        }
    }
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