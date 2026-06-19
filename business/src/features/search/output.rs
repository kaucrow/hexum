use async_trait::async_trait;
use thiserror::Error;

use super::GameSearchResultItem;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait InternalRepository: Send + Sync + 'static {
    // ─── Getters ───
    async fn search_for_game(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<GameSearchResultItem>, InternalRepositoryError>;

    /// Returns all external IDs from the internal DB matching the search query.
    /// Used to build the exclusion list when querying external APIs.
    async fn get_external_ids_for_search(
        &self,
        query: &str,
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
    /// Attempts to retrieve the cached external IDs for a specific search query.
    /// Returns `None` if the query hasn't been cached yet.
    async fn get_search_exclusion_ids(&self, query: &str) -> Result<Option<Vec<u64>>, CacheRepositoryError>;

    /// Caches the list of external IDs belonging to the internal DB for a specific query.
    async fn cache_search_exclusion_ids(&self, query: &str, ids: &[u64]) -> Result<(), CacheRepositoryError>;
}

#[derive(Error, Debug)]
pub enum CacheRepositoryError {
    #[error("Error in Cache Repository Search: {0}")]
    Internal(String),
}