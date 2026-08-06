use super::dtos::*;
use crate::{
    prelude::*,
    api::*,
    BusinessState,
};

// ─── List users ───

#[utoipa::path(
    get,
    path = "/users",
    description = "Lists users available for friending. Excludes the current user, existing friends, and users with pending friend requests (either direction). Supports pagination via `limit` and `offset` query parameters.",
    params(PaginationQuery),
    responses(
        (status = 200, description = "Paginated list of strangers", body = UserListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation Error"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Friends"]
)]
pub async fn list_users(
    auth: AuthenticatedUser,
    State(state): State<BusinessState>,
    ValidatedQuery(pagination): ValidatedQuery<PaginationQuery>,
) -> Result<Json<UserListResponse>, ApiError> {
    info!("Listing strangers for user ID '{}'", &auth.user_id);

    let limit = pagination.limit();
    let offset = pagination.offset();

    let users = state.friends
        .list_users(auth.user_id, limit, offset)
        .await?;

    let response = UserListResponse {
        users: users.into_iter().map(UserSummaryResponse::from).collect(),
        limit,
        offset,
    };

    Ok(Json(response))
}


// ─── Get friends ───

#[utoipa::path(
    get,
    path = "/friends",
    description = "Lists all accepted friends of the authenticated user.",
    responses(
        (status = 200, description = "List of friends", body = FriendListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Friends"]
)]
pub async fn list_friends(
    auth: AuthenticatedUser,
    State(state): State<BusinessState>,
) -> Result<Json<FriendListResponse>, ApiError> {
    info!("Getting friends for user ID '{}'", &auth.user_id);

    let friends = state.friends
        .get_friends(auth.user_id)
        .await?;

    let response = FriendListResponse {
        friends: friends.into_iter().map(UserSummaryResponse::from).collect(),
    };

    Ok(Json(response))
}