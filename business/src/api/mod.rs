mod routes;
mod docs;

pub use docs::Docs;

pub use platform::api::*;

use axum::{Router, routing::{get, post}};
use tower_http::services::ServeDir;
use crate::BusinessState;

pub fn router(state: BusinessState, upload_dir: String) -> Router {
    Router::new()
        .route("/business-health", get(routes::health::health))
        .route("/recipes", post(routes::recipes::create))
        .route("/recipes/sync", get(routes::recipes::sync))
        .route("/recipes/search", get(routes::recipes::search))
        .route("/recipes/{id}", get(routes::recipes::get_by_id))
        .route("/recipes/popular", get(routes::recipes::popular))
        .route("/recipes/latest", get(routes::recipes::latest))
        .route("/recipes/top-tags", get(routes::recipes::top_tags))
        .route("/tags/autocomplete", get(routes::tags::autocomplete))
        .nest_service("/uploads", ServeDir::new(&upload_dir))
        .with_state(state)
}