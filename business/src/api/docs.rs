use utoipa::OpenApi;
use super::routes;

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::health::health,
        routes::games::search::search,
        routes::platforms::get::get_platforms,
        routes::platforms::sync::sync_platforms,
    ),
    components(
        schemas(
            // ─── Requests & Responses ───
            routes::dtos::BusinessHealthResponse,
            routes::games::dtos::GameSearchResultItemResponse,
            routes::games::dtos::GameSearchMeta,
            routes::games::dtos::GameSearchResponse,
            routes::platforms::dtos::PlatformResponse,
            routes::platforms::dtos::PlatformSyncResponse,
        )
    ),
)]
pub struct Docs;