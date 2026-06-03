use platform::api::extractors::AuthenticatedUser;

use crate::{
    prelude::*,
    api::*,
    features::group,
};
use super::dtos::*;

#[utoipa::path(
    delete,
    path = "/groups/{group_id}",
    description = "Deletes a group and the recipes in it that belong to the authenticated user. Only the group owner can access this.",
    params(DeleteGroupPathParams),
    responses(
        (status = 200, description = "The deleted group's ID", body = DeleteGroupResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation Error"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Groups"]
)]
pub async fn delete(
    auth: AuthenticatedUser,
    State(group_service): State<Arc<dyn group::UseCase>>,
    ValidatedPath(params): ValidatedPath<DeleteGroupPathParams>,
) -> Result<Json<DeleteGroupResponse>, ApiError> {
    let user_id = auth.0.id;

    info!(
        "Deleting group with ID '{}' from user '{}'",
        params.id, user_id,
    );

    let group_id = Uuid::from_str(&params.id)
        .map_err(|_| {
            let mut errors = HashMap::new();
            errors.insert("group_id".to_string(), vec![format!("Invalid UUID format '{}'", params.id)]);
            ApiError::Validation(errors)
        })?;

    let deleted_group_id = group_service
        .delete_group(&group_id, &user_id)
        .await?
        .ok_or(ApiError::NotFound(format!("Group with ID '{}' was not found.", group_id)))?;

    Ok(Json(DeleteGroupResponse { id: deleted_group_id.to_string() }))
}