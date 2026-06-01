use crate::{
    prelude::*,
    api::*,
    features::recipe::{
        self, RecipePreview,
    },
};
use super::dtos::*;

#[utoipa::path(
    get,
    path = "/recipes/popular",
    description = "Gets the current most popular recipes.",
    params(PopularRecipesQueryParams),
    responses(
        (status = 200, description = "Current Most Popular Recipes", body = PopularRecipesResponse),
        (status = 422, description = "Validation Error"),
        (status = 500, description = "Internal Server Error"),
    ),
    tags = ["Recipes"]
)]
pub async fn popular(
    State(recipe_service): State<Arc<dyn recipe::UseCase>>,
    ValidatedQuery(queries): ValidatedQuery<PopularRecipesQueryParams>,
) -> Result<Json<PopularRecipesResponse>, ApiError> {
    info!("Getting {} most popular recipes", queries.limit);

    let recipes = recipe_service.get_popular_recipes(queries.limit).await?;

    Ok(Json(PopularRecipesResponse::from(recipes)))
}

#[utoipa::path(
    get,
    path = "/recipes/latest",
    description = "Gets the latest recipes.",
    params(LatestRecipesQueryParams),
    responses(
        (status = 200, description = "Most Recently Added Recipes", body = LatestRecipesResponse),
        (status = 422, description = "Validation Error"),
        (status = 500, description = "Internal Server Error"),
    ),
    tags = ["Recipes"]
)]
pub async fn latest(
    State(recipe_service): State<Arc<dyn recipe::UseCase>>,
    ValidatedQuery(queries): ValidatedQuery<LatestRecipesQueryParams>,
) -> Result<Json<LatestRecipesResponse>, ApiError> {
    info!("Getting {} latest recipes", queries.limit);

    let recipes = recipe_service.get_latest_recipes(queries.limit).await?;

    Ok(Json(LatestRecipesResponse::from(recipes)))
}

impl From<Vec<RecipePreview>> for PopularRecipesResponse {
    fn from(recipe_previews: Vec<RecipePreview>) -> Self {
        Self {
            recipes: recipe_previews.into_iter().map(|item| RecipePreviewItem::from(item)).collect(),
        }
    }
}

impl From<Vec<RecipePreview>> for LatestRecipesResponse {
    fn from(recipe_previews: Vec<RecipePreview>) -> Self {
        Self {
            recipes: recipe_previews.into_iter().map(|item| RecipePreviewItem::from(item)).collect(),
        }
    }
}