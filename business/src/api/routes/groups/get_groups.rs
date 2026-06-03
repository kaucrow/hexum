use platform::api::extractors::AuthenticatedUser;

use crate::{
    prelude::*,
    api::*,
    features::group::RecipesGroup,
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
    let user_id = auth.0.id;

    info!(
        "Getting {} groups with up to {} recipes each for user '{}'",
        queries.groups_limit, queries.recipes_limit, user_id,
    );

    let groups = state.group
        .get_user_recipe_groups(&user_id, queries.groups_limit, queries.recipes_limit)
        .await?;

    Ok(Json(UserGroupsResponse::from(groups)))
}

impl From<Vec<RecipesGroup>> for UserGroupsResponse {
    fn from(groups: Vec<RecipesGroup>) -> Self {
        Self {
            groups: groups.into_iter().map(|group| RecipesGroupItem::from(group)).collect(),
        }
    }
}

impl From<RecipesGroup> for RecipesGroupItem {
    fn from(group: RecipesGroup) -> Self {
        Self {
            group_id: group.group_id.to_string(),
            group_name: group.group_name,
            recipes: group.recipes.into_iter()
                .map(|recipe| RecipePreviewItem::from(recipe))
                .collect(),
        }
    }
}