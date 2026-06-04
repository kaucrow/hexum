use platform::api::extractors::AuthenticatedUser;

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
    path = "/recipes/history",
    description = "Gets the latest recipes the authenticated user has viewed.",
    params(RecipeHistoryQueryParams),
    responses(
        (status = 200, description = "The user's most recently viewed recipes", body = RecipeHistoryResponse),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation Error"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Recipes"]
)]
pub async fn history(
    auth: AuthenticatedUser,
    State(recipe_service): State<Arc<dyn recipe::UseCase>>,
    ValidatedQuery(queries): ValidatedQuery<RecipeHistoryQueryParams>,
) -> Result<Json<RecipeHistoryResponse>, ApiError> {
    let user_id = auth.user_id;

    info!(
        "Getting recipe history for user '{}' (limit: {}, offset: {})",
        &user_id, queries.limit, queries.offset,
    );

    let recipes = recipe_service
        .get_latest_recipe_history(&user_id, queries.limit, queries.offset)
        .await?;

    Ok(Json(RecipeHistoryResponse::from(recipes)))
}

impl From<Vec<RecipePreview>> for RecipeHistoryResponse {
    fn from(recipe_previews: Vec<RecipePreview>) -> Self {
        Self {
            recipes: recipe_previews.into_iter().map(|item| RecipePreviewItem::from(item)).collect(),
        }
    }
}