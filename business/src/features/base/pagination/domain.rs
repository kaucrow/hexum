use uuid::Uuid;

/// A game result item returned by paginated queries.
/// Used by both internal and external repositories.
#[derive(Debug, Clone)]
pub struct GameResultItem {
    pub id: Uuid,
    pub external_id: Option<i64>,
    pub name: String,
}