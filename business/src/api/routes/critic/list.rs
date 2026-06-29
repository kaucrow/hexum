use crate::{
    prelude::*,
    api::*,
    features::critic,
};
use super::dtos::*;

#[utoipa::path(
    get,
    path = "/critic/applications",
    description = "Admin lists all pending Critic applications.",
    responses(
        (status = 200, description = "List of pending applications", body = [CriticApplicationResponse]),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - must be an Admin"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Critic"]
)]
pub async fn list_applications(
    RequireRole(_auth, _): RequireRole<role::Admin>,
    State(critic_service): State<Arc<dyn critic::UseCase>>,
) -> Result<Json<Vec<CriticApplicationResponse>>, ApiError> {
    info!("Admin is listing pending critic applications");

    let applications = critic_service
        .list_pending_applications()
        .await
        .map_err(|e| {
            error!("Critic list error: {e}");
            ApiError::Internal("An internal error occurred".to_string())
        })?;

    let response: Vec<CriticApplicationResponse> = applications
        .into_iter()
        .map(|app| CriticApplicationResponse {
            id: app.id.to_string(),
            user_id: app.user_id.to_string(),
            username: app.username.unwrap_or_default(),
            email: app.email.unwrap_or_default(),
            status: app.status.to_string(),
            applied_at: app.applied_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(response))
}