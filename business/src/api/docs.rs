use utoipa::OpenApi;
use super::routes;

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::health::health,
    ),
    components(
        schemas(
            // ==== Requests & Responses ====
            routes::dtos::BusinessHealthResponse,
        )
    ),
)]
pub struct Docs;