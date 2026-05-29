use crate::{
    prelude::*,
    api::*,
    features::recipe,
};
use super::dtos::*;

#[utoipa::path(
    get,
    path = "/recipe/search",
    description = "Checks if the business logic is healthy.",
    params(RecipeSearchQueryParams),
    responses(
        (status = 200, description = "Recipe search results", body = RecipeSearchResponse),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["Recipe"]
)]
pub async fn recipe_search(
    State(recipe_service): State<Arc<dyn recipe::UseCase>>,
    Query(queries): Query<RecipeSearchQueryParams>,
) -> Result<Json<RecipeSearchResponse>, ApiError> {
    info!("Getting page {} of search for recipe '{}'", queries.page, queries.name);

    let search_result = recipe_service.search_recipe_by_name(&queries.name, queries.page).await?;

    let response = RecipeSearchResponse::new(
        search_result.items.into_iter().map(|item| RecipeSearchResult::from(item)).collect(),
        search_result.total_items,
    );

    Ok(Json(response))
}

impl From<recipe::domain::RecipeSearchResult> for RecipeSearchResult {
    fn from(search_result: recipe::domain::RecipeSearchResult) -> Self {
        let id = match search_result.origin {
            recipe::RecipeOrigin::Local(id) => id.to_string(),
            recipe::RecipeOrigin::External(ref id) => id.clone(),
        };

        Self {
            id,
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