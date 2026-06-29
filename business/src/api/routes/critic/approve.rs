use crate::{
    prelude::*,
    api::*,
    features::critic,
};
use super::dtos::*;

#[utoipa::path(
    post,
    path = "/critic/applications/{id}/approve",
    description = "Admin approves a pending Critic application, granting the Critic role.",
    params(
        ("id" = String, Path, description = "Application ID"),
    ),
    responses(
        (status = 200, description = "Application approved", body = ApproveApplicationResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - must be an Admin"),
        (status = 404, description = "Application not found"),
        (status = 409, description = "Application already reviewed"),
        (status = 422, description = "Validation Error"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Critic"]
)]
pub async fn approve_application(
    RequireRole(auth, _): RequireRole<role::Admin>,
    State(critic_service): State<Arc<dyn critic::UseCase>>,
    ValidatedPath(params): ValidatedPath<ApplicationPathParams>,
) -> Result<Json<ApproveApplicationResponse>, ApiError> {
    let application_id = Uuid::parse_str(&params.id)
        .map_err(|_| ApiError::BadRequest("Invalid application ID format.".to_string()))?;

    info!("Admin '{}' is approving critic application '{}'", auth.user_id, application_id);

    critic_service.approve_application(&application_id, &auth.user_id).await?;

    Ok(Json(ApproveApplicationResponse {
        message: "Application approved. User has been granted the Critic role.".to_string(),
    }))
}

#[utoipa::path(
    post,
    path = "/critic/applications/{id}/reject",
    description = "Admin rejects a pending Critic application.",
    params(
        ("id" = String, Path, description = "Application ID"),
    ),
    responses(
        (status = 200, description = "Application rejected", body = RejectApplicationResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - must be an Admin"),
        (status = 404, description = "Application not found"),
        (status = 409, description = "Application already reviewed"),
        (status = 422, description = "Validation Error"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Critic"]
)]
pub async fn reject_application(
    RequireRole(auth, _): RequireRole<role::Admin>,
    State(critic_service): State<Arc<dyn critic::UseCase>>,
    ValidatedPath(params): ValidatedPath<ApplicationPathParams>,
) -> Result<Json<RejectApplicationResponse>, ApiError> {
    let application_id = Uuid::parse_str(&params.id)
        .map_err(|_| ApiError::BadRequest("Invalid application ID format.".to_string()))?;

    info!("Admin '{}' is rejecting critic application '{}'", auth.user_id, application_id);

    critic_service.reject_application(&application_id, &auth.user_id).await?;

    Ok(Json(RejectApplicationResponse {
        message: "Application rejected.".to_string(),
    }))
}

impl From<critic::UseCaseError> for ApiError {
    fn from(e: critic::UseCaseError) -> Self {
        match e {
            critic::UseCaseError::ApplicationNotFound => {
                ApiError::BadRequest("Application not found.".to_string())
            }
            critic::UseCaseError::AlreadyReviewed => {
                ApiError::Conflict("This application has already been reviewed.".to_string())
            }
            critic::UseCaseError::Internal(msg) => {
                error!("Critic reject error: {msg}");
                ApiError::Internal("An internal error occurred".to_string())
            }
            _ => {
                error!("Critic reject error: {e}");
                ApiError::Internal("An internal error occurred".to_string())
            }
        }
    }
}