use utoipa::{
    Modify, OpenApi,
    openapi::security::SecurityScheme,
};
use super::routes;

#[derive(OpenApi)]
#[openapi(
    paths(
        // /user
        routes::user::register,
        routes::user::verify,
        routes::user::verify_ui,

        // /auth
        routes::auth::local::login,
        routes::auth::oauth::oauth_login_ui,
        routes::auth::oauth::google_login,
        routes::auth::oauth::github_login,
        routes::auth::refresh::refresh_session,
        routes::auth::logout::logout,
    ),
    components(
        schemas(
            // ==== Requests & Responses ====

            // /user
            routes::dtos::RegisterRequest,
            routes::dtos::RegisterResponse,
            routes::dtos::VerifyResponse,

            // /auth
            routes::auth::dtos::LoginRequest,
            routes::auth::dtos::LoginResponse,
            routes::auth::dtos::LogoutResponse,
            routes::auth::dtos::OAuthLoginRequest,
        )
    ),
    modifiers(&SecurityAddon),
)]
pub struct Docs;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);

        components.add_security_scheme(
            "cookie_auth",
            SecurityScheme::ApiKey(
                utoipa::openapi::security::ApiKey::Cookie(
                    utoipa::openapi::security::ApiKeyValue::new("access_token"),
                ),
            ),
        );

        components.add_security_scheme(
            "refresh_cookie_auth",
            SecurityScheme::ApiKey(
                utoipa::openapi::security::ApiKey::Cookie(
                    utoipa::openapi::security::ApiKeyValue::new("refresh_token"),
                ),
            ),
        );
    }
}