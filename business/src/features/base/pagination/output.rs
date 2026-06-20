use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

use super::*;

// ─── Internal repository ───────────────────────────────────

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait InternalRepository: Send + Sync + 'static {
    /// Resolve a query to internal game UUIDs that match the query.
    async fn get_game_ids_by_query(
        &self,
        query: &str,
    ) -> Result<Vec<Uuid>, InternalRepositoryError>;

    /// Resolve a batch of internal UUIDs to full GameResultItems.
    async fn get_games_by_ids(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<GameResultItem>, InternalRepositoryError>;

    /// Get external IDs (as u64) for a set of internal UUIDs.
    /// Used to build the exclusion list for the external API.
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

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ExternalRepository: Send + Sync + 'static {
    /// Search the external API with exclusion support.
    async fn search(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
        exclude_ids: &[u64],
    ) -> Result<Vec<GameResultItem>, ExternalRepositoryError>;

    /// Count total results in the external API matching the query,
    /// excluding the given IDs.
    async fn count(
        &self,
        query: &str,
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
        session_id: &str,
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