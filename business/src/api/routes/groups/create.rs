use platform::api::extractors::AuthenticatedUser;

use crate::{
    prelude::*,
    api::*,
};
use super::dtos::*;

#[utoipa::path(
    post,
    path = "/groups",
    description = "Creates a new recipe group for the authenticated user.",
    request_body(content = CreateGroupRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "The created group's ID", body = CreateGroupResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation Error"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Groups"]
)]
pub async fn create(
    auth: AuthenticatedUser,
    State(state): State<BusinessState>,
    ValidatedJson(body): ValidatedJson<CreateGroupRequest>,
) -> Result<(StatusCode, Json<CreateGroupResponse>), ApiError> {
    let user_id = auth.0.id;

    info!(
        "Creating group '{}' for user '{}'",
        body.name, user_id,
    );

    let created_group_id = state.group
        .create_group(&body.name, body.description, &user_id)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateGroupResponse { id: created_group_id.to_string() }),
    ))
}