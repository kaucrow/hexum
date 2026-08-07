use crate::{
    prelude::*,
    features::user::{self, User, NewUserData},
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
    let user_id = auth.user_id;

    info!("Getting profile data for user ID '{}'", &user_id);

    let user_data = user_service
        .get_user_by_id(&user_id)
        .await?
        .ok_or(ApiError::NotFound(format!("User with ID '{}' was not found.", &user_id)))?;

    let response = UserDataResponse::from(user_data);
    Ok(Json(response))
}

#[utoipa::path(
    patch,
    path = "/user/update-data",
    description = "Updates data in the user's profile. Accepts a multipart form with optional `new_username` text field and optional `image` file. If the image is omitted the existing profile picture is left unchanged.",
    request_body(content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "User data updated successfully", body = UserDataUpdateResponse),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "New data has a conflict with existing data"),
        (status = 422, description = "Validation Error"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["User"]
)]
pub async fn update_user_data(
    auth: AuthenticatedUser,
    State(state): State<PlatformState>,
    multipart: Multipart,
) -> Result<Json<UserDataUpdateResponse>, ApiError> {
    let user_id = auth.user_id;

    info!("Updating profile data for user ID '{}'", &user_id);

    // ─── Parse multipart form ───
    let form = ParsedUserDataForm::from_multipart(multipart).await?;

    // ─── Convert Bytes → Vec<u8> for the service layer ───
    let image: Option<(Vec<u8>, String)> = form.image.map(|(bytes, ct)| (bytes.to_vec(), ct));

    let new_data = NewUserData {
        username: form.new_username,
        profile_picture_url: None,
    };

    // The service handles image saving (filename = user UUID + extension) and
    // delegates the final data update to the repository.
    state.user.update_user_data(&user_id, new_data, image).await?;

    let response = UserDataUpdateResponse { message: "User data updated successfully.".to_string() };
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
            profile_picture_url: user.profile_picture_url,
        }
    }
}

// ─── Multipart form parsing ───

struct ParsedUserDataForm {
    pub new_username: Option<String>,
    pub image: Option<(Bytes, String)>,
}

impl ParsedUserDataForm {
    // Limits image payloads to 20MB to avoid memory exhaustion
    const MAX_FILE_SIZE: usize = 20 * 1024 * 1024;

    pub async fn from_multipart(mut multipart: Multipart) -> Result<Self, ApiError> {
        let mut new_username: Option<String> = None;
        let mut image_data: Option<(Bytes, String)> = None;

        while let Some(field) = multipart.next_field().await.map_err(|e| {
            ApiError::BadRequest(format!("Multipart processing stream failed: {e}"))
        })? {
            let field_name: String = field.name().map(|s| s.to_string()).unwrap_or_default();

            match field_name.as_str() {
                "new_username" => {
                    let text = field.text().await.map_err(|e| {
                        ApiError::BadRequest(format!("Failed to read username field: {e}"))
                    })?;
                    let trimmed = text.trim().to_string();
                    if !trimmed.is_empty() {
                        new_username = Some(trimmed);
                    }
                }
                "image" => {
                    let content_type: String = field
                        .content_type()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "application/octet-stream".to_string());
                    let bytes = field.bytes().await.map_err(|e| {
                        ApiError::BadRequest(format!("Failed to read image data: {e}"))
                    })?;

                    if bytes.len() > Self::MAX_FILE_SIZE {
                        return Err(ApiError::BadRequest(
                            "Image payload size limit exceeded (20MB max)".to_string(),
                        ));
                    }

                    if !bytes.is_empty() {
                        image_data = Some((bytes, content_type));
                    }
                }
                _ => {
                    warn!("Unexpected field in multipart form: {field_name}");
                }
            }
        }

        Ok(Self {
            new_username,
            image: image_data,
        })
    }
}