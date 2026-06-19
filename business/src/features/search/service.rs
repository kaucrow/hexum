use async_trait::async_trait;
use uuid::Uuid;

use crate::prelude::*;
use super::*;

#[derive(Clone)]
pub struct Service {
    internal_repo: Arc<dyn InternalRepository>,
    external_repo: Arc<dyn ExternalRepository>,
    cache_repo: Arc<dyn CacheRepository>,
}

impl Service {
    pub fn new(
        internal_repo: Arc<dyn InternalRepository>,
        external_repo: Arc<dyn ExternalRepository>,
        cache_repo: Arc<dyn CacheRepository>,
    ) -> Self {
        Self { internal_repo, external_repo, cache_repo }
    }

    /// Handles pagination with external repository fallback when internal results are exhausted.
    async fn paginate_with_fallback(
        &self,
        query: Option<&str>,
        total_internal: usize,
        mut items: Vec<GameSearchResultItem>,
        limit: usize,
        offset: usize,
        exclude: &[u64],
    ) -> Result<(Vec<GameSearchResultItem>, usize), UseCaseError> {
        // ─── If we have enough internal items for this page, count & return ───
        if items.len() >= limit || offset + limit <= total_internal {
            let ext_count = if let Some(q) = query {
                self.external_repo.count_search_results(q, exclude).await?
            } else {
                0
            };

            let total_count = total_internal + ext_count;
            return Ok((items, total_count));
        }

        // ─── Internal items exhausted. Fall back to external repository ───
        let q = query.unwrap_or("");

        let mut current_ext_offset = if offset >= total_internal {
            offset - total_internal
        } else {
            0   // If this is the first page on the boundary
        };

        const BATCH_SIZE: usize = 50;
        let mut external_repo_retries = 10;

        while items.len() < limit && external_repo_retries > 0 {
            let remaining_needed = limit - items.len();

            let batch = self
                .external_repo
                .search_for_game(q, BATCH_SIZE, current_ext_offset, exclude)
                .await?;

            if batch.is_empty() {
                break;
            }

            let take_amount = std::cmp::min(remaining_needed, batch.len());
            items.extend(batch.into_iter().take(take_amount));

            current_ext_offset += BATCH_SIZE;
            external_repo_retries -= 1;
        }

        // ─── Count the totals ───
        let ext_count = self
            .external_repo
            .count_search_results(q, exclude)
            .await?;

        let total_count = total_internal + ext_count;

        Ok((items, total_count))
    }
}

#[async_trait]
impl UseCase for Service {
    async fn search_for_game(
        &self,
        query: Option<&str>,
        search_id: Option<Uuid>,
        limit: usize,
        offset: usize,
    ) -> Result<SearchSession, UseCaseError> {
        // ─── Try existing search session via search_id ───
        if let Some(search_id) = search_id {
            let search_id_str = search_id.to_string();

            if let Some(page) = self
                .cache_repo
                .get_search_result_ids(&search_id_str, limit, offset)
                .await?
            {
                let total_internal = page.total_items;

                // Return an empty page if there are no internal item results & there's no query
                if total_internal == 0 && query.is_none() {
                    return Ok(SearchSession {
                        search_id,
                        result: SearchResult { items: vec![], total_count: 0 },
                    });
                }

                // Fetch page items and the full cached ID list concurrently
                let (items, cached_internal_ids_res) = tokio::try_join!(
                    async { self.internal_repo.get_games_by_ids(&page.ids).await.map_err(UseCaseError::from) },
                    async { self.cache_repo.get_all_search_result_ids(&search_id_str).await.map_err(UseCaseError::from) }
                )?;

                let cached_internal_ids = cached_internal_ids_res.unwrap_or_default();

                let exclude = self
                    .internal_repo
                    .get_external_ids_by_internal_ids(&cached_internal_ids)
                    .await?;

                let (items, total_count) = self
                    .paginate_with_fallback(
                        query,
                        total_internal,
                        items,
                        limit,
                        offset,
                        &exclude,
                    )
                    .await?;

                return Ok(SearchSession {
                    search_id,
                    result: SearchResult { items, total_count },
                });
            }
        }

        // ─── Fresh search ───
        let new_search_id = Uuid::new_v4();
        let q = match query {
            Some(q) => q,
            None => {
                return Ok(SearchSession {
                    search_id: new_search_id,
                    result: SearchResult { items: vec![], total_count: 0 },
                });
            }
        };

        // Get all internal matches' IDs
        let internal_matches_ids = self
            .internal_repo
            .get_game_ids_by_query(q)
            .await?;

        let total_internal = internal_matches_ids.len();

        // ─── Cache all internal repository matches ───
        let cache_repo = Arc::clone(&self.cache_repo);
        let cache_key = new_search_id.to_string();
        let cache_ids = internal_matches_ids.clone(); // Cloned to move safely into the static thread block

        tokio::spawn(async move {
            if let Err(e) = cache_repo.cache_search_result_ids(&cache_key, &cache_ids).await {
                error!("Search cache saving failed: {:?}", e);
            }
        });

        // Paginate the matches
        let page_uuids: Vec<Uuid> = internal_matches_ids
            .iter()
            .copied()
            .skip(offset)
            .take(limit)
            .collect();

        // Get the games & the external IDs concurrently
        let (items, exclude) = tokio::try_join!(
            async { self.internal_repo.get_games_by_ids(&page_uuids).await.map_err(UseCaseError::from) },
            async { self.internal_repo.get_external_ids_by_internal_ids(&internal_matches_ids).await.map_err(UseCaseError::from) }
        )?;

        let (items, total_count) = self
            .paginate_with_fallback(
                Some(q),
                total_internal,
                items,
                limit,
                offset,
                &exclude,
            )
            .await?;

        Ok(SearchSession {
            search_id: new_search_id,
            result: SearchResult { items, total_count },
        })
    }
}