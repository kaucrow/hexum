use uuid::Uuid;
use crate::features::base::pagination::GameResultItem;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub items: Vec<GameResultItem>,
    /// Total number of games matching the query from both internal DB
    /// and external API (respecting the exclusion list).
    pub total_count: usize,
}

/// A search session that ties a search_id to its results.
/// The search_id must be passed back by the client for paginated requests.
#[derive(Debug, Clone)]
pub struct SearchSession {
    pub search_id: Uuid,
    pub result: SearchResult,
}
