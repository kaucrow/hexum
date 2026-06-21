use crate::prelude::*;

// ─── Internal repository ───────────────────────────────────

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait InternalRepository: Send + Sync + 'static {
    /// Get external IDs for a set of internal UUIDs (for building exclusion lists).
    async fn get_external_ids_by_internal_ids(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<u64>, InternalRepositoryError>;
}

#[derive(Error, Debug)]
pub enum InternalRepositoryError {
    #[error("Error in Internal Repository: {0}")]
    Internal(String),
}

// ─── External repository ───────────────────────────────────

#[async_trait]
pub trait ExternalRepository: Send + Sync + 'static {
    /// The item type this repository returns (e.g. `GameResultItem`).
    type Item: Clone + Send + 'static;

    /// The search criteria type (e.g. `GameSearch`).
    type Search: Clone + Send + 'static;

    /// Fetch a batch of items from the external API.
    async fn fetch(
        &self,
        search: &Self::Search,
        limit: usize,
        offset: usize,
        exclude_ids: &[u64],
    ) -> Result<Vec<Self::Item>, ExternalRepositoryError>;

    /// Count total results in the external API matching the criteria,
    /// excluding the given IDs.
    async fn count(
        &self,
        search: &Self::Search,
        exclude_ids: &[u64],
    ) -> Result<usize, ExternalRepositoryError>;
}

#[derive(Error, Debug)]
pub enum ExternalRepositoryError {
    #[error("External error: {0}")]
    External(String),
}

// ─── Cache repository ──────────────────────────────────────

#[async_trait]
pub trait CacheRepository: Send + Sync {
    /// Gets a specific paginated slice of the cached IDs, plus the total count.
    async fn get_cached_ids(
        &self,
        key_prefix: &str,
        session_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Option<CachedIdsPage>, CacheRepositoryError>;

    /// Gets ALL cached IDs for a session (used for building exclusion lists).
    async fn get_all_cached_ids(
        &self,
        key_prefix: &str,
        session_id: &str,
    ) -> Result<Option<Vec<Uuid>>, CacheRepositoryError>;

    /// Caches the full list of internal UUIDs for a new session.
    async fn cache_ids(
        &self,
        key_prefix: &str,
        pagination_id: &str,
        ids: &[Uuid],
    ) -> Result<(), CacheRepositoryError>;
}

pub struct CachedIdsPage {
    pub ids: Vec<Uuid>,
    pub total_items: usize,
}

#[derive(Error, Debug)]
pub enum CacheRepositoryError {
    #[error("Error in Cache Repository: {0}")]
    Internal(String),
}

// ─── Unified pagination error ──────────────────────────────

#[derive(Error, Debug)]
pub enum PaginationError {
    #[error("Internal repository: {0}")]
    Internal(#[from] InternalRepositoryError),

    #[error("External repository: {0}")]
    External(#[from] ExternalRepositoryError),

    #[error("Cache repository: {0}")]
    Cache(#[from] CacheRepositoryError),
}
