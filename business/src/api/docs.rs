use utoipa::OpenApi;
use super::routes;

#[derive(OpenApi)]
#[openapi(
    paths(
        // /business-health
        routes::health::health,

        // /recipe
        routes::recipes::search::recipe_search
    ),
    components(
        schemas(
            // ==== Requests & Responses ====

            // /business-health
            routes::dtos::BusinessHealthResponse,

            // /recipe
            routes::recipes::dtos::RecipeSearchQueryParams,
            routes::recipes::dtos::RecipeSearchResponse,
        )
    ),
)]
pub struct Docs;