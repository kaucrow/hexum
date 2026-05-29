use std::sync::Arc;

use uuid::Uuid;
use async_trait::async_trait;

use crate::prelude::*;
use super::*;

#[derive(Clone)]
pub struct Service {
    local_repo: Arc<dyn LocalRepository>,
    cache_repo: Arc<dyn CacheRepository>,
}

impl Service {
    pub fn new(
        local_repo: Arc<dyn LocalRepository>,
        cache_repo: Arc<dyn CacheRepository>,
    ) -> Self {
        Self { local_repo, cache_repo }
    }
}

#[async_trait]
impl UseCase for Service {
    async fn search_recipe(&self,
        query: &str,
        page: usize,
        limit: usize,
        search_id: Option<Uuid>,
    ) -> Result<SearchResultsPage, UseCaseError>
    {
        let safe_page = if page == 0 { 1 } else { page };
        let offset = (safe_page - 1) * limit;

        let (search_id, matching_ids) = match search_id {
            // ─── Brand New Search ───
            None => {
                let new_id = Uuid::new_v4();
                let search_cache_key = format!("search:{}", new_id);

                // Get all matching recipe IDs, sorted by similarity to the query
                let matching_ids = self.local_repo.get_recipe_search_ids(query).await?;

                if !matching_ids.is_empty() {
                    // Set the recipe search cache in Redis
                    self.cache_repo.set_recipe_ids(&search_cache_key, &matching_ids, 300).await?;   // 5 minutes
                }

                (new_id, matching_ids)
            }

            // ─── Fetching an existing page jump ───
            Some(existing_id) => {
                let search_cache_key = format!("search:{}", existing_id);

                match self.cache_repo.get_recipe_ids(&search_cache_key).await? {
                    Some(ids) => (existing_id, ids),
                    None => {
                        // The search cache expired, set it again
                        let ids = self.local_repo.get_recipe_search_ids(query).await?;
                        if !ids.is_empty() {
                            self.cache_repo.set_recipe_ids(&search_cache_key, &ids, 300).await?;
                        }
                        (existing_id, ids)
                    }
                }
            }
        };

        if matching_ids.is_empty() {
            return Ok(SearchResultsPage { items: Vec::new(), total_items: 0, search_id: Uuid::new_v4() });
        }

        let total_items = matching_ids.len();

        // Slice the matching_ids vector based on offset and limit
        let page_ids: Vec<Uuid> = matching_ids
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();

        // Get the actual recipe search result data
        let items = self.local_repo.get_recipe_search_data_by_ids(&page_ids).await?;

        Ok(SearchResultsPage {
            items,
            total_items,
            search_id,
        })
    }
}

impl From<LocalRepositoryError> for UseCaseError {
    fn from(e: LocalRepositoryError) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<CacheRepositoryError> for UseCaseError {
    fn from(e: CacheRepositoryError) -> Self {
        Self::Internal(e.to_string())
    }
}