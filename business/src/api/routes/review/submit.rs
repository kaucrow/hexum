use crate::{
    prelude::*,
    api::*,
    features::review,
};
use super::dtos::*;

#[utoipa::path(
    post,
    path = "/game/{id}/review",
    description = "Submit or update a review for a game. If the user already has a review for this game, it will be updated.",
    params(
        ("id" = String, Path, description = "Game internal UUID"),
    ),
    request_body = SubmitReviewRequest,
    responses(
        (status = 200, description = "Review submitted/updated", body = SubmitReviewResponse),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation Error"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Reviews"]
)]
pub async fn submit_review(
    auth: AuthenticatedUser,
    State(review_service): State<Arc<dyn review::UseCase>>,
    ValidatedPath(path): ValidatedPath<GameIdPath>,
    ValidatedJson(payload): ValidatedJson<SubmitReviewRequest>,
) -> Result<Json<SubmitReviewResponse>, ApiError> {
    let game_id = Uuid::parse_str(&path.id)
        .map_err(|_| ApiError::BadRequest("Invalid game ID format.".to_string()))?;

    info!(
        "User '{}' is submitting a review for game '{}' with rating {}",
        auth.user_id, game_id, payload.rating
    );

    let saved = review_service
        .submit_review(
            &auth.user_id,
            &auth.roles,
            &game_id,
            payload.rating,
            &payload.title,
            &payload.content,
        )
        .await
        .map_err(|e| match e {
            review::UseCaseError::InvalidRating => {
                ApiError::BadRequest("Rating must be between 0 and 100.".to_string())
            }
            review::UseCaseError::GameNotFound => {
                ApiError::BadRequest("Game not found.".to_string())
            }
            review::UseCaseError::Internal(msg) => {
                error!("Review submit error: {msg}");
                ApiError::Internal("An internal error occurred".to_string())
            }
        })?;

    Ok(Json(SubmitReviewResponse {
        id: saved.id.to_string(),
        user_id: saved.user_id.to_string(),
        game_id: saved.game_id.to_string(),
        review_type: saved.review_type.to_string(),
        rating: saved.rating,
        title: saved.title,
        content: saved.content,
        created_at: saved.created_at.to_rfc3339(),
        updated_at: saved.updated_at.to_rfc3339(),
    }))
}