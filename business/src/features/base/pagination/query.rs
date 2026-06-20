use crate::prelude::*;
use super::*;

/// Allows for pagination session management with external API fallback for paginated queries.
/// Feature-agnostic: Each feature provides its own logic for obtaining the initial
/// UUID list and then delegates session lifecycle and external fallback to this struct.
#[derive(Clone)]
pub struct PaginatedQuery {
    session: PaginatorSession,
    internal_repo: Arc<dyn InternalRepository>,
    external_repo: Arc<dyn ExternalRepository>,
}

impl PaginatedQuery {
    pub fn new(
        session: PaginatorSession,
        internal_repo: Arc<dyn InternalRepository>,
        external_repo: Arc<dyn ExternalRepository>,
    ) -> Self {
        Self { session, internal_repo, external_repo }
    }

    /// Access the session paginator for session lifecycle operations.
    pub fn session(&self) -> &PaginatorSession {
        &self.session
    }

    /// Access the internal repository for ID fetching & item resolution.
    pub fn internal_repo(&self) -> &Arc<dyn InternalRepository> {
        &self.internal_repo
    }

    /// Handles pagination with external repository fallback when internal
    /// results are exhausted.
    ///
    /// * `query`: Query string for the external API.
    /// * `total_internal`: Total number of internal matches cached.
    /// * `items`: Already-resolved internal items for the current page.
    /// * `exclude`: External IDs to exclude (from cached internal UUIDs).
    ///
    /// Returns (items, total_count).
    pub async fn paginate_with_fallback(
        &self,
        query: &str,
        total_internal: usize,
        mut items: Vec<GameResultItem>,
        limit: usize,
        offset: usize,
        exclude: &[u64],
    ) -> Result<(Vec<GameResultItem>, usize), PaginationError> {
        // ─── If we have enough internal items for this page, count & return ───
        if items.len() >= limit || offset + limit <= total_internal {
            let ext_count = self.external_repo.count(query, exclude).await?;
            let total_count = total_internal + ext_count;
            return Ok((items, total_count));
        }

        // ─── Internal items exhausted. Fall back to external repository ───
        let mut current_ext_offset = if offset >= total_internal {
            offset - total_internal
        } else {
            0
        };

        const BATCH_SIZE: usize = 50;
        let mut external_repo_retries = 10;

        while items.len() < limit && external_repo_retries > 0 {
            let batch = self
                .external_repo
                .search(query, BATCH_SIZE, current_ext_offset, exclude)
                .await?;

            if batch.is_empty() {
                break;
            }

            let remaining_needed = limit - items.len();
            let take_amount = std::cmp::min(remaining_needed, batch.len());
            items.extend(batch.into_iter().take(take_amount));

            current_ext_offset += BATCH_SIZE;
            external_repo_retries -= 1;
        }

        // ─── Count the totals ───
        let ext_count = self.external_repo.count(query, exclude).await?;
        let total_count = total_internal + ext_count;

        Ok((items, total_count))
    }
}