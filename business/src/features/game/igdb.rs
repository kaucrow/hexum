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
    async fn get_game(&self, id: u64) -> Result<Option<Game>, ExternalRepositoryError> {
        let res: Result<Option<Game>, LocalError> = async {
            let url = "https://api.igdb.com/v4/games";

            let body = format!("fields id, name, platforms; where id = {};", id);

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
struct IgdbGameResponse {
    pub id: u64,
    pub name: String,
    pub platforms: Option<Vec<u64>>,
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

        Self {
            id: Uuid::new_v4(),
            external_id: Some(response.id),
            name: response.name,
            platforms,
        }
    }
}