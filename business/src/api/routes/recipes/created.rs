use crate::{
    prelude::*,
    api::*,
    features::recipe,
};
use super::dtos::*;

#[utoipa::path(
    get,
    path = "/recipes/created",
    description = "Gets the recipes created by the authenticated user.",
    params(UserCreatedRecipesQueryParams),
    responses(
        (status = 200, description = "The user's created recipes", body = UserCreatedRecipesResponse),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation Error"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Recipes"]
)]
pub async fn created(
    auth: AuthenticatedUser,
    State(recipe_service): State<Arc<dyn recipe::UseCase>>,
    ValidatedQuery(queries): ValidatedQuery<UserCreatedRecipesQueryParams>,
) -> Result<Json<UserCreatedRecipesResponse>, ApiError> {
    let user_id = auth.user_id;

    info!(
        "Getting {} recipes created by user '{}' with offset {}",
        queries.limit, &user_id, queries.offset,
    );

    let page = recipe_service
        .get_recipes_created_by_user(&user_id, queries.limit, queries.offset)
        .await?;

    Ok(Json(UserCreatedRecipesResponse::from(page)))
}

impl From<recipe::UserCreatedRecipesPage> for UserCreatedRecipesResponse {
    fn from(page: recipe::UserCreatedRecipesPage) -> Self {
        Self {
            recipes: page.items.into_iter().map(|item| RecipePreviewItem::from(item)).collect(),
            meta: UserCreatedRecipesMeta {
                total_items: page.total_items,
            },
        }
    }
}