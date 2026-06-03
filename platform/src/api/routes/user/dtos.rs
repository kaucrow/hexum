use crate::{
    prelude::*,
    api::*,
};
use utoipa::ToSchema;

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
pub struct VerifyRequest {
    /// The 6-digit verification code sent via email.
    #[schema(example = "042739")]
    #[validate(length(equal = 6))]
    pub code: String,
}

#[derive(Serialize, ToSchema)]
pub struct VerifyResponse {
    #[schema(example = "Account verification successful. You can now log in.")]
    pub message: String,
}

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