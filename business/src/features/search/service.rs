use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use crate::prelude::*;
use crate::features::base::{
    PaginatedQuery, PaginatorSession,
};
use crate::features::base::pagination::{
    CacheRepository, ExternalRepository, InternalRepository,
};

use super::*;

#[derive(Clone)]
pub struct Service {
    query: PaginatedQuery,
}

impl Service {
    pub fn new(
        internal_repo: Arc<dyn InternalRepository>,
        external_repo: Arc<dyn ExternalRepository>,
        pagination_cache_repo: Arc<dyn CacheRepository>,
    ) -> Self {
        let paginator = PaginatorSession::new(pagination_cache_repo, "search");
        let query = PaginatedQuery::new(paginator, internal_repo, external_repo);
        Self { query }
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
            if let Some(page) = self
                .query
                .session()
                .get_page(&search_id, limit, offset)
                .await?
            {
                let total_internal = page.total_items;

                // Return an empty page if there are no internal results & no query
                if total_internal == 0 && query.is_none() {
                    return Ok(SearchSession {
                        search_id,
                        result: SearchResult { items: vec![], total_count: 0 },
                    });
                }

                // Fetch page items and the full cached ID list concurrently
                let internal_repo = self.query.internal_repo();
                let paginator_session = self.query.session();

                let (items, cached_internal_ids_res) = tokio::try_join!(
                    async { internal_repo.get_games_by_ids(&page.ids).await.map_err(UseCaseError::from) },
                    async { paginator_session.get_all_ids(&search_id).await.map_err(UseCaseError::from) }
                )?;

                let cached_internal_ids = cached_internal_ids_res.unwrap_or_default();

                let exclude = internal_repo
                    .get_external_ids_by_internal_ids(&cached_internal_ids)
                    .await?;

                let q = query.unwrap_or("");

                let (items, total_count) = self
                    .query
                    .paginate_with_fallback(
                        q,
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
            .query
            .internal_repo()
            .get_game_ids_by_query(q)
            .await?;

        let total_internal = internal_matches_ids.len();

        // Cache all internal repository matches in the background
        let paginator_session = self.query.session().clone();
        let cache_key = new_search_id;
        let cache_ids = internal_matches_ids.clone();

        tokio::spawn(async move {
            if let Err(e) = paginator_session.cache_session(&cache_key, &cache_ids).await {
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

        let internal_repo = self.query.internal_repo();

        // Get the games & the external IDs concurrently
        let (items, exclude) = tokio::try_join!(
            async { internal_repo.get_games_by_ids(&page_uuids).await.map_err(UseCaseError::from) },
            async { internal_repo.get_external_ids_by_internal_ids(&internal_matches_ids).await.map_err(UseCaseError::from) }
        )?;

        let (items, total_count) = self
            .query
            .paginate_with_fallback(
                q,
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