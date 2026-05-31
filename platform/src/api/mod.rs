mod error;
mod docs;
mod extractors;
pub(crate) mod routes;

pub use docs::Docs;

#[allow(unused_imports)]
pub(crate) use axum::{
    Json,
    extract::{State, Query},
    response::{Html, Response, IntoResponse},
    http::StatusCode,
};
pub(crate) use axum_extra::extract::cookie::CookieJar;
pub(crate) use askama::Template;

pub(crate) use error::ApiError;

use crate::PlatformState;
use axum::{
    Router,
    routing::{get, post},
};

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