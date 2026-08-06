use super::dtos::*;
use crate::{
    prelude::*,
    api::*,
    BusinessState,
};

// ─── Send friend request ───

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

// ─── Get sent requests ───

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

// ─── Get received requests ───

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

// ─── Accept friend request ───

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

// ─── Reject friend request ───

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