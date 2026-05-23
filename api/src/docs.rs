use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Hexum",
        description = "Scalable Axum backend using Hexagonal Architecture",
    )
)]
pub struct MasterDocs;