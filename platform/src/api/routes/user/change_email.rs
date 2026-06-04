use crate::{
    prelude::*,
    features::user,
    api::*,
};
use super::dtos::*;

#[utoipa::path(
    post,
    path = "/user/change-email",
    description = "Initiates an email change for the authenticated user. A 6-digit verification code will be sent to the new email address.",
    request_body = ChangeEmailRequest,
    responses(
        (status = 200, description = "Verification email sent", body = ChangeEmailResponse),
        (status = 400, description = "Not logged in via username/email"),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "The new email is already in use"),
        (status = 422, description = "Validation Error"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["User"]
)]
pub async fn change_email(
    auth: AuthenticatedUser,
    State(user_service): State<Arc<dyn user::UseCase>>,
    ValidatedJson(payload): ValidatedJson<ChangeEmailRequest>,
) -> Result<Json<ChangeEmailResponse>, ApiError> {
    let user_id = &auth.user_id;

    if !(auth.provider != AuthProvider::Local) {
        return Err(ApiError::BadRequest(
            "You must be logged in via username/email to request an email change".to_string()
        ));
    }

    info!("Email change request for user ID '{}'", user_id);

    user_service.change_user_email(&payload.new_email).await?;

    info!("Verification code sent to new email for user ID '{}'", &user_id);

    let response = ChangeEmailResponse {
        message: "A 6-digit verification code has been sent to your new email. Please check your inbox and enter the code to confirm the email change.".to_string(),
    };
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/user/verify-email-change",
    description = "Completes the email change by verifying the 6-digit code sent to the new email address.",
    request_body = VerifyEmailChangeRequest,
    responses(
        (status = 200, description = "Email change successful", body = VerifyEmailChangeResponse),
        (status = 400, description = "Invalid or expired code"),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "The new email is already in use"),
        (status = 422, description = "Validation Error"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["User"]
)]
pub async fn verify_email_change(
    auth: AuthenticatedUser,
    State(user_service): State<Arc<dyn user::UseCase>>,
    ValidatedJson(payload): ValidatedJson<VerifyEmailChangeRequest>,
) -> Result<Json<VerifyEmailChangeResponse>, ApiError> {
    info!("Verifying email change for user ID '{}'", &auth.user_id);

    user_service.verify_user_email_change(&auth.user_id, &payload.code).await?;

    info!("Email change successful for user ID '{}'", &auth.user_id);

    let response = VerifyEmailChangeResponse {
        message: "Email change successful. Your email has been updated.".to_string(),
    };
    Ok(Json(response))
}