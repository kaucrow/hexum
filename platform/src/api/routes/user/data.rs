use crate::{
    prelude::*,
    features::user::{self, User},
    api::*,
};
use super::dtos::*;

#[utoipa::path(
    get,
    path = "/user",
    description = "Gets the authenticated user's profile data.",
    responses(
        (status = 200, description = "User's profile data", body = UserDataResponse),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation Error"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["User"]
)]
pub async fn get_user_data(
    auth: AuthenticatedUser,
    State(user_service): State<Arc<dyn user::UseCase>>,
) -> Result<Json<UserDataResponse>, ApiError> {
    let user_id = auth.0.id;

    info!("Getting profile data for user ID '{}'", &user_id);

    let user_data = user_service
        .get_user_by_id(&user_id)
        .await?
        .ok_or(ApiError::NotFound(format!("User with ID '{}' was not found.", &user_id)))?;

    let response = UserDataResponse::from(user_data);

    Ok(Json(response))
}

impl From<User> for UserDataResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username.as_str().to_string(),
            email: user.email.as_str().to_string(),
            roles: user.roles.into_iter().map(|role| role.to_string()).collect(),
            is_active: user.is_active,
        }
    }
}