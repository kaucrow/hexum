use crate::prelude::*;
use crate::features::base::{
    PaginatedQuery, PaginatorSession,
};
use crate::features::base::pagination::CacheRepository;
use crate::features::videogame_api;

use super::*;

#[derive(Clone)]
pub struct Service {
    query: PaginatedQuery,
    /// Combined internal repository for search-specific queries & pagination.
    internal_repo: Arc<dyn PaginatedInternalRepository>,
    /// IGDB adapter.
    igdb: IgdbAdapter,
}

impl Service {
    pub fn new(
        internal_repo: Arc<dyn PaginatedInternalRepository>,
        pagination_cache_repo: Arc<dyn CacheRepository>,
        igdb: videogame_api::IgdbAdapter,
    ) -> Self {
        let paginator = PaginatorSession::new(pagination_cache_repo, "search");
        let query = PaginatedQuery::new(paginator);
        Self { query, internal_repo, igdb: IgdbAdapter::new(igdb) }
    }

    /// Resolve UUIDs to items, build exclusion list, then call [`PaginatedQuery::paginate_with_fallback`].
    async fn paginate(
        &self,
        search: &GameSearch,
        pagination_id: Uuid,
        internal_count: usize,
        limit: usize,
        offset: usize,
    ) -> Result<SearchResult, UseCaseError> {
        let session = self.query.session();

        // Build exclusion list (skip cache if no internal results)
        let (exclude, page_ids) = if internal_count == 0 {
            (vec![], vec![])
        } else {
            let cached_ids = session.get_all_ids(&pagination_id).await?.unwrap_or_default();
            let exclude = self.internal_repo.get_external_ids_by_internal_ids(&cached_ids).await?;
            let (page_ids, _) = self.query.get_page_uuids(&pagination_id, limit, offset).await?;
            (exclude, page_ids)
        };

        // Resolve UUIDs to GameResultItems
        let items = self.internal_repo.get_games_by_ids(&page_ids).await?;

        // Resolve internal platform UUIDs to external platform IDs
        let ext_platforms = if let Some(platform_uuids) = &search.platforms {
            Some(self.internal_repo.get_external_platform_ids_by_internal_ids(platform_uuids).await?)
        } else {
            None
        };

        // Build PaginationGameSearch
        let pagination_search = PaginationGameSearch {
            query: search.query.clone(),
            ext_platforms,
        };

        // Perform pagination
        let (items, total_count) =
            self.query.paginate_with_fallback(
                &self.igdb,
                &pagination_search,
                internal_count,
                items,
                limit,
                offset,
                &exclude,
            ).await?;

        Ok(SearchResult { items, total_count, pagination_id })
    }
}

#[async_trait]
impl UseCase for Service {
    async fn search_for_game(
        &self,
        search: GameSearch,
        search_id: Option<Uuid>,
        limit: usize,
        offset: usize,
    ) -> Result<SearchResult, UseCaseError> {
        // If the search is empty, return an error
        if !(search.query.is_some() || search.platforms.is_some()) {
            return Err(UseCaseError::EmptySearch);
        }

        // ─── Existing session. Paginate directly ───
        if let Some(sid) = search_id {
            if let Some(page) = self.query.session().get_page(&sid, 1, 0).await? {
                return self.paginate(&search, sid, page.total_items, limit, offset).await;
            }
        }

        // ─── Fresh search ───
        let new_pagination_id = Uuid::new_v4();

        let internal_ids = self.internal_repo.get_game_ids_by_criteria(&search).await?;
        let internal_count = internal_ids.len();

        // Cache IDs
        let session = self.query.session().clone();
        let cache_key = new_pagination_id;
        let cache_ids = internal_ids.clone();
        tokio::spawn(async move {
            if let Err(e) = session.cache_pagination(&cache_key, &cache_ids).await {
                error!("Search cache saving failed: {:?}", e);
            }
        });

        // Paginate the fresh results
        self.paginate(&search, new_pagination_id, internal_count, limit, offset).await
    }
}