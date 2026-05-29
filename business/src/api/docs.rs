use utoipa::OpenApi;
use super::routes;

#[derive(OpenApi)]
#[openapi(
    paths(
        // /business-health
        routes::health::health,

        // /recipes
        routes::recipes::sync::sync,
        routes::recipes::search::search,
        routes::recipes::get_by_id::get_by_id,

        // /tags
        routes::tags::autocomplete::autocomplete,
    ),
    components(
        schemas(
            // ==== Requests & Responses ====

            // /business-health
            routes::dtos::BusinessHealthResponse,

            // /recipes
            routes::recipes::dtos::RecipeSearchQueryParams,
            routes::recipes::dtos::RecipeSearchResponse,

            routes::recipes::dtos::RecipePathParams,
            routes::recipes::dtos::RecipeResponse,

            // /tags
            routes::tags::dtos::TagsAutocompleteQueryParams,
            routes::tags::dtos::TagsAutocompleteResponse,
        )
    ),
)]
pub struct Docs;