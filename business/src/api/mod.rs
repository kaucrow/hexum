mod routes;
mod docs;

pub use docs::Docs;

pub use platform::api::*;

use axum::{Router, routing::get};
use crate::BusinessState;

pub fn router(state: BusinessState) -> Router {
    Router::new()
        .route("/business-health", get(routes::health::health))
        .route("/recipes/sync", get(routes::recipes::sync))
        .route("/recipes/search", get(routes::recipes::search))
        .route("/recipes/{id}", get(routes::recipes::get_by_id))
        .route("/recipes/popular", get(routes::recipes::popular))
        .route("/recipes/latest", get(routes::recipes::latest))
        .route("/recipes/top-tags", get(routes::recipes::top_tags))
        .route("/tags/autocomplete", get(routes::tags::autocomplete))
        .with_state(state)
}