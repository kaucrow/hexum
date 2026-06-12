use crate::{
    Config,
    prelude::*,
    features::{user, ratelimit},
    api::*,
};
use super::dtos::*;

#[utoipa::path(
    post,
    path = "/user/register",
    description = "Registers a new user with username, password & email. The username must only contain alphanumeric characters, and will be converted to lowercase. A 6-digit verification code will be sent to the user's email address.",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "Registration successful", body = RegisterResponse),
        (status = 409, description = "The provided username or email is already in use"),
        (status = 422, description = "Validation Error"),
        (status = 429, description = "Too Many Requests"),
        (status = 500, description = "Internal Server Error"),
    ),
    tags = ["User"]
)]
pub async fn register(
    State(config): State<Arc<Config>>,
    State(user_service): State<Arc<dyn user::UseCase>>,
    State(ratelimit): State<Arc<dyn ratelimit::UseCase>>,
    ClientIp(client_ip): ClientIp,
    ValidatedJson(payload): ValidatedJson<RegisterRequest>,
) -> Result<Json<RegisterResponse>, ApiError> {
    // ─── IP-based rate limiting for registration (per hour) ───
    ratelimit
        .check_custom_limit(
            &format!("ratelimit:ip:{}:register", client_ip),
            config.ratelimit.register_ip_max_per_hour,
            3600,
        )
        .await?;

    info!("User registration request with username `{}` & email `{}`", &payload.username, &payload.email);

    let user = user::User::new(&payload.username, &payload.email)?;
    user_service.register_user(user, &payload.password).await?;

    info!("Registration successful for user with username `{}` & email `{}`", &payload.username, &payload.email);

    let response = RegisterResponse {
        message: "Registration successful. A 6-digit verification code has been sent to your email. Please check your inbox and enter the code to activate your account.".to_string()
    };
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/user/verify-account",
    description = "Verifies the user account using a 6-digit code sent via email.",
    request_body = VerifyAccountRequest,
    responses(
        (status = 200, description = "Account verified successfully", body = VerifyAccountResponse),
        (status = 400, description = "Invalid or expired code"),
        (status = 422, description = "Validation Error"),
        (status = 429, description = "Too Many Requests"),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["User"]
)]
pub async fn verify_account(
    State(config): State<Arc<Config>>,
    State(user_service): State<Arc<dyn user::UseCase>>,
    State(ratelimit): State<Arc<dyn ratelimit::UseCase>>,
    ClientIp(client_ip): ClientIp,
    ValidatedJson(payload): ValidatedJson<VerifyAccountRequest>,
) -> Result<Json<VerifyAccountResponse>, ApiError> {
    // ── IP-based rate limiting for verification (per minute) ──
    ratelimit
        .check_custom_limit(
            &format!("ratelimit:ip:{}:verify", client_ip),
            config.ratelimit.verify_max_per_minute,
            60,
        )
        .await?;

    info!("Verifying account with code: {}", &payload.code);

    user_service.verify_user_account(&payload.code).await?;

    info!("Account successfully verified for code");

    Ok(Json(VerifyAccountResponse {
        message: "Account verification successful. You can now log in.".to_string(),
    }))
}

impl From<user::UserError> for ApiError {
    fn from(e: user::UserError) -> Self {
        match e {
            user::UserError::InvalidUsername => {
                warn!("Validation error: {e}");
                let mut errors = HashMap::new();
                errors.insert("username".to_string(), vec![e.to_string()]);
                ApiError::Validation(errors)
            }
            user::UserError::InvalidEmail => {
                warn!("Validation error: {e}");
                let mut errors = HashMap::new();
                errors.insert("email".to_string(), vec![e.to_string()]);
                ApiError::Validation(errors)
            }
            user::UserError::InvalidPassword => {
                warn!("Validation error: {e}");
                let mut errors = HashMap::new();
                errors.insert("password".to_string(), vec![e.to_string()]);
                ApiError::Validation(errors)
            }
            user::UserError::UserAlreadyDeactivated
            | user::UserError::InsufficientPermissions => {
                error!("Unexpected domain error: {e}");
                ApiError::Internal
            }
        }
    }
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
                ApiError::from(e)
            },
            user::UseCaseError::VerificationTokenInvalid => {
                warn!("User verification token is invalid. It might have expired.");
                ApiError::Unauthorized("Failed to verify account due to email expiration.".to_string())
            },
            user::UseCaseError::Internal(e) => {
                error!("An internal error occurred: {e}");
                ApiError::Internal
            }
        }
    }
}