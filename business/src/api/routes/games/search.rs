use crate::{
    prelude::*,
    api::*,
    features::search,
};
use super::dtos::*;

#[utoipa::path(
    get,
    path = "/games/search",
    description = "Searches for games.",
    params(GameSearchQueryParams),
    responses(
        (status = 200, description = "Search results", body = GameSearchResponse),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["Games"]
)]
pub async fn search(
    State(search_service): State<Arc<dyn search::UseCase>>,
    Query(params): Query<GameSearchQueryParams>,
) -> Result<Json<GameSearchResponse>, ApiError> {
    let result = search_service
        .search_for_game(&params.q, params.limit, params.offset)
        .await?;

    let games: Vec<GameSearchResultItemDto> = result
        .items
        .into_iter()
        .map(|item| GameSearchResultItemDto {
            id: item.id,
            external_id: item.external_id,
            name: item.name,
        })
        .collect();

    Ok(Json(GameSearchResponse {
        games,
        meta: GameSearchMeta {
            total_count: result.total_count,
        },
    }))
}

impl From<search::UseCaseError> for ApiError {
    fn from(e: search::UseCaseError) -> Self {
        match e {
            search::UseCaseError::VideogameApi(e) => {
                error!("Videogame API error in search: {e}");
                ApiError::Internal("An internal error occurred".to_string())
            }
            search::UseCaseError::Internal(e) => {
                error!("An internal error occurred in search: {e}");
                ApiError::Internal("An internal error occurred".to_string())
            }
        }
    }
}