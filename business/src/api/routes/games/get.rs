use crate::{
    prelude::*,
    api::*,
    features::game,
};
use super::dtos::*;

#[utoipa::path(
    get,
    path = "/game",
    description = "Gets a game by internal UUID or external IGDB ID. Exactly one of 'id' or 'externalId' must be provided.",
    params(GameQueryParams),
    responses(
        (status = 200, description = "Game data with platforms", body = GameResponse),
        (status = 400, description = "Validation Error"),
        (status = 404, description = "Game not found"),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["Games"]
)]
pub async fn get_game(
    State(game_service): State<Arc<dyn game::UseCase>>,
    ValidatedQuery(params): ValidatedQuery<GameQueryParams>,
) -> Result<Json<GameResponse>, ApiError> {
    // Validate exactly one of id or external_id is provided
    let game = match (params.id, params.external_id) {
        (Some(id), None) => {
            info!("Got request to get game by internal ID: {}", id);

            game_service
                .get_internal_game(&id)
                .await?
                .ok_or_else(|| ApiError::BadRequest("Game not found.".to_string()))?
        }
        (None, Some(external_id)) => {
            info!("Got request to get game by external ID: {}", external_id);

            let game = match game_service
                .get_external_game(external_id)
                .await?
            {
                Some(game) => game,
                None => return Err(ApiError::BadRequest("Game not found.".to_string())),
            };
            game
        }
        _ => {
            return Err(ApiError::BadRequest(
                "Provide exactly one of either 'id' or 'externalId'.".to_string(),
            ));
        }
    };

    let platforms: Vec<GamePlatformResponse> = game
        .platforms
        .into_iter()
        .map(|p| GamePlatformResponse {
            id: p.id,
            external_id: p.external_id,
            name: p.name,
            generation: p.generation,
        })
        .collect();

    let genres: Vec<GenreResponse> = game
        .genres
        .into_iter()
        .map(|g| GenreResponse {
            id: g.id,
            external_id: g.external_id,
            name: g.name,
        })
        .collect();

    let companies: Vec<GameCompanyResponse> = game
        .companies
        .into_iter()
        .map(|gc| GameCompanyResponse {
            id: gc.company.id,
            external_id: gc.company.external_id,
            name: gc.company.name,
            role: match gc.role {
                game::CompanyRole::Developer => "developer".to_string(),
                game::CompanyRole::Publisher => "publisher".to_string(),
            },
        })
        .collect();

    Ok(Json(GameResponse {
        id: game.id,
        external_id: game.external_id,
        name: game.name,
        first_release_date: game.first_release_date,
        cover_url: game.cover_url,
        rating: game.rating,
        critic_rating: game.critic_rating,
        total_rating_count: game.total_rating_count,
        summary: game.summary,
        platforms,
        genres,
        companies,
    }))
}

impl From<game::UseCaseError> for ApiError {
    fn from(e: game::UseCaseError) -> Self {
        match e {
            game::UseCaseError::VideogameApi(msg) => {
                error!("Videogame API error in game: {msg}");
                ApiError::Internal("An internal error occurred".to_string())
            }
            game::UseCaseError::Internal(msg) => {
                error!("An internal error occurred in game: {msg}");
                ApiError::Internal("An internal error occurred".to_string())
            }
        }
    }
}