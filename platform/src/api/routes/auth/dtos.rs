use crate::{
    prelude::*,
    api::*,
};

#[derive(Deserialize, ToSchema, Validate)]
pub struct LoginRequest {
    #[schema(example = "alicesmith")]
    #[validate(length(max = 200))]
    /// Username or Email Address
    pub identity: String,

    #[schema(example = "supersecret123")]
    #[validate(length(max = 200))]
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    #[schema(example = "Login successful.")]
    pub message: String,
}

#[derive(Serialize, ToSchema)]
pub struct LogoutResponse {
    #[schema(example = "Logout successful.")]
    pub message: String,
}

#[derive(Deserialize, ToSchema)]
pub struct OAuthLoginRequest {
    #[schema(example = "4/0AfgeXvvV...")]
    pub code: String,
}