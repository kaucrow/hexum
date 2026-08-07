pub mod dtos;

use uuid::Uuid;

use crate::{
    prelude::*,
    api::*,
    features::friends::{self, UserSummary, FriendRequest},
    BusinessState,
};
use self::dtos::*;

// ─── 1. List users ───

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

// ─── 2. Send friend request ───

#[utoipa::path(
    post,
    path = "/users/{user_id}/friend-request",
    description = "Sends a friend request to the specified user.",
    params(
        ("user_id" = Uuid, Path, description = "The ID of the user to send a friend request to"),
    ),
    responses(
        (status = 200, description = "Friend request sent", body = SendFriendRequestResponse),
        (status = 400, description = "Cannot friend yourself"),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "A pending request already exists"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Friends"]
)]
pub async fn send_friend_request(
    auth: AuthenticatedUser,
    State(state): State<BusinessState>,
    Path(receiver_id): Path<Uuid>,
) -> Result<Json<SendFriendRequestResponse>, ApiError> {
    info!(
        "User '{}' sending friend request to user '{}'",
        &auth.user_id, &receiver_id
    );

    let request = state.friends
        .send_friend_request(auth.user_id, receiver_id)
        .await?;

    let response = SendFriendRequestResponse {
        message: "Friend request sent successfully.".to_string(),
        request: FriendRequestResponse::from(request),
    };

    Ok(Json(response))
}

// ─── 3. Get sent requests ───

#[utoipa::path(
    get,
    path = "/friend-requests/sent",
    description = "Lists all pending friend requests sent by the authenticated user.",
    responses(
        (status = 200, description = "List of pending sent requests", body = FriendRequestListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Friends"]
)]
pub async fn get_sent_requests(
    auth: AuthenticatedUser,
    State(state): State<BusinessState>,
) -> Result<Json<FriendRequestListResponse>, ApiError> {
    info!("Getting sent requests for user ID '{}'", &auth.user_id);

    let requests = state.friends
        .get_sent_requests(auth.user_id)
        .await?;

    let response = FriendRequestListResponse {
        requests: requests.into_iter().map(FriendRequestResponse::from).collect(),
    };

    Ok(Json(response))
}

// ─── 4. Get received requests ───

#[utoipa::path(
    get,
    path = "/friend-requests/received",
    description = "Lists all pending friend requests received by the authenticated user.",
    responses(
        (status = 200, description = "List of pending received requests", body = FriendRequestListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Friends"]
)]
pub async fn get_received_requests(
    auth: AuthenticatedUser,
    State(state): State<BusinessState>,
) -> Result<Json<FriendRequestListResponse>, ApiError> {
    info!("Getting received requests for user ID '{}'", &auth.user_id);

    let requests = state.friends
        .get_received_requests(auth.user_id)
        .await?;

    let response = FriendRequestListResponse {
        requests: requests.into_iter().map(FriendRequestResponse::from).collect(),
    };

    Ok(Json(response))
}

// ─── 5. Accept friend request ───

#[utoipa::path(
    post,
    path = "/friend-requests/{request_id}/accept",
    description = "Accepts a pending friend request. Only the receiver of the request can accept it.",
    params(
        ("request_id" = Uuid, Path, description = "The ID of the friend request to accept"),
    ),
    responses(
        (status = 200, description = "Friend request accepted", body = AcceptRejectResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Request not found"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Friends"]
)]
pub async fn accept_friend_request(
    auth: AuthenticatedUser,
    State(state): State<BusinessState>,
    Path(request_id): Path<Uuid>,
) -> Result<Json<AcceptRejectResponse>, ApiError> {
    info!(
        "User '{}' accepting friend request '{}'",
        &auth.user_id, &request_id
    );

    let request = state.friends
        .accept_friend_request(request_id, auth.user_id)
        .await?;

    let response = AcceptRejectResponse {
        message: "Friend request accepted.".to_string(),
        request: FriendRequestResponse::from(request),
    };

    Ok(Json(response))
}

// ─── 6. Reject friend request ───

#[utoipa::path(
    post,
    path = "/friend-requests/{request_id}/reject",
    description = "Rejects a pending friend request. Only the receiver of the request can reject it. The sender may send a new request afterwards.",
    params(
        ("request_id" = Uuid, Path, description = "The ID of the friend request to reject"),
    ),
    responses(
        (status = 200, description = "Friend request rejected", body = AcceptRejectResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Request not found"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Friends"]
)]
pub async fn reject_friend_request(
    auth: AuthenticatedUser,
    State(state): State<BusinessState>,
    Path(request_id): Path<Uuid>,
) -> Result<Json<AcceptRejectResponse>, ApiError> {
    info!(
        "User '{}' rejecting friend request '{}'",
        &auth.user_id, &request_id
    );

    let request = state.friends
        .reject_friend_request(request_id, auth.user_id)
        .await?;

    let response = AcceptRejectResponse {
        message: "Friend request rejected.".to_string(),
        request: FriendRequestResponse::from(request),
    };

    Ok(Json(response))
}

// ─── 7. Get friends ───

#[utoipa::path(
    get,
    path = "/friends",
    description = "Lists all accepted friends of the authenticated user, including the last message exchanged.",
    responses(
        (status = 200, description = "List of friends with last message", body = FriendListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Friends"]
)]
pub async fn get_friends(
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

// ─── DTO mappings ───

impl From<UserSummary> for UserSummaryResponse {
    fn from(u: UserSummary) -> Self {
        Self {
            id: u.id.to_string(),
            username: u.username,
            profile_picture_url: u.profile_picture_url,
            last_message: u.last_message.map(LastMessageResponse::from),
        }
    }
}

impl From<FriendRequest> for FriendRequestResponse {
    fn from(r: FriendRequest) -> Self {
        Self {
            id: r.id.to_string(),
            sender_id: r.sender_id.to_string(),
            sender_username: r.sender_username,
            receiver_id: r.receiver_id.to_string(),
            receiver_username: r.receiver_username,
            status: r.status.as_str().to_string(),
            created_at: r.created_at.to_rfc3339(),
            updated_at: r.updated_at.to_rfc3339(),
        }
    }
}

// ─── Error mappings ───

impl From<friends::UseCaseError> for ApiError {
    fn from(e: friends::UseCaseError) -> Self {
        match e {
            friends::UseCaseError::CannotFriendYourself => {
                ApiError::BadRequest(e.to_string())
            }
            friends::UseCaseError::PendingRequestExists => {
                ApiError::Conflict(e.to_string())
            }
            friends::UseCaseError::RequestNotFound => {
                ApiError::NotFound(e.to_string())
            }
            friends::UseCaseError::NotReceiver => {
                ApiError::BadRequest(e.to_string())
            }
            friends::UseCaseError::Internal(msg) => {
                error!("Internal friends error: {msg}");
                ApiError::Internal("An internal error occurred".to_string())
            }
        }
    }
}
