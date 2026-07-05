use crate::{
    prelude::*,
    api::*,
    features::review,
};
use super::dtos::*;

#[utoipa::path(
    get,
    path = "/game/{id}/reviews",
    description = "Get all user and critic reviews for a specific game.",
    params(
        ("id" = String, Path, description = "Game internal UUID"),
    ),
    responses(
        (status = 200, description = "List of reviews", body = GetGameReviewsResponse),
        (status = 400, description = "Invalid game ID format"),
        (status = 500, description = "Internal Server Error"),
    ),
    tags = ["Reviews"]
)]
pub async fn get_game_reviews(
    State(review_service): State<Arc<dyn review::UseCase>>,
    ValidatedPath(path): ValidatedPath<GameIdPath>,
) -> Result<Json<GetGameReviewsResponse>, ApiError> {
    let game_id = Uuid::parse_str(&path.id)
        .map_err(|_| ApiError::BadRequest("Invalid game ID format.".to_string()))?;

    let reviews = review_service
        .get_game_reviews(&game_id)
        .await
        .map_err(|e| match e {
            review::UseCaseError::Internal(msg) => {
                error!("Get game reviews error: {msg}");
                ApiError::Internal("An internal error occurred".to_string())
            }
            _ => ApiError::Internal("An internal error occurred".to_string()),
        })?;

    let items: Vec<ReviewResponse> = reviews
        .into_iter()
        .map(|r| ReviewResponse {
            id: r.id.to_string(),
            user_id: r.user_id.to_string(),
            game_id: r.game_id.to_string(),
            review_type: r.review_type.to_string(),
            rating: r.rating,
            title: r.title,
            content: r.content,
            created_at: r.created_at.to_rfc3339(),
            updated_at: r.updated_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(GetGameReviewsResponse { reviews: items }))
}