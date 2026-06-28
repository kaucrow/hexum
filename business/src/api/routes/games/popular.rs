use crate::{
    prelude::*,
    api::*,
    features::game,
};
use super::dtos::*;

#[utoipa::path(
    get,
    path = "/game/popular",
    description = "Gets the most popular games. Tries the external API first (sorted by IGDB popularity); falls back to the internal database sorted by view count if the external API is unreachable.",
    params(PopularGamesQueryParams),
    responses(
        (status = 200, description = "Popular games list", body = PopularGamesResponse),
        (status = 400, description = "Validation Error"),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["Games"]
)]
pub async fn popular_games(
    State(game_service): State<Arc<dyn game::UseCase>>,
    ValidatedQuery(params): ValidatedQuery<PopularGamesQueryParams>,
) -> Result<Json<PopularGamesResponse>, ApiError> {
    info!(
        "Got request for popular games (limit: {}, offset: {})",
        params.limit, params.offset,
    );

    let result = game_service
        .get_popular_games(params.limit, params.offset)
        .await?;

    let games: Vec<PopularGameItemResponse> = result
        .games
        .into_iter()
        .map(|item| PopularGameItemResponse {
            id: item.id,
            external_id: item.external_id,
            name: item.name,
        })
        .collect();

    Ok(Json(PopularGamesResponse {
        games,
        meta: PopularGamesMeta {
            total_count: result.total_count,
            from_internal: result.from_internal,
        },
    }))
}