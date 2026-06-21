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
/// `id` is `Some` if the game exists in the internal repository and `None` if it
/// only exists in the external repository.
#[derive(Debug, Clone)]
pub struct GameResultItem {
    pub id: Option<Uuid>,
    pub external_id: u64,
    pub name: String,
}