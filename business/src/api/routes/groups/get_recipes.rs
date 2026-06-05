use std::collections::HashMap;

use platform::api::extractors::AuthenticatedUser;

use crate::{
    prelude::*,
    api::*,
    features::group,
};
use super::dtos::*;
use super::super::recipes::dtos::RecipePreviewItem;

#[utoipa::path(
    get,
    path = "/groups/{group_id}/recipes",
    description = "Gets the recipes within a specific group. Only the group owner can access this.",
    params(GroupIdPathParams, GroupRecipesQueryParams),
    responses(
        (status = 200, description = "Recipes in the group", body = GroupRecipesResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Group not found"),
        (status = 422, description = "Validation Error"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Groups"]
)]
pub async fn get_recipes(
    auth: AuthenticatedUser,
    State(state): State<BusinessState>,
    ValidatedPath(params): ValidatedPath<GroupIdPathParams>,
    ValidatedQuery(queries): ValidatedQuery<GroupRecipesQueryParams>,
) -> Result<Json<GroupRecipesResponse>, ApiError> {
    let user_id = auth.user_id;

    let group_id = Uuid::from_str(&params.group_id)
        .map_err(|_| {
            let mut errors = HashMap::new();
            errors.insert("group_id".to_string(), vec![format!("Invalid UUID format '{}'", params.group_id)]);
            ApiError::Validation(errors)
        })?;

    info!(
        "Getting {} recipes from group '{}' with offset {} for user '{}'",
        queries.recipes_limit, group_id, queries.offset, user_id,
    );

    let page = state.group
        .get_group_recipes(&user_id, &group_id, queries.recipes_limit, queries.offset)
        .await?;

    Ok(Json(GroupRecipesResponse::from(page)))
}

impl From<group::GroupRecipesPage> for GroupRecipesResponse {
    fn from(page: group::GroupRecipesPage) -> Self {
        Self {
            recipes: page.items.into_iter()
                .map(|recipe| RecipePreviewItem::from(recipe))
                .collect(),
            meta: GroupRecipesMeta { total_items: page.total_items },
        }
    }
}