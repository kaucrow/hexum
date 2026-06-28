use crate::{
    prelude::*,
    features::videogame_api,
};
use super::*;

#[derive(Clone)]
pub struct IgdbAdapter {
    igdb: Arc<videogame_api::IgdbClient>,
}

impl IgdbAdapter {
    pub fn new(igdb: Arc<videogame_api::IgdbClient>) -> Self {
        Self { igdb }
    }
}

#[async_trait]
impl ExternalRepository for IgdbAdapter {
    async fn get_popular_games(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<(Vec<PopularGameItem>, usize), ExternalRepositoryError> {
        let res: Result<(Vec<PopularGameItem>, usize), LocalError> = async {
            let url = "https://api.igdb.com/v4/games";

            // Fetch the page of popular games
            let body = format!(
                "fields name, cover.url; sort popularity desc; limit {}; offset {};",
                limit, offset
            );

            let items: Vec<IgdbPopularGameResponse> = videogame_api::Port::request(
                &self.igdb,
                url,
                &body,
            ).await?;

            // Get total count
            let count_body = "sort popularity desc;".to_string();
            let count_response: IgdbCountResponse = videogame_api::Port::request(
                &self.igdb,
                "https://api.igdb.com/v4/games/count",
                &count_body,
            ).await?;

            let games: Vec<PopularGameItem> = items
                .into_iter()
                .map(|item| {
                    let cover_url = item.cover.map(|c| {
                        if c.url.starts_with("//") {
                            format!("https:{}", c.url)
                        } else {
                            c.url
                        }
                    });

                    PopularGameItem {
                        id: None,
                        external_id: item.id,
                        name: item.name,
                        cover_url,
                    }
                })
                .collect();

            Ok((games, count_response.count))
        }.await;

        res.map_err(Into::into)
    }

    async fn get_game(&self, id: u64) -> Result<Option<Game>, ExternalRepositoryError> {
        let res: Result<Option<Game>, LocalError> = async {
            let url = "https://api.igdb.com/v4/games";

            let body = format!(
                "fields id, name, platforms, first_release_date, cover.url, \
                 genres.name, involved_companies.company.name, \
                 involved_companies.publisher, involved_companies.developer, \
                 rating, aggregated_rating, total_rating_count, summary; \
                 where id = {};",
                id
            );

            let items: Vec<IgdbGameResponse> = videogame_api::Port::request(
                &self.igdb,
                url,
                &body,
            ).await?;

            let game = items.into_iter().next().map(Game::from);

            Ok(game)
        }.await;

        res.map_err(Into::into)
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error("IGDB HTTP error: {0}")]
    IgdbHttp(String),

    #[error("IGDB Internal (auth) error: {0}")]
    IgdbInternal(String),
}

impl From<videogame_api::PortError> for LocalError {
    fn from(e: videogame_api::PortError) -> Self {
        match e {
            videogame_api::PortError::Http(msg) => LocalError::IgdbHttp(msg),
            videogame_api::PortError::Internal(msg) => LocalError::IgdbInternal(msg),
        }
    }
}

impl From<LocalError> for ExternalRepositoryError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::IgdbHttp(msg) => ExternalRepositoryError::VideogameApi(msg),
            LocalError::IgdbInternal(msg) => ExternalRepositoryError::VideogameApi(msg),
        }
    }
}

// ─── IGDB responses ──────────────────────────────────────────

#[derive(Deserialize)]
struct IgdbPopularGameResponse {
    pub id: u64,
    pub name: String,
    pub cover: Option<IgdbCoverResponse>,
}

#[derive(Deserialize)]
struct IgdbCountResponse {
    pub count: usize,
}

#[derive(Deserialize)]
struct IgdbPlatformResponse {
    pub id: u64,
    pub name: String,
    pub generation: Option<u8>,
}

impl From<IgdbPlatformResponse> for Platform {
    fn from(response: IgdbPlatformResponse) -> Self {
        Self {
            id: Uuid::new_v4(),
            external_id: Some(response.id),
            name: response.name,
            generation: response.generation,
        }
    }
}

#[derive(Deserialize)]
struct IgdbGenreResponse {
    pub id: u64,
    pub name: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct IgdbCoverResponse {
    pub id: u64,
    pub url: String,
}

#[derive(Deserialize)]
struct IgdbCompanyResponse {
    pub id: u64,
    pub name: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct IgdbInvolvedCompanyResponse {
    pub id: u64,
    pub company: IgdbCompanyResponse,
    pub publisher: bool,
    pub developer: bool,
}

#[derive(Deserialize)]
struct IgdbGameResponse {
    pub id: u64,
    pub name: String,
    pub platforms: Option<Vec<u64>>,
    pub first_release_date: Option<i64>,
    pub cover: Option<IgdbCoverResponse>,
    pub genres: Option<Vec<IgdbGenreResponse>>,
    pub involved_companies: Option<Vec<IgdbInvolvedCompanyResponse>>,
    pub rating: Option<f64>,
    pub aggregated_rating: Option<f64>,
    pub total_rating_count: Option<i32>,
    pub summary: Option<String>,
}

impl From<IgdbGameResponse> for Game {
    fn from(response: IgdbGameResponse) -> Self {
        let platforms = response
            .platforms
            .unwrap_or_default()
            .into_iter()
            .map(|ext_id| Platform {
                id: Uuid::new_v4(),
                external_id: Some(ext_id),
                name: String::new(),
                generation: None,
            })
            .collect();

        let genres = response
            .genres
            .unwrap_or_default()
            .into_iter()
            .map(|g| Genre {
                id: Uuid::new_v4(),
                external_id: Some(g.id),
                name: g.name,
            })
            .collect();

        let companies = response
            .involved_companies
            .unwrap_or_default()
            .into_iter()
            .flat_map(|ic| {
                let mut roles = Vec::new();
                if ic.developer {
                    roles.push(GameCompany {
                        company: Company {
                            id: Uuid::new_v4(),
                            external_id: Some(ic.company.id),
                            name: ic.company.name.clone(),
                        },
                        role: CompanyRole::Developer,
                    });
                }
                if ic.publisher {
                    roles.push(GameCompany {
                        company: Company {
                            id: Uuid::new_v4(),
                            external_id: Some(ic.company.id),
                            name: ic.company.name,
                        },
                        role: CompanyRole::Publisher,
                    });
                }
                roles
            })
            .collect();

        let first_release_date = response
            .first_release_date
            .and_then(|ts| DateTime::from_timestamp(ts, 0));

        let cover_url = response.cover.map(|c| {
            if c.url.starts_with("//") {
                format!("https:{}", c.url)
            } else {
                c.url
            }
        });

        Self {
            id: Uuid::new_v4(),
            external_id: Some(response.id),
            name: response.name,
            first_release_date,
            cover_url,
            rating: response.rating,
            aggregated_rating: response.aggregated_rating,
            total_rating_count: response.total_rating_count,
            summary: response.summary,
            platforms,
            genres,
            companies,
        }
    }
}