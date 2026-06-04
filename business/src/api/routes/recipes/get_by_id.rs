use std::collections::HashMap;

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
        (status = 422, description = "Validation Error"),
        (status = 500, description = "Internal Server Error"),
    ),
    tags = ["Recipes"]
)]
pub async fn get_by_id(
    State(recipe_service): State<Arc<dyn recipe::UseCase>>,
    ValidatedPath(params): ValidatedPath<RecipePathParams>,
    OptionalUser(user): OptionalUser,
) -> Result<Json<RecipeResponse>, ApiError> {
    info!("Getting recipe with ID '{}'", params.id);

    let id = Uuid::from_str(&params.id)
        .map_err(|_| {
            let mut errors = HashMap::new();
            errors.insert("id".to_string(), vec![format!("Invalid ID format '{}'", params.id)]);
            ApiError::Validation(errors)
        })?;

    let recipe_result = recipe_service.get_recipe_by_id(id).await?;

    if let Some(recipe) = recipe_result {
        // If the user is authenticated, record this recipe in their history
        if let Some(user) = user {
            let svc = recipe_service.clone();
            let recipe_id = recipe.id;
            tokio::spawn(async move {
                if let Err(e) = svc.record_recipe_history(user.user_id, recipe_id).await {
                    error!("Failed to record recipe history: {:?}", e);
                }
            });
        }

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