use async_trait::async_trait;
use serde::Deserialize;
use thiserror::Error;
use uuid::Uuid;

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

    /// Builds the exclusion filter clause for an IGDB query body.
    /// Returns an empty string if there are no exclusion IDs.
    fn build_exclusion_clause(exclude_ids: &[u64]) -> String {
        if exclude_ids.is_empty() {
            String::new()
        } else {
            let list = exclude_ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            format!(" where id != ({});", list)
        }
    }
}

#[async_trait]
impl ExternalRepository for IgdbAdapter {
    async fn search_for_game(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
        exclude_ids: &[u64],
    ) -> Result<Vec<GameSearchResultItem>, ExternalRepositoryError> {
        let res: Result<Vec<GameSearchResultItem>, LocalError> = async {
            let url = "https://api.igdb.com/v4/games";
            let escaped_query = query.replace('"', "\\\"");
            let exclusion = Self::build_exclusion_clause(exclude_ids);

            let body = format!(
                "search \"{}\"; fields name;{} limit {}; offset {};",
                escaped_query, exclusion, limit, offset
            );

            let items: Vec<IgdbGameResponse> = videogame_api::Port::request(
                &self.igdb,
                url,
                &body,
            ).await?;

            let items = items
                .into_iter()
                .map(GameSearchResultItem::from)
                .collect();

            Ok(items)
        }
        .await;

        res.map_err(Into::into)
    }

    async fn count_search_results(
        &self,
        query: &str,
        exclude_ids: &[u64],
    ) -> Result<usize, ExternalRepositoryError> {
        let res: Result<usize, LocalError> = async {
            let url = "https://api.igdb.com/v4/games/count";
            let escaped_query = query.replace('"', "\\\"");
            let exclusion = Self::build_exclusion_clause(exclude_ids);

            let body = format!(
                "search \"{}\";{}",
                escaped_query, exclusion
            );

            let response: IgdbCountResponse = videogame_api::Port::request(
                &self.igdb,
                url,
                &body,
            ).await?;

            Ok(response.count)
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
struct IgdbGameResponse {
    pub id: u64,
    pub name: String,
}

#[derive(Deserialize)]
struct IgdbCountResponse {
    pub count: usize,
}

impl From<IgdbGameResponse> for GameSearchResultItem {
    fn from(response: IgdbGameResponse) -> Self {
        Self {
            id: Uuid::new_v4(),
            external_id: Some(response.id as i64),
            name: response.name,
        }
    }
}