use platform::api::extractors::AuthenticatedUser;

use crate::{
    prelude::*,
    api::*,
    features::group::{self, RecipesGroup},
};
use super::dtos::*;
use super::super::recipes::dtos::RecipePreviewItem;

#[utoipa::path(
    get,
    path = "/groups",
    description = "Gets the authenticated user's recipe groups.",
    params(UserGroupsQueryParams),
    responses(
        (status = 200, description = "User's recipe groups", body = UserGroupsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation Error"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Groups"]
)]
pub async fn get_groups(
    auth: AuthenticatedUser,
    State(state): State<BusinessState>,
    ValidatedQuery(queries): ValidatedQuery<UserGroupsQueryParams>,
) -> Result<Json<UserGroupsResponse>, ApiError> {
    let user_id = auth.user_id;

    info!(
        "Getting {} groups with offset {} with up to {} recipes each for user '{}'",
        queries.groups_limit, queries.groups_offset, queries.recipes_limit, user_id,
    );

    let page = state.group
        .get_user_recipe_groups(
            &user_id, queries.groups_limit, queries.groups_offset, queries.recipes_limit
        )
        .await?;

    Ok(Json(UserGroupsResponse::from(page)))
}

impl From<RecipesGroup> for RecipesGroupItem {
    fn from(group: RecipesGroup) -> Self {
        Self {
            group_id: group.group_id.to_string(),
            group_name: group.group_name,
            recipes: group.recipes.into_iter()
                .map(|recipe| RecipePreviewItem::from(recipe))
                .collect(),
            total_recipes: group.total_recipes,
        }
    }
}

impl From<group::UserRecipeGroupsPage> for UserGroupsResponse {
    fn from(page: group::UserRecipeGroupsPage) -> Self {
        Self {
            groups: page.groups.into_iter().map(|group| RecipesGroupItem::from(group)).collect(),
            meta: UserGroupsMeta { total_items: page.total_groups },
        }
    }
}