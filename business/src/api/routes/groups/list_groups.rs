use platform::api::extractors::AuthenticatedUser;

use crate::{
    prelude::*,
    api::*,
    features::group::Group,
};
use super::dtos::*;

#[utoipa::path(
    get,
    path = "/groups/list",
    description = "Gets the names and IDs of the authenticated user's created groups.",
    responses(
        (status = 200, description = "User's group names and IDs", body = UserGroupsListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation Error"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Groups"]
)]
pub async fn list_groups(
    auth: AuthenticatedUser,
    State(state): State<BusinessState>,
) -> Result<Json<UserGroupsListResponse>, ApiError> {
    let user_id = auth.user_id;

    info!("Getting all group names/IDs for user '{}'", user_id);

    let groups = state.group
        .get_user_groups(&user_id)
        .await?;

    Ok(Json(UserGroupsListResponse::from(groups)))
}

impl From<Vec<Group>> for UserGroupsListResponse {
    fn from(groups: Vec<Group>) -> Self {
        Self {
            groups: groups.into_iter().map(|group| UserGroupItem::from(group)).collect(),
        }
    }
}

impl From<Group> for UserGroupItem {
    fn from(group: Group) -> Self {
        Self {
            group_id: group.id.to_string(),
            group_name: group.name,
        }
    }
}