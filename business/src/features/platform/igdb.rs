use crate::{
    prelude::*,
    features::videogame_api,
};
use super::*;

#[derive(Clone)]
pub struct IgdbAdapter {
    igdb: videogame_api::IgdbAdapter,
}

impl IgdbAdapter {
    pub fn new(igdb: videogame_api::IgdbAdapter) -> Self {
        Self { igdb }
    }
}

#[async_trait]
impl ExternalRepository for IgdbAdapter {
    async fn get_platforms(&self) -> Result<Vec<Platform>, ExternalRepositoryError> {
        let res: Result<Vec<Platform>, LocalError> = async {
            let url = "https://api.igdb.com/v4/platforms";

            let body = "fields id, name, generation; limit 500;";

            let items: Vec<IgdbPlatformResponse> = videogame_api::Port::request(
                &self.igdb,
                url,
                body,
            ).await?;

            let items = items
                .into_iter()
                .map(Platform::from)
                .collect();

            Ok(items)
        }
        .await;

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
            external_id: Some(response.id as u64),
            name: response.name,
            generation: response.generation,
        }
    }
}