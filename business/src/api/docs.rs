use utoipa::OpenApi;
use super::routes;

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::health::health,
        routes::games::get::get_game,
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
            routes::games::dtos::GameResponse,
            routes::games::dtos::GamePlatformResponse,
            routes::platforms::dtos::PlatformResponse,
            routes::platforms::dtos::PlatformSyncResponse,
        )
    ),
)]
pub struct Docs;