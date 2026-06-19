use utoipa::OpenApi;
use super::routes;

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::health::health,
        routes::games::search::search,
    ),
    components(
        schemas(
            // ─── Requests & Responses ───
            routes::dtos::BusinessHealthResponse,
            routes::games::dtos::GameSearchResultItemDto,
            routes::games::dtos::GameSearchMeta,
            routes::games::dtos::GameSearchResponse,
        )
    ),
)]
pub struct Docs;