use crate::{
    prelude::*,
    api::*,
    features::recipe::{
        self, Recipe,
    }
};
use super::dtos::*;

#[utoipa::path(
    get,
    path = "/recipes/{id}",
    description = "Gets a recipe's full details by its UUID.",
    params(RecipePathParams),
    responses(
        (status = 200, description = "Recipe details", body = RecipeResponse),
        (status = 404, description = "Recipe not found"),
        (status = 500, description = "Internal Server Error"),
    ),
    tags = ["Recipes"]
)]
pub async fn get_by_id(
    State(recipe_service): State<Arc<dyn recipe::UseCase>>,
    ValidatedPath(params): ValidatedPath<RecipePathParams>,
) -> Result<Json<RecipeResponse>, ApiError> {
    info!("Getting recipe with ID '{}'", params.id);

    let id = Uuid::from_str(&params.id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid ID format '{}'", params.id)))?;

    let recipe_result = recipe_service.get_recipe_by_id(id).await?;

    if let Some(recipe) = recipe_result {
        Ok(Json(RecipeResponse::from(recipe)))
    } else {
        Err(ApiError::NotFound(format!("Failed to find recipe with ID '{}'", params.id)))
    }
}

impl From<Recipe> for RecipeResponse {
    fn from(recipe: Recipe) -> Self {
        Self {
            id: recipe.id.to_string(),
            origin: recipe.origin.to_string(),
            name: recipe.name,
            description: recipe.description,
            tags: recipe.tags,
            instructions: recipe.instructions,
            ingredients: recipe.ingredients,
            thumbnail_url: recipe.thumbnail_url,
            video_url: recipe.video_url,
        }
    }
}