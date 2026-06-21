use crate::prelude::*;
use super::*;

/// Feature-agnostic pagination session management with external API fallback.
///
/// Works purely at the UUID level. It does not know about feature-specific
/// item types. Features resolve UUIDs to their own item types and pass them
/// to [`paginate_with_fallback`].
#[derive(Clone)]
pub struct PaginatedQuery {
    session: PaginatorSession,
}

impl PaginatedQuery {
    pub fn new(
        session: PaginatorSession,
    ) -> Self {
        Self { session }
    }

    /// Access the session paginator for session lifecycle operations.
    pub fn session(&self) -> &PaginatorSession {
        &self.session
    }

    /// Get a paginated page of UUIDs from a cached session.
    /// Returns the UUID slice and the total number of cached items.
    pub async fn get_page_uuids(
        &self,
        session_id: &Uuid,
        limit: usize,
        offset: usize,
    ) -> Result<(Vec<Uuid>, usize), PaginationError> {
        let page = self
            .session
            .get_page(session_id, limit, offset)
            .await?
            .ok_or_else(|| PaginationError::Cache(
                CacheRepositoryError::Internal("Session expired".to_string()),
            ))?;

        Ok((page.ids, page.total_items))
    }

    /// Handles pagination with external repository fallback when internal
    /// results are exhausted.
    ///
    /// Generic over the item type `T`. The caller resolves UUIDs to items
    /// using their own feature-specific repository and passes them here.
    ///
    /// * `external_repo`: Feature-specific adapter.
    /// * `search`: Feature-specific search criteria passed to `fetch`/`count`.
    /// * `total_internal`: Total number of internal matches cached.
    /// * `items`: Already-resolved internal items for the current page.
    /// * `exclude`: External IDs to exclude (from cached internal UUIDs).
    ///
    /// Returns `(items, total_count)`.
    pub async fn paginate_with_fallback<T: Clone + Send + 'static, S: Clone + Send + 'static>(
        &self,
        external_repo: &dyn ExternalRepository<Item = T, Search = S>,
        search: &S,
        total_internal: usize,
        mut items: Vec<T>,
        limit: usize,
        offset: usize,
        exclude: &[u64],
    ) -> Result<(Vec<T>, usize), PaginationError> {
        // ─── If we have enough internal items for this page, count & return ───
        if items.len() >= limit || offset + limit <= total_internal {
            let ext_count = external_repo.count(search, exclude).await?;
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
            let batch = external_repo
                .fetch(search, BATCH_SIZE, current_ext_offset, exclude)
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
        let ext_count = external_repo.count(search, exclude).await?;
        let total_count = total_internal + ext_count;

        Ok((items, total_count))
    }
}