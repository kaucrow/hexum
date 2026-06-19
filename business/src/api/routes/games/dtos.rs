use uuid::Uuid;
use utoipa::{IntoParams, ToSchema};

use crate::prelude::*;

fn default_limit() -> usize { 10 }

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GameSearchQueryParams {
    /// Search query string.
    pub q: String,

    /// Maximum results to return.
    #[serde(default = "default_limit")]
    pub limit: usize,

    /// Results to skip.
    #[serde(default)]
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
}

#[derive(Serialize, ToSchema)]
pub struct GameSearchResponse {
    pub games: Vec<GameSearchResultItemDto>,
    pub meta: GameSearchMeta,
}