use uuid::Uuid;

use crate::{
    prelude::*,
    api::*,
};

#[derive(Deserialize, IntoParams, Validate)]
#[into_params(parameter_in = Query)]
pub struct GameSearchQueryParams {
    /// Search query string (required for initial search, optional for pagination).
    #[serde(default)]
    pub q: Option<String>,

    /// Search session ID from a previous search response (for paginated requests).
    #[serde(default)]
    pub search_id: Option<Uuid>,

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
pub struct GameSearchResultItemDto {
    pub id: Uuid,
    pub external_id: Option<i64>,
    pub name: String,
}

#[derive(Serialize, ToSchema)]
pub struct GameSearchMeta {
    /// Total number of games matching the query.
    pub total_count: usize,
    /// The session ID for this search. Pass back for paginated requests.
    pub search_id: Uuid,
}

#[derive(Serialize, ToSchema)]
pub struct GameSearchResponse {
    pub games: Vec<GameSearchResultItemDto>,
    pub meta: GameSearchMeta,
}