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
        routes::recipes::explore::popular,
        routes::recipes::explore::latest,
        routes::recipes::explore::top_tags,

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

            routes::recipes::dtos::RecipePreviewItem,

            routes::recipes::dtos::RecipePathParams,
            routes::recipes::dtos::RecipeResponse,

            routes::recipes::dtos::PopularRecipesQueryParams,
            routes::recipes::dtos::PopularRecipesResponse,

            routes::recipes::dtos::LatestRecipesQueryParams,
            routes::recipes::dtos::LatestRecipesResponse,

            routes::recipes::dtos::TopTagsQueryParams,
            routes::recipes::dtos::TopTagsResponse,

            // /tags
            routes::tags::dtos::TagsAutocompleteQueryParams,
            routes::tags::dtos::TagsAutocompleteResponse,
        )
    ),
)]
pub struct Docs;