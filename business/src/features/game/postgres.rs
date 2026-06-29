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

    async fn increment_view_count(&self, id: &Uuid) -> Result<(), InternalRepositoryError> {
        let res: Result<_, LocalError> = async {
            sqlx::query(sql(&QUERIES.game.increment_view_count))
                .bind(id)
                .execute(&self.pool)
                .await?;

            Ok(())
        }.await;

        res.map_err(Into::into)
    }

    async fn get_popular_items_by_view_count(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<PopularGameItem>, InternalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let items = sqlx::query_as::<_, PopularGameItemDbRow>(sql(&QUERIES.game.get_popular_by_view_count))
                .bind(limit as i64)
                .bind(offset as i64)
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(PopularGameItem::from)
                .collect();

            Ok(items)
        }.await;

        res.map_err(Into::into)
    }

    async fn count_all_games(&self) -> Result<usize, InternalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let count = sqlx::query_scalar::<_, i32>(sql(&QUERIES.game.count_all_games))
                .fetch_one(&self.pool)
                .await?;

            Ok(count as usize)
        }.await;

        res.map_err(Into::into)
    }

    async fn sync_db_game_from_external(&self, game: &Game) -> Result<Uuid, InternalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let mut tx = self.pool.begin().await?;

            // ─── Upsert genres ──────────────────────────────────────
            let genre_uuids: Vec<Uuid> = if game.genres.is_empty() {
                vec![]
            } else {
                let ids: Vec<Uuid> = game.genres.iter().map(|g| g.id).collect();
                let ext_ids: Vec<i32> = game.genres.iter()
                    .map(|g| g.external_id.map(|e| e as i32).unwrap_or(0))
                    .collect();
                let names: Vec<String> = game.genres.iter()
                    .map(|g| g.name.clone())
                    .collect();

                sqlx::query_as::<_, ExternalIdMappingDbRow>(sql(&QUERIES.genre.upsert_many))
                    .bind(&ids)
                    .bind(&ext_ids)
                    .bind(&names)
                    .fetch_all(&mut *tx)
                    .await?
                    .into_iter()
                    .map(|row| row.id)
                    .collect()
            };

            // ─── Upsert companies ───────────────────────────────────
            // Collect unique companies (same company can be both developer and publisher)
            let mut company_entries: Vec<(Uuid, i32, String)> = Vec::new();
            for gc in &game.companies {
                let ext_id = gc.company.external_id.map(|e| e as i32).unwrap_or(0);
                // Avoid duplicates
                if !company_entries.iter().any(|(_, eid, _)| *eid == ext_id) {
                    company_entries.push((gc.company.id, ext_id, gc.company.name.clone()));
                }
            }

            if !company_entries.is_empty() {
                let (ids, ext_ids, names): (Vec<Uuid>, Vec<i32>, Vec<String>) =
                    company_entries.into_iter().fold(
                        (vec![], vec![], vec![]),
                        |(mut ids, mut eids, mut names), (id, eid, name)| {
                            ids.push(id);
                            eids.push(eid);
                            names.push(name);
                            (ids, eids, names)
                        },
                    );

                sqlx::query(sql(&QUERIES.company.upsert_many))
                    .bind(&ids)
                    .bind(&ext_ids)
                    .bind(&names)
                    .execute(&mut *tx)
                    .await?;
            }

            // Build a mapping from external_id -> internal UUID for companies
            let company_ext_to_uuid: std::collections::HashMap<i32, Uuid> = {
                let ext_ids: Vec<i32> = game.companies.iter()
                    .filter_map(|gc| gc.company.external_id.map(|e| e as i32))
                    .collect();
                if ext_ids.is_empty() {
                    std::collections::HashMap::new()
                } else {
                    sqlx::query_as::<_, ExternalIdMappingDbRow>(sql(&QUERIES.company.get_internal_ids_by_external_ids))
                        .bind(&ext_ids)
                        .fetch_all(&mut *tx)
                        .await?
                        .into_iter()
                        .map(|row| (row.external_id, row.id))
                        .collect()
                }
            };

            // ─── Resolve platform external IDs to internal UUIDs ───
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
                        .fetch_all(&mut *tx)
                        .await?
                        .into_iter()
                        .map(|row| row.id)
                        .collect()
                }
            };

            // ─── Upsert game and re-link platforms ─────────────────
            let game_internal_id = sqlx::query_as::<_, InternalIdDbRow>(sql(&QUERIES.game.upsert_from_external))
                .bind(game.id)
                .bind(game.external_id.map(|eid| eid as i64))
                .bind(&game.name)
                .bind(game.first_release_date)
                .bind(&game.cover_url)
                .bind(&game.summary)
                .bind(&platform_uuids)
                .fetch_one(&mut *tx)
                .await?
                .id;

            // ─── Re-link genres ────────────────────────────────────
            sqlx::query(sql(&QUERIES.game.delete_game_genres))
                .bind(game_internal_id)
                .execute(&mut *tx)
                .await?;

            if !genre_uuids.is_empty() {
                sqlx::query(sql(&QUERIES.game.insert_game_genres))
                    .bind(game_internal_id)
                    .bind(&genre_uuids)
                    .execute(&mut *tx)
                    .await?;
            }

            // ─── Re-link companies ─────────────────────────────────
            sqlx::query(sql(&QUERIES.game.delete_game_companies))
                .bind(game_internal_id)
                .execute(&mut *tx)
                .await?;

            if !game.companies.is_empty() {
                let (company_uuids, roles): (Vec<Uuid>, Vec<String>) = game.companies.iter()
                    .filter_map(|gc| {
                        let ext_id = gc.company.external_id.map(|e| e as i32)?;
                        let uuid = company_ext_to_uuid.get(&ext_id)?;
                        let role = match gc.role {
                            CompanyRole::Developer => "developer".to_string(),
                            CompanyRole::Publisher => "publisher".to_string(),
                        };
                        Some((*uuid, role))
                    })
                    .unzip();

                if !company_uuids.is_empty() {
                    sqlx::query(sql(&QUERIES.game.insert_game_companies))
                        .bind(game_internal_id)
                        .bind(&company_uuids)
                        .bind(&roles)
                        .execute(&mut *tx)
                        .await?;
                }
            }

            tx.commit().await?;

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
struct PopularGameItemDbRow {
    pub id: Uuid,
    pub external_id: Option<i32>,
    pub game_name: String,
    pub cover_url: Option<String>,
}

impl From<PopularGameItemDbRow> for PopularGameItem {
    fn from(row: PopularGameItemDbRow) -> Self {
        Self {
            id: Some(row.id),
            external_id: row.external_id.unwrap_or(0) as u64,
            name: row.game_name,
            cover_url: row.cover_url,
        }
    }
}

#[derive(FromRow)]
pub struct ExternalIdMappingDbRow {
    pub id: Uuid,
    pub external_id: i32,
}

#[derive(FromRow)]
pub struct GameDbRow {
    id: Uuid,
    external_id: Option<i32>,
    game_name: String,
    first_release_date: Option<DateTime<Utc>>,
    cover_url: Option<String>,
    rating: Option<f64>,
    critic_rating: Option<f64>,
    total_rating_count: Option<i32>,
    summary: Option<String>,
    #[sqlx(json)]
    platforms: Vec<PlatformDbRow>,
    #[sqlx(json)]
    genres: Vec<GenreDbRow>,
    #[sqlx(json)]
    companies: Vec<CompanyDbRow>,
}

impl From<GameDbRow> for Game {
    fn from(row: GameDbRow) -> Self {
        let platforms =
            row.platforms
            .into_iter()
            .map(Platform::from)
            .collect();

        let genres =
            row.genres
            .into_iter()
            .map(Genre::from)
            .collect();

        let companies =
            row.companies
            .into_iter()
            .map(GameCompany::from)
            .collect();

        Self {
            id: row.id,
            external_id: row.external_id.map(|ext_id| ext_id as u64),
            name: row.game_name,
            first_release_date: row.first_release_date,
            cover_url: row.cover_url,
            rating: row.rating,
            critic_rating: row.critic_rating,
            total_rating_count: row.total_rating_count,
            summary: row.summary,
            platforms,
            genres,
            companies,
        }
    }
}

#[derive(FromRow, Deserialize)]
pub struct GenreDbRow {
    pub id: Uuid,
    pub external_id: Option<i32>,
    pub genre_name: String,
}

impl From<GenreDbRow> for Genre {
    fn from(row: GenreDbRow) -> Self {
        Self {
            id: row.id,
            external_id: row.external_id.map(|ext_id| ext_id as u64),
            name: row.genre_name,
        }
    }
}

#[derive(FromRow, Deserialize)]
pub struct CompanyDbRow {
    pub id: Uuid,
    pub external_id: Option<i32>,
    pub company_name: String,
    pub role: String,
}

impl From<CompanyDbRow> for GameCompany {
    fn from(row: CompanyDbRow) -> Self {
        let role = match row.role.as_str() {
            "developer" => CompanyRole::Developer,
            "publisher" => CompanyRole::Publisher,
            _ => CompanyRole::Developer, // Default fallback
        };

        Self {
            company: Company {
                id: row.id,
                external_id: row.external_id.map(|ext_id| ext_id as u64),
                name: row.company_name,
            },
            role,
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