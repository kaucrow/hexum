use crate::{
    Config,
    prelude::*,
    features::user,
    api::*,
};
use super::dtos::*;

#[utoipa::path(
    post,
    path = "/user/register",
    description = "Registers a new user with username, password & email. The username must only contain alphanumeric characters, and will be converted to lowercase.",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "Registration successful", body = RegisterResponse),
        (status = 400, description = "Bad Request - Invalid email format, username format, or password too short"),
        (status = 409, description = "Conflict - The provided username or email is already in use"),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["User"]
)]
pub async fn register(
    State(user_service): State<Arc<dyn user::UseCase>>,
    ValidatedJson(payload): ValidatedJson<RegisterRequest>,
) -> Result<Json<RegisterResponse>, ApiError> {
    info!("User registration request with username `{}` & email `{}`", &payload.username, &payload.email);

    let user = user::User::new(&payload.username, &payload.email)?;
    user_service.register_user(user, &payload.password).await?;

    info!("Registration successful for user with username `{}` & email `{}`", &payload.username, &payload.email);

    let response = RegisterResponse {
        message: "Registration successful. A verification link has been sent to your email. Please click it to activate your account.".to_string()
    };
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/user/verify",
    description = "Validates the email verification token & activates the user account.",
    params(VerifyQueryParams),
    responses(
        (status = 200, description = "Account verified successfully"),
        (status = 400, description = "Invalid or expired token"),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["User"]
)]
pub async fn verify(
    State(user_service): State<Arc<dyn user::UseCase>>,
    Query(queries): Query<VerifyQueryParams>,
) -> Result<Json<RegisterResponse>, ApiError> {
    info!("Verifying account with token: {}", &queries.token);

    user_service.verify_user_account(&queries.token).await?;

    info!("Account successfully verified for token: {}", &queries.token);

    Ok(Json(RegisterResponse {
        message: "Account verification successful. You can now log in.".to_string(),
    }))
}

impl From<user::UserError> for ApiError {
    fn from(e: user::UserError) -> Self {
        match e {
            user::UserError::InvalidUsername
            | user::UserError::InvalidEmail
            | user::UserError::InvalidPassword => {
                warn!("Validation error: {e}");
                ApiError::BadRequest(e.to_string())
            }
            user::UserError::UserAlreadyDeactivated
            | user::UserError::InsufficientPermissions => {
                error!("Unexpected domain error: {e}");
                ApiError::Internal("An internal error occurred".to_string())
            }
        }
    }
}

#[derive(Template)]
#[template(path = "verify.html")]
pub struct VerifyTemplate {
    token: String,
    api_path_suffix: String,
}

#[utoipa::path(
    get,
    path = "/user/verify-ui",
    description = "**[DEVELOPMENT ENDPOINT]** Renders an email verification UI for testing purposes.",
    params(
        ("token" = String, Query, description = "The verification token sent via email")
    ),
    responses(
        (
            status = 200,
            description = "Successfully rendered the verification HTML page",
            body = String,
            content_type = "text/html"
        ),
        (status = 400, description = "Missing or invalid query parameters"),
        (status = 401, description = "Verification token invalid/expired"),
        (status = 500, description = "Internal Server Error: Template rendering failed")
    ),
    tags = ["User"]
)]
pub async fn verify_ui(
    State(config): State<Arc<Config>>,
    Query(queries): Query<VerifyQueryParams>,
) -> Result<impl IntoResponse, ApiError> {
    let template = VerifyTemplate {
        token: queries.token,
        api_path_suffix: config.api.path_suffix.clone(),
    };

    let html_content = template
        .render()
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Html(html_content))
}

impl From<user::UseCaseError> for ApiError {
    fn from(e: user::UseCaseError) -> Self {
        match e {
            user::UseCaseError::Conflict(c) => match c {
                user::ConflictError::UsernameInUse => {
                    warn!("{c}");
                    ApiError::Conflict("A user with this username already exists.".to_string())
                }
                user::ConflictError::EmailInUse => {
                    warn!("{c}");
                    ApiError::Conflict("A user with this email already exists.".to_string())
                }
            },
            user::UseCaseError::Validation(e) => {
                warn!("Validation error: {e}");
                ApiError::BadRequest(e.to_string())
            },
            user::UseCaseError::VerificationTokenInvalid => {
                warn!("User verification token is invalid. It might have expired.");
                ApiError::Unauthorized("Failed to verify account due to email expiration.".to_string())
            },
            user::UseCaseError::Internal(e) => {
                error!("An internal error occurred: {e}");
                ApiError::Internal("An internal error occurred".to_string())
            }
        }
    }
}