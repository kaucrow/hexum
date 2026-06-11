use crate::{
    prelude::*,
    api::*,
};
use utoipa::ToSchema;

// ─── User Registration DTOs ───

#[derive(Deserialize, ToSchema, Validate)]
pub struct RegisterRequest {
    #[schema(example = "johndoe")]
    #[validate(length(max = 200))]
    pub username: String,

    #[schema(example = "MyP@ssword123")]
    #[validate(length(max = 200))]
    pub password: String,

    #[schema(example = "johndoe@gmail.com")]
    #[validate(email)]
    pub email: String
}

#[derive(Serialize, ToSchema)]
pub struct RegisterResponse {
    #[schema(example = "Registration successful. A verification code has been sent to your email. Please check your inbox and enter the code to activate your account.")]
    pub message: String,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct VerifyAccountRequest {
    /// The 6-digit verification code sent via email.
    #[schema(example = "042739")]
    #[validate(length(equal = 6))]
    pub code: String,
}

#[derive(Serialize, ToSchema)]
pub struct VerifyAccountResponse {
    #[schema(example = "Account verification successful. You can now log in.")]
    pub message: String,
}

// ─── Email Change DTOs ───

#[derive(Deserialize, ToSchema, Validate)]
pub struct ChangeEmailRequest {
    /// The new email address to change to.
    #[schema(example = "newemail@gmail.com")]
    #[validate(email)]
    pub new_email: String,
}

#[derive(Serialize, ToSchema)]
pub struct ChangeEmailResponse {
    #[schema(example = "A 6-digit verification code has been sent to your new email. Please check your inbox and enter the code to confirm the email change.")]
    pub message: String,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct VerifyEmailChangeRequest {
    /// The 6-digit verification code sent to the new email.
    #[schema(example = "042739")]
    #[validate(length(equal = 6))]
    pub code: String,
}

#[derive(Serialize, ToSchema)]
pub struct VerifyEmailChangeResponse {
    #[schema(example = "Email change successful. Your email has been updated.")]
    pub message: String,
}

// ─── User Data DTOs ───

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "id": "05639468-710b-44fe-9fc7-372514e95c37",
    "username": "johndoe",
    "email": "johndoe@gmail.com",
    "roles": ["BasicUser", "Admin", "Owner"],
    "isActive": true,
}))]
pub struct UserDataResponse {
    /// The User's ID (UUID).
    #[schema(format = "uuid")]
    pub id: String,

    pub username: String,

    #[schema(format = "email")]
    pub email: String,

    /// The roles that the user has.
    pub roles: Vec<String>,

    /// The user's "is_active" flag. False if the user has been suspended.
    pub is_active: bool,
}

#[derive(Deserialize, ToSchema, Validate, Debug)]
pub struct UserDataUpdateRequest {
    /// Sets a new username.
    #[schema(example = "janedoe")]
    #[validate(length(max = 200))]
    pub new_username: Option<String>
}

#[derive(Serialize, ToSchema)]
pub struct UserDataUpdateResponse {
    #[schema(example = "User data updated successfully.")]
    pub message: String,
}

// ─── User Deletion DTOs ───

#[derive(Serialize, ToSchema)]
pub struct UserDeletionResponse {
    #[schema(example = "User deleted successfully.")]
    pub message: String,
}