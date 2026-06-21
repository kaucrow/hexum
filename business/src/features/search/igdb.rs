use crate::{
    prelude::*,
    features::videogame_api,
};
use crate::features::base::pagination;
use super::*;

/// IGDB adapter for search queries.
#[derive(Clone)]
pub struct IgdbAdapter {
    igdb: videogame_api::IgdbAdapter,
}

impl IgdbAdapter {
    pub fn new(
        igdb: videogame_api::IgdbAdapter,
    ) -> Self {
        Self { igdb }
    }

    /// Builds the complete IGDB request body dynamically, only including
    /// clauses for what's present (search, exclusion, platform filter,
    /// fields, limit/offset).
    fn build_body(
        &self,
        search: &PaginationGameSearch,
        exclude_ids: &[u64],
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> String {
        let mut parts: Vec<String> = Vec::new();

        if let Some(query) = &search.query {
            let escaped = query.replace('"', "\\\"");
            parts.push(format!("search \"{}\";", escaped));
        }

        if limit.is_some() {
            parts.push("fields name;".to_string());
        }

        let mut conditions: Vec<String> = Vec::new();

        if !exclude_ids.is_empty() {
            let list = exclude_ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            conditions.push(format!("id != ({})", list));
        }

        if let Some(ref pids) = search.ext_platforms {
            if !pids.is_empty() {
                let list = pids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                conditions.push(format!("platforms = ({})", list));
            }
        }

        if !conditions.is_empty() {
            parts.push(format!("where {};", conditions.join(" & ")));
        }

        if let Some(l) = limit {
            parts.push(format!("limit {};", l));
        }
        if let Some(o) = offset {
            parts.push(format!("offset {};", o));
        }

        parts.join(" ")
    }
}

#[async_trait]
impl pagination::ExternalRepository for IgdbAdapter {
    type Item = GameResultItem;
    type Search = PaginationGameSearch;

    async fn fetch(
        &self,
        search: &Self::Search,
        limit: usize,
        offset: usize,
        exclude_ids: &[u64],
    ) -> Result<Vec<Self::Item>, pagination::ExternalRepositoryError> {
        let res: Result<Vec<GameResultItem>, LocalError> = async {
            let body = self.build_body(search, exclude_ids, Some(limit), Some(offset));

            let items: Vec<IgdbGameResponse> = videogame_api::Port::request(
                &self.igdb,
                "https://api.igdb.com/v4/games",
                &body,
            ).await?;

            Ok(items.into_iter().map(GameResultItem::from).collect())
        }.await;

        res.map_err(Into::into)
    }

    async fn count(
        &self,
        search: &Self::Search,
        exclude_ids: &[u64],
    ) -> Result<usize, pagination::ExternalRepositoryError> {
        let res: Result<usize, LocalError> = async {
            let body = self.build_body(search, exclude_ids, None, None);

            let response: IgdbCountResponse = videogame_api::Port::request(
                &self.igdb,
                "https://api.igdb.com/v4/games/count",
                &body,
            ).await?;

            Ok(response.count)
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

impl From<LocalError> for pagination::ExternalRepositoryError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::IgdbHttp(msg) => pagination::ExternalRepositoryError::External(msg),
            LocalError::IgdbInternal(msg) => pagination::ExternalRepositoryError::External(msg),
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

impl From<IgdbGameResponse> for GameResultItem {
    fn from(response: IgdbGameResponse) -> Self {
        Self {
            id: None,
            external_id: response.id,
            name: response.name,
        }
    }
}