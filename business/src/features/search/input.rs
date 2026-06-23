use crate::prelude::*;
use crate::features::base::pagination::{
    CacheRepositoryError, ExternalRepositoryError,
    InternalRepositoryError, PaginationError,
};
use super::SearchResult;

#[async_trait]
pub trait UseCase: Send + Sync + 'static {
    /// Search for games matching the given criteria and return a paginated page.
    ///
    /// Internally delegates all pagination logic to the feature-agnostic
    /// [`PaginatedQuery::get_items_page`].
    async fn search_for_game(
        &self,
        search: GameSearch,
        search_id: Option<Uuid>,
        limit: usize,
        offset: usize,
    ) -> Result<SearchResult, UseCaseError>;
}

pub struct GameSearch {
    pub query: Option<String>,
    pub platforms: Option<Vec<Uuid>>,
}

#[derive(Clone)]
pub struct PaginationGameSearch {
    pub query: Option<String>,
    pub ext_platforms: Option<Vec<u64>>,
}

#[derive(Error, Debug)]
pub enum UseCaseError {
    /// Videogame API error.
    #[error("External error in Videogame API (Search Service): {0}")]
    VideogameApi(String),

    /// The search has no fields.
    #[error("The search has no fields.")]
    EmptySearch,

    /// Unexpected internal error.
    #[error("Search service: {0}.")]
    Internal(String),
}

impl From<InternalRepositoryError> for UseCaseError {
    fn from(e: InternalRepositoryError) -> Self {
        UseCaseError::Internal(e.to_string())
    }
}

impl From<super::output::InternalRepositoryError> for UseCaseError {
    fn from(e: super::output::InternalRepositoryError) -> Self {
        UseCaseError::Internal(e.to_string())
    }
}

impl From<CacheRepositoryError> for UseCaseError {
    fn from(e: CacheRepositoryError) -> Self {
        UseCaseError::Internal(e.to_string())
    }
}

impl From<ExternalRepositoryError> for UseCaseError {
    fn from(e: ExternalRepositoryError) -> Self {
        UseCaseError::VideogameApi(e.to_string())
    }
}

impl From<PaginationError> for UseCaseError {
    fn from(e: PaginationError) -> Self {
        match e {
            PaginationError::External(e) => UseCaseError::VideogameApi(e.to_string()),
            PaginationError::Internal(e) => UseCaseError::Internal(e.to_string()),
            PaginationError::Cache(e) => UseCaseError::Internal(e.to_string()),
        }
    }
}