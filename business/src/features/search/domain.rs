use crate::prelude::*;

/// Paginated search result for a single page.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub items: Vec<GameResultItem>,
    /// Total number of games matching the query from both internal DB
    /// and external API (respecting the exclusion list).
    pub total_count: usize,
    /// The pagination ID for this search. Pass back for paginated requests.
    pub pagination_id: Uuid,
}

/// A game result item returned by paginated queries.
/// Used by both internal and external repositories.
#[derive(Debug, Clone)]
pub struct GameResultItem {
    pub id: Uuid,
    pub external_id: Option<u64>,
    pub name: String,
}