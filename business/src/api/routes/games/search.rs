use uuid::Uuid;

use crate::{
    prelude::*,
    api::*,
    features::search,
};
use super::dtos::*;

#[utoipa::path(
    get,
    path = "/game/search",
    description = "Searches for games.",
    params(GameSearchQueryParams),
    responses(
        (status = 200, description = "Search results", body = GameSearchResponse),
        (status = 400, description = "Validation Error"),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["Games"]
)]
pub async fn search(
    State(search_service): State<Arc<dyn search::UseCase>>,
    ValidatedQuery(params): ValidatedQuery<GameSearchQueryParams>,
) -> Result<Json<GameSearchResponse>, ApiError> {
    info!(
        "Got game search request with query '{:?}', platforms '{:?}', pagination ID '{:?}'",
        params.q,
        params.platforms,
        params.pagination_id,
    );

    // ─── Build GameSearch from query params ───
    let platforms = params.platforms
    .as_deref()
    .map(|platforms_raw_str| {
        platforms_raw_str.split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(Uuid::parse_str)
            .collect::<Result<Vec<Uuid>, _>>()
    })
    .transpose()
    .map_err(|_| ApiError::BadRequest("Invalid platforms format.".to_string()))?
    .filter(|ids| !ids.is_empty());

    let game_search = search::GameSearch {
        query: params.q.clone(),
        platforms,
    };

    // ─── Search + paginate ───
    let result = search_service
        .search_for_game(game_search, params.pagination_id, params.limit, params.offset)
        .await?;

    let games: Vec<GameSearchResultItemResponse> = result
        .items
        .into_iter()
        .map(|item| GameSearchResultItemResponse {
            id: item.id,
            external_id: item.external_id,
            name: item.name,
        })
        .collect();

    Ok(Json(GameSearchResponse {
        games,
        meta: GameSearchMeta {
            total_count: result.total_count,
            pagination_id: result.pagination_id,
        },
    }))
}

impl From<search::UseCaseError> for ApiError {
    fn from(e: search::UseCaseError) -> Self {
        match e {
            search::UseCaseError::VideogameApi(e) => {
                error!("Videogame API error in search: {e}");
                ApiError::Internal("An internal error occurred".to_string())
            },
            search::UseCaseError::EmptySearch => {
                warn!("Got a search request with no search fields.");
                ApiError::BadRequest("In order to search for a game, at least one of the following must be provided: 'query', 'platformIds'.".to_string())
            },
            search::UseCaseError::Internal(e) => {
                error!("An internal error occurred in search: {e}");
                ApiError::Internal("An internal error occurred".to_string())
            }
        }
    }
}