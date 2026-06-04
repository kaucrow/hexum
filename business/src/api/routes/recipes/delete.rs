use crate::{
    prelude::*,
    api::*,
    features::recipe,
};
use super::dtos::*;

#[utoipa::path(
    delete,
    path = "/recipes/{id}",
    description = "Deletes a recipe by its UUID. Only the creator of the recipe can delete it.",
    params(RecipePathParams),
    responses(
        (status = 200, description = "Recipe deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Recipe not found"),
        (status = 422, description = "Validation Error"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Recipes"]
)]
pub async fn delete(
    auth: AuthenticatedUser,
    State(recipe_service): State<Arc<dyn recipe::UseCase>>,
    ValidatedPath(params): ValidatedPath<RecipePathParams>,
) -> Result<StatusCode, ApiError> {
    let user_id = auth.user_id;
    let recipe_id_str = params.id;

    info!(
        "Deleting recipe with ID '{}' by user '{}'",
        recipe_id_str, user_id,
    );

    let recipe_id = Uuid::from_str(&recipe_id_str)
        .map_err(|_| {
            let mut errors = HashMap::new();
            errors.insert("id".to_string(), vec![format!("Invalid UUID format '{}'", recipe_id_str)]);
            ApiError::Validation(errors)
        })?;

    let _ = recipe_service
        .delete_recipe(&recipe_id, &user_id)
        .await?
        .ok_or(ApiError::NotFound(format!("Recipe with ID '{}' was not found or does not belong to you.", recipe_id)))?;

    Ok(StatusCode::OK)
}