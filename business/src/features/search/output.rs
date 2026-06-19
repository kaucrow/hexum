use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

use super::GameSearchResultItem;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait InternalRepository: Send + Sync + 'static {
    // ─── Getters ───
    /// Resolve a query to internal game UUIDs that match the query.
    /// Used to build the search result IDs cache.
    async fn get_game_ids_by_query(
        &self,
        query: &str,
    ) -> Result<Vec<Uuid>, InternalRepositoryError>;

    /// Resolve a batch of internal UUIDs to full GameSearchResultItems.
    /// Used after retrieving paginated UUIDs from cache.
    async fn get_games_by_ids(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<GameSearchResultItem>, InternalRepositoryError>;

    /// Get external IDs (as u64) for a set of internal UUIDs.
    /// Used to build the exclusion list for the external API.
    async fn get_external_ids_by_internal_ids(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<u64>, InternalRepositoryError>;
}

#[derive(Error, Debug)]
pub enum InternalRepositoryError {
    #[error("Error in Internal Repository Search: {0}")]
    Internal(String),
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ExternalRepository: Send + Sync + 'static {
    // ─── Getters ───
    async fn search_for_game(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
        exclude_ids: &[u64],
    ) -> Result<Vec<GameSearchResultItem>, ExternalRepositoryError>;

    /// Returns the total number of games in the external API matching the query,
    /// excluding the given IDs. Used for computing `total_count` metadata.
    async fn count_search_results(
        &self,
        query: &str,
        exclude_ids: &[u64],
    ) -> Result<usize, ExternalRepositoryError>;
}

#[derive(Error, Debug)]
pub enum ExternalRepositoryError {
    #[error("External error in Videogame API Search: {0}")]
    VideogameApi(String),
}

#[async_trait]
pub trait CacheRepository: Send + Sync {
    /// Gets a specific paginated slice of the cached IDs, plus the total count.
    async fn get_search_result_ids(
        &self,
        session_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Option<SearchResultPage>, CacheRepositoryError>;

    /// Gets ALL cached IDs for a session (used by the Service for building exclusion lists).
    async fn get_all_search_result_ids(
        &self,
        session_id: &str,
    ) -> Result<Option<Vec<Uuid>>, CacheRepositoryError>;

    /// Caches the full list of internal UUIDs for a new search session.
    async fn cache_search_result_ids(
        &self,
        session_id: &str,
        ids: &[Uuid],
    ) -> Result<(), CacheRepositoryError>;
}

pub struct SearchResultPage {
    pub ids: Vec<Uuid>,
    pub total_items: usize,
}

#[derive(Error, Debug)]
pub enum CacheRepositoryError {
    #[error("Error in Cache Repository Search: {0}")]
    Internal(String),
}