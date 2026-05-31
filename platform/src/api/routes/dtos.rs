use crate::{
    prelude::*,
    api::*,
};
use utoipa::{ToSchema, IntoParams};

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
    #[schema(example = "Registration successful. A verification link has been sent to your email. Please click it to activate your account.")]
    pub message: String,
}

#[derive(Deserialize, IntoParams)]
pub struct VerifyQueryParams {
    /// The verification token sent via email.
    pub token: String,
}

#[derive(Serialize, ToSchema)]
pub struct VerifyResponse {
    #[schema(example = "Account verification successful. You can now log in.")]
    pub message: String,
}