use std::collections::HashMap;

use platform::api::extractors::AuthenticatedUser;

use crate::{
    prelude::*,
    api::*,
};
use super::dtos::*;

#[utoipa::path(
    get,
    path = "/groups/{group_id}",
    description = "Gets a single recipe group by ID with its recipes. Only the group owner can access this.",
    params(GroupIdPathParams, GetGroupQueryParams),
    responses(
        (status = 200, description = "The recipe group", body = RecipesGroupItem),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Group not found"),
        (status = 422, description = "Validation Error"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(
        ("cookie_auth" = [])
    ),
    tags = ["Groups"]
)]
pub async fn get_group(
    auth: AuthenticatedUser,
    State(state): State<BusinessState>,
    ValidatedPath(params): ValidatedPath<GroupIdPathParams>,
    ValidatedQuery(queries): ValidatedQuery<GetGroupQueryParams>,
) -> Result<Json<RecipesGroupItem>, ApiError> {
    let user_id = auth.0.id;

    let group_id = Uuid::from_str(&params.group_id)
        .map_err(|_| {
            let mut errors = HashMap::new();
            errors.insert("group_id".to_string(), vec![format!("Invalid UUID format '{}'", params.group_id)]);
            ApiError::Validation(errors)
        })?;

    info!(
        "Getting group '{}' with up to {} recipes for user '{}'",
        group_id, queries.recipes_limit, user_id,
    );

    let group = state.group
        .get_recipes_group(&group_id, &user_id, queries.recipes_limit)
        .await?
        .ok_or(ApiError::NotFound(format!("Group with ID '{}' was not found.", group_id)))?;

    Ok(Json(RecipesGroupItem::from(group)))
}
