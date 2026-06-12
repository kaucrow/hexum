mod error;
mod docs;
pub mod extractors;
pub(crate) mod routes;

pub use docs::Docs;

#[allow(dead_code)]
pub use axum::{
    Json, Router,
    routing::{get, post, delete, put, patch},
    extract::{State, Query, Path, FromRef},
    response::{Html, Response, IntoResponse},
    http::{StatusCode, HeaderMap},
    body::Bytes,
};
#[allow(dead_code)]
pub use utoipa::{ToSchema, IntoParams};
#[allow(dead_code)]
pub use axum_extra::extract::{CookieJar, Multipart};
pub use validator::Validate;
pub use askama::Template;

pub use error::ApiError;
pub use extractors::{AuthenticatedUser, ClientIp, OptionalUser, RequireRole, role};
pub use crate::features::user::AuthProvider;

use axum::{
    body::Body,
    extract::{Request, FromRequest, FromRequestParts},
    http::request::Parts,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use time::Duration;
use serde::de::DeserializeOwned;
use validator::ValidationErrors;
use crate::{
    PlatformState,
    prelude::*,
    config::ApiProtocol,
};

pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S, Body> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = ApiError;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        // Try to parse the request body as JSON
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|err| ApiError::BadRequest(format!("Malformed JSON payload: {}", err)))?;

        // Validate the struct
        value.validate().map_err(|e| e.into_api_error())?;

        Ok(ValidatedJson(value))
    }
}

pub struct ValidatedQuery<T>(pub T);

impl<S, T> FromRequestParts<S> for ValidatedQuery<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection>
    {
        // Try to parse the query parameters from the URL
        let Query(value) = Query::<T>::from_request_parts(parts, state)
            .await
            .map_err(|err| ApiError::BadRequest(format!("Malformed query parameters: {}", err)))?;

        // Validate the struct
        value.validate().map_err(|e| e.into_api_error())?;

        Ok(ValidatedQuery(value))
    }
}

pub struct ValidatedPath<T>(pub T);

impl<S, T> FromRequestParts<S> for ValidatedPath<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate + Send,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Try to parse the path parameters
        let Path(value) = Path::<T>::from_request_parts(parts, state)
            .await
            .map_err(|err| ApiError::BadRequest(format!("Malformed path parameters: {}", err)))?;

        // Validate the struct
        value.validate().map_err(|e| e.into_api_error())?;

        Ok(ValidatedPath(value))
    }
}

trait ValidationErrorsExt {
    fn into_api_error(self) -> ApiError;
}

impl ValidationErrorsExt for ValidationErrors {
    fn into_api_error(self) -> ApiError {
        let mut error_map = HashMap::new();

        for (field, errors) in self.field_errors() {
            let messages: Vec<String> = errors
                .iter()
                .map(|error| {
                    match &error.message {
                        Some(msg) => msg.to_string(),
                        None => format!("Invalid value for rule: {}", error.code),
                    }
                })
                .collect();

            error_map.insert(field.to_string(), messages);
        }

        ApiError::Validation(error_map)
    }
}

// Helper function to build cookies
pub fn build_cookie<'a>(name: &'a str, value: String, path: &'a str, protocol: &ApiProtocol) -> Cookie<'a> {
    let mut cookie = Cookie::build((name, value))
        .http_only(true)
        .same_site(SameSite::Strict)
        .path(path);

    if matches!(protocol, ApiProtocol::Http) {
        cookie = cookie.secure(false);
    } else {
        cookie = cookie.secure(true);
    }

    cookie.build()
}


// Helper function to build removal cookies
pub fn build_removal_cookie<'a>(name: &'a str, path: &'a str, protocol: &ApiProtocol) -> Cookie<'a> {
    let mut cookie = Cookie::build((name, ""))
        .http_only(true)
        .same_site(SameSite::Strict)
        .path(path)
        .max_age(Duration::ZERO);

    if matches!(protocol, ApiProtocol::Http) {
        cookie = cookie.secure(false);
    } else {
        cookie = cookie.secure(true);
    }

    cookie.build()
}

pub fn router(state: PlatformState, enable_dev_endpoints: bool) -> Router {
    let mut r = Router::new()
        .route("/user", get(routes::user::get_user_data))
        .route("/user", delete(routes::user::delete))
        .route("/user/register", post(routes::user::register))
        .route("/user/verify-account", post(routes::user::verify_account))
        .route("/user/change-email", post(routes::user::change_email))
        .route("/user/verify-email-change", post(routes::user::verify_email_change))
        .route("/user/update-data", patch(routes::user::update_user_data))
        .route("/auth/local/login", post(routes::auth::local::login))
        .route("/auth/oauth/google/login", post(routes::auth::oauth::google_login))
        .route("/auth/oauth/github/login", post(routes::auth::oauth::github_login))
        .route("/auth/refresh-session", post(routes::auth::refresh_session))
        .route("/auth/logout", post(routes::auth::logout));

    if enable_dev_endpoints {
        r = r
            .route("/auth/oauth/login-ui", get(crate::routes::auth::oauth::oauth_login_ui))
            .route("/auth/oauth/callback-ui", get(crate::routes::auth::oauth::oauth_callback_ui));
    }

    r.with_state(state)
}