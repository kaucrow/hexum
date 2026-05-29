use std::sync::Arc;

use async_trait::async_trait;
use textdistance::nstr::jaro_winkler;

use super::*;

#[derive(Clone)]
pub struct Service {
    local_repo: Arc<dyn LocalRepository>,
    external_repo: Arc<dyn ExternalRepository>,
    cache_repo: Arc<dyn CacheRepository>,
}

impl Service {
    pub fn new(
        local_repo: Arc<dyn LocalRepository>,
        external_repo: Arc<dyn ExternalRepository>,
        cache_repo: Arc<dyn CacheRepository>,
    ) -> Self {
        Self {
            local_repo,
            external_repo,
            cache_repo,
        }
    }
}

#[async_trait]
impl UseCase for Service {
    async fn search_recipe_by_name(&self, name: &str, page: usize) -> Result<input::RecipeSearchResult, UseCaseError> {
        let index_cache_key = format!("search:name:{}", name);

        // ─── Get API search results ───
        let api_search_results: Vec<domain::RecipeSearchResult>;

        // ─── Check the cache for the search results ───
        if let Some(cached_search_results) = self.cache_repo.get_search_results(&index_cache_key)
            .await
            .ok()
            .flatten()
        {
            api_search_results = cached_search_results;
        } else {
            // Cache miss: Hit the external API
            api_search_results = self.external_repo.get_recipe_search_results(name).await?;

            // Set index & individual recipes concurrently.
            // Save the search results (valid for 1 hour).
            let _ = self.cache_repo.set_search_results(&index_cache_key, &api_search_results, 3600).await;
        }

        // ─── Get DB candidates ───
        let db_search_results = self.local_repo.get_recipe_search_results(name).await?;

        // ─── Sort the candidates by similarity to the search ───
        let mut scored_candidates: Vec<(domain::RecipeSearchResult, f64)> = api_search_results
            .into_iter()
            .chain(db_search_results.into_iter())
            .map(|candidate| {
                // Lowercase the candidate name for accurate comparison
                let candidate_lower = candidate.name.to_lowercase();

                // Jaro-Winkler returns an f64 between 0.0 and 1.0.
                let score = jaro_winkler(&candidate_lower, &name);

                (candidate, score)
            })
            .collect();

        scored_candidates
            .sort_by(
                |a, b|
                a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
            );

        let total_items = scored_candidates.len();

        // ─── Apply pagination ───
        let page_size = 10;
        let start_index = page * page_size;

        let paginated_results: Vec<domain::RecipeSearchResult> = scored_candidates
            .into_iter()
            .skip(start_index)
            .take(page_size)
            .map(|(candidate, _score)| candidate)
            .collect();

        Ok(input::RecipeSearchResult {
            items: paginated_results,
            total_items,
        })
    }
}

impl From<LocalRepositoryError> for UseCaseError {
    fn from(e: LocalRepositoryError) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<ExternalRepositoryError> for UseCaseError {
    fn from(e: ExternalRepositoryError) -> Self {
        Self::Internal(e.to_string())
    }
}