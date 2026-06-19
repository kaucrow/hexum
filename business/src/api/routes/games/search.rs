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
        (status = 429, description = "Validation Error"),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["Games"]
)]
pub async fn search(
    State(search_service): State<Arc<dyn search::UseCase>>,
    ValidatedQuery(params): ValidatedQuery<GameSearchQueryParams>,
) -> Result<Json<GameSearchResponse>, ApiError> {
    info!(
        "Got game search request with query '{:?}' & search ID '{:?}'",
        params.q,
        params.search_id,
    );

    let session = search_service
        .search_for_game(
            params.q.as_deref(),
            params.search_id,
            params.limit,
            params.offset,
        )
        .await?;

    let games: Vec<GameSearchResultItemDto> = session
        .result
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
            total_count: session.result.total_count,
            search_id: session.search_id,
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