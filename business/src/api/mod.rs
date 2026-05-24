pub(crate) mod routes;
mod error;
mod docs;

pub use docs::Docs;

#[allow(unused_imports)]
pub(crate) use axum::{
    Json,
    extract::{State, Query},
    response::{Html, Response, IntoResponse},
    http::StatusCode,
};

pub(crate) use error::ApiError;

use axum::{Router, routing::get};
use crate::BusinessState;

pub fn router(state: BusinessState) -> Router {
    Router::new()
        .route("/business-health", get(crate::routes::health::health))
        .with_state(state)
}