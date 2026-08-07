use axum::extract::ws::{WebSocket, WebSocketUpgrade};

use crate::{
    prelude::*,
    api::*,
    BusinessState,
    ws,
};

// ─── WebSocket upgrade ───

#[utoipa::path(
    get,
    path = "/ws/friends/{friend_id}",
    description = "Upgrades to a WebSocket connection for real-time messaging with the specified friend. Send JSON text frames with `{\"content\": \"...\"}`. Receive `{\"type\": \"message\", ...}` frames.",
    params(
        ("friend_id" = Uuid, Path, description = "The friend's user ID"),
    ),
    responses(
        (status = 101, description = "WebSocket upgrade successful"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not friends with this user"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Messages"]
)]
pub async fn ws_handler(
    auth: AuthenticatedUser,
    State(state): State<BusinessState>,
    Path(friend_id): Path<Uuid>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, ApiError> {
    info!(
        "WebSocket upgrade for user '{}' with friend '{}'",
        &auth.user_id, &friend_id
    );

    // Verify the users are friends
    let are_friends = state.friends
        .get_friends(auth.user_id)
        .await?
        .iter()
        .any(|f| f.id == friend_id);

    if !are_friends {
        return Err(ApiError::Forbidden("You are not friends with this user.".to_string()));
    }

    let messages_svc = state.messages.clone();
    let cm = state.connection_manager.clone();
    let upload_dir = state.upload_dir.clone();
    let user_id = auth.user_id;

    Ok(ws.on_upgrade(move |socket: WebSocket| {
        ws::handle_socket(socket, user_id, friend_id, messages_svc, cm, upload_dir)
    }))
}