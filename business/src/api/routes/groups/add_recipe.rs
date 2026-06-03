use std::collections::HashMap;

use platform::api::extractors::AuthenticatedUser;

use crate::{
    prelude::*,
    api::*,
};
use super::dtos::*;

#[utoipa::path(
    post,
    path = "/groups/{group_id}/recipes/{recipe_id}",
    description = "Adds a recipe to a group. Only the group owner can perform this action.",
    params(GroupRecipePathParams),
    responses(
        (status = 200, description = "Recipe added to group successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Group not found"),
        (status = 422, description = "Validation Error"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Groups"]
)]
pub async fn add_recipe(
    auth: AuthenticatedUser,
    State(state): State<BusinessState>,
    ValidatedPath(params): ValidatedPath<GroupRecipePathParams>,
) -> Result<StatusCode, ApiError> {
    let user_id = auth.0.id;

    let group_id = Uuid::from_str(&params.group_id)
        .map_err(|_| {
            let mut errors = HashMap::new();
            errors.insert("group_id".to_string(), vec![format!("Invalid UUID format '{}'", params.group_id)]);
            ApiError::Validation(errors)
        })?;

    let recipe_id = Uuid::from_str(&params.recipe_id)
        .map_err(|_| {
            let mut errors = HashMap::new();
            errors.insert("recipe_id".to_string(), vec![format!("Invalid UUID format '{}'", params.recipe_id)]);
            ApiError::Validation(errors)
        })?;

    info!(
        "Adding recipe '{}' to group '{}' for user '{}'",
        recipe_id, group_id, user_id,
    );

    state.group
        .add_recipe_to_group(&user_id, &group_id, &recipe_id)
        .await?;

    Ok(StatusCode::OK)
}