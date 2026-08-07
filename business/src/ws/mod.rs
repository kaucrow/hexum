pub mod connection;

use std::sync::Arc;

use axum::extract::ws::{Message as WsMessage, WebSocket};
use futures::{SinkExt, StreamExt};
use uuid::Uuid;
use tracing::info;

use crate::features::messages;
use crate::ws::connection::ConnectionManager;

/// Spawns the WebSocket send/receive loops for a conversation between
/// `user_id` (authenticated) and `friend_id` (path param).
pub async fn handle_socket(
    socket: WebSocket,
    user_id: Uuid,
    friend_id: Uuid,
    messages_service: Arc<dyn messages::UseCase>,
    connection_manager: Arc<ConnectionManager>,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Subscribe to this conversation's broadcast channel
    let mut broadcast_rx = connection_manager.subscribe(user_id, friend_id).await;

    // ─── Send loop: forward broadcast messages to this WebSocket client ───
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            if ws_sender
                .send(WsMessage::Text(msg.into()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    // ─── Receive loop: read client messages, persist, broadcast ───
    let recv_svc = messages_service.clone();
    let recv_cm = connection_manager.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            match msg {
                WsMessage::Text(text) => {
                    let content = match serde_json::from_str::<serde_json::Value>(&text) {
                        Ok(v) => v
                            .get("content")
                            .and_then(|c| c.as_str())
                            .unwrap_or("")
                            .to_string(),
                        Err(_) => continue,
                    };

                    if content.is_empty() {
                        continue;
                    }

                    let message = match recv_svc
                        .send_message(user_id, friend_id, &content)
                        .await
                    {
                        Ok(m) => m,
                        Err(_) => continue,
                    };

                    let outbound = serde_json::json!({
                        "type": "message",
                        "id": message.id.to_string(),
                        "sender_id": message.sender_id.to_string(),
                        "content": message.content,
                        "created_at": message.created_at.to_rfc3339(),
                    })
                    .to_string();

                    recv_cm
                        .broadcast(user_id, friend_id, outbound)
                        .await;

                    info!(
                        "Message from '{}' to '{}' processed",
                        user_id, friend_id
                    );
                }
                WsMessage::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
        },
        _ = &mut recv_task => {
            send_task.abort();
        },
    }
}
