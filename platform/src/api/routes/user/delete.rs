use crate::{
    Config,
    prelude::*,
    features::{user, auth},
    api::*,
};
use super::dtos::*;

#[utoipa::path(
    delete,
    path = "/user",
    description = "Deletes the authenticated user's account & logs them out.",
    responses(
        (status = 200, description = "User deleted & logged out successfully", body = UserDeletionResponse),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation Error"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["User"]
)]
pub async fn delete(
    auth: AuthenticatedUser,
    State(config): State<Arc<Config>>,
    State(user_service): State<Arc<dyn user::UseCase>>,
    State(auth_service): State<Arc<dyn auth::UseCase>>,
    jar: CookieJar,
) -> Result<(CookieJar, Json<UserDeletionResponse>), ApiError> {
    let user_id = auth.user_id;

    info!("Deleting user with ID '{}'", &user_id);

    // Delete the user account
    let _ = user_service
        .delete_user(&user_id)
        .await?
        .ok_or(ApiError::NotFound(format!("User with ID '{}' was not found.", &user_id)))?;

    // Log out the user
    if let Some(cookie) = jar.get("refresh_token") {
        let _ = auth_service.logout_user(cookie.value()).await;
    }

    let access_cookie = build_removal_cookie("access_token", "/", &config.api.protocol);
    let refresh_cookie = build_removal_cookie("refresh_token", "/auth/refresh-session", &config.api.protocol);

    let response = UserDeletionResponse { message: "User deleted successfully.".to_string() };
    Ok((jar.add(access_cookie).add(refresh_cookie), Json(response)))
}