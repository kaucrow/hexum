use crate::{
    prelude::*,
    api::*,
    features::critic,
};
use super::dtos::*;

#[utoipa::path(
    post,
    path = "/critic/apply",
    description = "A BasicUser applies to become a Critic.",
    responses(
        (status = 201, description = "Application submitted", body = ApplyForCriticResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - must be a BasicUser"),
        (status = 409, description = "Already applied"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Critic"]
)]
pub async fn apply_for_critic(
    RequireRole(auth, _): RequireRole<role::BasicUser>,
    State(critic_service): State<Arc<dyn critic::UseCase>>,
) -> Result<(StatusCode, Json<ApplyForCriticResponse>), ApiError> {
    info!("User '{}' is applying for Critic role", auth.user_id);

    let application = critic_service
        .apply_for_critic(&auth.user_id)
        .await
        .map_err(|e| match e {
            critic::UseCaseError::AlreadyApplied => {
                ApiError::Conflict("You have already applied for the Critic role.".to_string())
            }
            critic::UseCaseError::Internal(msg) => {
                error!("Critic apply error: {msg}");
                ApiError::Internal("An internal error occurred".to_string())
            }
            _ => {
                error!("Critic apply error: {e}");
                ApiError::Internal("An internal error occurred".to_string())
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(ApplyForCriticResponse {
            message: "Your application for the Critic role has been submitted.".to_string(),
            application_id: application.id.to_string(),
        }),
    ))
}