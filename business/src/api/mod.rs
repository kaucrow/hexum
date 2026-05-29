mod routes;
mod error;
mod docs;

pub use docs::Docs;

#[allow(unused_imports)]
pub(crate) use axum::{
    Json,
    extract::{State, Query, Path},
    response::{Html, Response, IntoResponse},
    http::StatusCode,
};
pub(crate) use utoipa::{IntoParams, ToSchema};

pub(crate) use error::ApiError;

use axum::{Router, routing::get};
use crate::BusinessState;

pub fn router(state: BusinessState) -> Router {
    Router::new()
        .route("/business-health", get(routes::health::health))
        .route("/recipes/sync", get(routes::recipes::sync))
        .route("/recipes/search", get(routes::recipes::search))
        .route("/recipes/{id}", get(routes::recipes::get_by_id))
        .route("/tags/autocomplete", get(routes::tags::autocomplete))
        .with_state(state)
}