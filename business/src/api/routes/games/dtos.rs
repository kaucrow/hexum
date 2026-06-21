use uuid::Uuid;

use crate::{
    prelude::*,
    api::*,
};

#[derive(Deserialize, IntoParams, Validate)]
#[into_params(parameter_in = Query)]
#[serde(rename_all = "camelCase")]
pub struct GameSearchQueryParams {
    /// Search query string.
    pub q: Option<String>,

    /// Comma-separated platform UUIDs to filter by.
    pub platforms: Option<String>,

    /// Pagination session ID from a previous search response (for paginated requests).
    pub pagination_id: Option<Uuid>,

    /// Maximum results to return.
    #[param(example = 4)]
    #[validate(range(min = 0))]
    pub limit: usize,

    /// Results to skip.
    #[param(example = 0)]
    #[validate(range(min = 0))]
    pub offset: usize,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GameSearchResultItemResponse {
    /// The internal game ID. Is null if it doesn't yet exist in the internal repository.
    pub id: Option<Uuid>,

    /// The external game ID.
    pub external_id: u64,

    /// The game's name.
    pub name: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GameSearchMeta {
    /// Total number of games matching the query.
    pub total_count: usize,
    /// The ID for this search. Pass back for paginated requests.
    pub pagination_id: Uuid,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GameSearchResponse {
    pub games: Vec<GameSearchResultItemResponse>,
    pub meta: GameSearchMeta,
}