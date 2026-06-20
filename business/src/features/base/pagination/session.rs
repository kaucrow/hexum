use uuid::Uuid;

use crate::prelude::*;
use super::*;

/// Manages the lifecycle of a paginated, session-based result set.
/// Owns a cache repository & provides paginated access to cached UUID lists.
#[derive(Clone)]
pub struct PaginatorSession {
    cache_repo: Arc<dyn CacheRepository>,
    prefix_key: String,
}

impl PaginatorSession {
    pub fn new(cache_repo: Arc<dyn CacheRepository>, prefix_key: &str) -> Self {
        Self {
            cache_repo,
            prefix_key: prefix_key.to_string(),
        }
    }

    /// Try to get a paginated slice from an existing session.
    /// Returns `None` if the session doesn't exist or has expired.
    pub async fn get_page(
        &self,
        session_id: &Uuid,
        limit: usize,
        offset: usize,
    ) -> Result<Option<CachedIdsPage>, CacheRepositoryError> {
        self.cache_repo
            .get_cached_ids(&self.prefix_key, &session_id.to_string(), limit, offset)
            .await
    }

    /// Cache a list of UUIDs for a new session.
    pub async fn cache_session(
        &self,
        session_id: &Uuid,
        ids: &[Uuid],
    ) -> Result<(), CacheRepositoryError> {
        self.cache_repo
            .cache_ids(&self.prefix_key, &session_id.to_string(), ids)
            .await
    }

    /// Get all cached UUIDs for a session.
    pub async fn get_all_ids(
        &self,
        session_id: &Uuid,
    ) -> Result<Option<Vec<Uuid>>, CacheRepositoryError> {
        self.cache_repo
            .get_all_cached_ids(&self.prefix_key, &session_id.to_string())
            .await
    }
}