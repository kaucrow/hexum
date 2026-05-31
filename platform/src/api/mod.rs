mod error;
mod docs;
mod extractors;
pub(crate) mod routes;

pub use docs::Docs;

#[allow(dead_code)]
pub use axum::{
    Json,
    extract::{State, Query, Path},
    response::{Html, Response, IntoResponse},
    http::StatusCode,
};
#[allow(dead_code)]
pub use utoipa::{ToSchema, IntoParams};
pub use axum_extra::extract::cookie::CookieJar;
pub use validator::Validate;
pub use askama::Template;

pub use error::ApiError;

use axum::{
    Router,
    routing::{get, post},
    body::Body,
    extract::{Request, FromRequest, FromRequestParts},
    http::request::Parts,
};
use serde::de::DeserializeOwned;
use crate::PlatformState;

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
        value.validate().map_err(|validation_errors| {
            let error_msg = format!("Validation failed: {}", validation_errors);
            ApiError::BadRequest(error_msg)
        })?;

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
        value.validate().map_err(|validation_errors| {
            let error_msg = format!("Validation failed: {}", validation_errors);

            ApiError::BadRequest(error_msg)
        })?;

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
        value.validate().map_err(|validation_errors| {
            let error_msg = format!("Validation failed: {}", validation_errors);
            ApiError::BadRequest(error_msg)
        })?;

        Ok(ValidatedPath(value))
    }
}

pub fn router(state: PlatformState, enable_dev_endpoints: bool) -> Router {
    let mut r = Router::new()
        .route("/user/register", post(crate::routes::user::register))
        .route("/user/verify", get(crate::routes::user::verify))
        .route("/user/verify-ui", get(crate::routes::user::verify_ui))
        .route("/auth/local/login", post(crate::routes::auth::local::login))
        .route("/auth/oauth/google/login", post(crate::routes::auth::oauth::google_login))
        .route("/auth/oauth/github/login", post(crate::routes::auth::oauth::github_login))
        .route("/auth/refresh-session", post(crate::routes::auth::refresh_session))
        .route("/auth/logout", post(crate::routes::auth::logout));

    if enable_dev_endpoints {
        r = r
            .route("/auth/oauth/login-ui", get(crate::routes::auth::oauth::oauth_login_ui))
            .route("/auth/oauth/callback-ui", get(crate::routes::auth::oauth::oauth_callback_ui));
    }

    r.with_state(state)
}