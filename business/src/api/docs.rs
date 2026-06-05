use utoipa::OpenApi;
use super::routes;

#[derive(OpenApi)]
#[openapi(
    paths(
        // /business-health
        routes::health::health,

        // /recipes
        routes::recipes::create::create,
        routes::recipes::sync::sync,
        routes::recipes::search::search,
        routes::recipes::get_by_id::get_by_id,
        routes::recipes::delete::delete,
        routes::recipes::explore::popular,
        routes::recipes::explore::latest,
        routes::recipes::explore::top_tags,
        routes::recipes::history::history,
        routes::recipes::created::created,

        // /tags
        routes::tags::autocomplete::autocomplete,

        // /groups
        routes::groups::get_group::get_group,
        routes::groups::get_groups::get_groups,
        routes::groups::list_groups::list_groups,
        routes::groups::get_recipes::get_recipes,
        routes::groups::create::create,
        routes::groups::delete::delete,
        routes::groups::add_recipe::add_recipe,
        routes::groups::remove_recipe::remove_recipe,
    ),
    components(
        schemas(
            // ==== Requests & Responses ====

            // /business-health
            routes::dtos::BusinessHealthResponse,

            // /recipes
            routes::recipes::dtos::CreateRecipeRequest,
            routes::recipes::dtos::CreateRecipeResponse,

            routes::recipes::dtos::RecipeSearchQueryParams,
            routes::recipes::dtos::RecipeSearchResponse,
            routes::recipes::dtos::RecipeSearchMeta,

            routes::recipes::dtos::RecipePreviewItem,

            routes::recipes::dtos::RecipePathParams,
            routes::recipes::dtos::RecipeResponse,

            routes::recipes::dtos::PopularRecipesQueryParams,
            routes::recipes::dtos::PopularRecipesResponse,

            routes::recipes::dtos::LatestRecipesQueryParams,
            routes::recipes::dtos::LatestRecipesResponse,

            routes::recipes::dtos::TopTagsQueryParams,
            routes::recipes::dtos::TopTagsResponse,

            routes::recipes::dtos::RecipeHistoryQueryParams,
            routes::recipes::dtos::RecipeHistoryResponse,

            routes::recipes::dtos::UserCreatedRecipesQueryParams,
            routes::recipes::dtos::UserCreatedRecipesResponse,
            routes::recipes::dtos::UserCreatedRecipesMeta,

            // /tags
            routes::tags::dtos::TagsAutocompleteQueryParams,
            routes::tags::dtos::TagsAutocompleteResponse,

            // /groups
            routes::groups::dtos::UserGroupsQueryParams,
            routes::groups::dtos::UserGroupsResponse,
            routes::groups::dtos::UserGroupsMeta,
            routes::groups::dtos::RecipesGroupItem,

            routes::groups::dtos::GetGroupQueryParams,

            routes::groups::dtos::GroupRecipesQueryParams,
            routes::groups::dtos::GroupRecipesResponse,
            routes::groups::dtos::GroupRecipesMeta,

            routes::groups::dtos::GroupIdPathParams,
            routes::groups::dtos::GroupRecipePathParams,

            routes::groups::dtos::CreateGroupRequest,
            routes::groups::dtos::CreateGroupResponse,

            routes::groups::dtos::DeleteGroupPathParams,
            routes::groups::dtos::DeleteGroupResponse,

            routes::groups::dtos::UserGroupItem,

            routes::groups::dtos::UserGroupsListResponse,
        )
    ),
    modifiers(&SecurityAddon),
)]
pub struct Docs;

use utoipa::openapi::security::{SecurityScheme, ApiKey, ApiKeyValue};

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "cookie_auth",
                SecurityScheme::ApiKey(
                    ApiKey::Cookie(ApiKeyValue::new("access_token")),
                ),
            );
        }
    }
}