use crate::{
    api::*, features::recipe::{
        self, RecipeSearchResult, SearchResultsPage
    }, prelude::*
};
use super::dtos::*;

#[utoipa::path(
    get,
    path = "/recipes/search",
    description = "Searches for a recipe.",
    params(RecipeSearchQueryParams),
    responses(
        (status = 200, description = "Recipe search results", body = RecipeSearchResponse),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["Recipes"]
)]
pub async fn search(
    State(recipe_service): State<Arc<dyn recipe::UseCase>>,
    Query(queries): Query<RecipeSearchQueryParams>,
) -> Result<Json<RecipeSearchResponse>, ApiError> {
    info!("Getting {} recipes from page {} of search for query '{}'", queries.limit, queries.page, queries.query);

    let search_id: Option<Uuid> = queries.search_id
        .and_then(|id| Uuid::from_str(&id).ok());

    let search_result = recipe_service.search_recipe(
        &queries.query, queries.page, queries.limit, search_id,
    ).await?;

    Ok(Json(RecipeSearchResponse::from(search_result)))
}

impl From<SearchResultsPage> for RecipeSearchResponse {
    fn from(search_result: SearchResultsPage) -> Self {
        Self {
            recipes: search_result.items.into_iter().map(|item| RecipeSearchResultItem::from(item)).collect(),
            meta: RecipeSearchMeta {
                total_items: search_result.total_items,
                search_id: search_result.search_id.to_string(),
            }
        }
    }
}

impl From<RecipeSearchResult> for RecipeSearchResultItem {
    fn from(search_result: RecipeSearchResult) -> Self {
        Self {
            id: search_result.id.to_string(),
            origin: search_result.origin.to_string(),
            name: search_result.name,
            tags: search_result.tags,
            thumbnail_url: search_result.thumbnail_url,
        }
    }
}

impl From<recipe::UseCaseError> for ApiError {
    fn from(e: recipe::UseCaseError) -> Self {
        #[allow(unreachable_patterns)]
        match e {
            recipe::UseCaseError::Internal(e) => {
                error!("An internal error occurred: {e}");
                ApiError::Internal("An internal error occurred".to_string())
            }
            _ => {
                error!("Unexpected domain error: {e}");
                ApiError::Internal("An internal error occurred".to_string())
            }
        }
    }
}