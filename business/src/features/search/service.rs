use std::sync::Arc;
use async_trait::async_trait;

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
}

#[async_trait]
impl UseCase for Service {
    async fn search_for_game(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> Result<SearchResult, UseCaseError> {
        // Try to get the exclusion IDs from cache
        let exclude_ids = match self.cache_repo.get_search_exclusion_ids(query).await? {
            Some(ids) => ids, // Cache hit
            None => {
                // Cache miss: fetch all matched internal IDs from DB and cache them
                let ids = self.internal_repo.get_external_ids_for_search(query).await?;
                self.cache_repo.cache_search_exclusion_ids(query, &ids).await?;
                ids
            }
        };

        // The length of our exclusion list equals the total internal matches
        let total_internal = exclude_ids.len();

        // Count total external matches (respecting exclusion list)
        let total_external = self.external_repo
            .count_search_results(query, &exclude_ids)
            .await?;

        let total_count = total_internal + total_external;

        let mut items = Vec::new();

        // Fetch internal results if the requested offset falls within internal range
        if offset < total_internal {
            let mut internal_games = self.internal_repo
                .search_for_game(query, limit, offset)
                .await?;
            items.append(&mut internal_games);
        }

        // Fetch from external API to fill any remaining slots up to `limit`
        if items.len() < limit {
            let remaining_limit = limit - items.len();

            let external_offset = if offset >= total_internal {
                offset - total_internal // Paginating purely within external results
            } else {
                0 // Crossing the boundary on this page
            };

            let mut external_games = self.external_repo
                .search_for_game(query, remaining_limit, external_offset, &exclude_ids)
                .await?;

            items.append(&mut external_games);
        }

        Ok(SearchResult { items, total_count })
    }
}