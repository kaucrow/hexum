use std::collections::HashMap;

use tokio::sync::{broadcast, RwLock};

use crate::prelude::*;

/// A normalized conversation key: (min(user_a, user_b), max(user_a, user_b)).
fn conversation_key(a: Uuid, b: Uuid) -> (Uuid, Uuid) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

/// Shared state for real-time messaging. Each conversation gets one broadcast
/// channel. When a user connects via WebSocket, they subscribe. When a message
/// is sent, it's broadcast to all subscribers of that conversation.
#[derive(Clone)]
pub struct ConnectionManager {
    channels: Arc<RwLock<HashMap<(Uuid, Uuid), broadcast::Sender<String>>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Subscribe to a conversation. Returns a receiver that yields JSON message
    /// strings. The sender is created lazily if this is the first subscriber.
    pub async fn subscribe(&self, user_a: Uuid, user_b: Uuid) -> broadcast::Receiver<String> {
        let key = conversation_key(user_a, user_b);
        let mut channels = self.channels.write().await;

        let sender = channels
            .entry(key)
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(256);
                tx
            });

        sender.subscribe()
    }

    /// Broadcast a JSON message string to all subscribers of a conversation.
    /// Returns the number of subscribers that received the message.
    pub async fn broadcast(&self, user_a: Uuid, user_b: Uuid, message: String) -> usize {
        let key = conversation_key(user_a, user_b);
        let channels = self.channels.read().await;

        if let Some(sender) = channels.get(&key) {
            sender.send(message).unwrap_or(0)
        } else {
            0
        }
    }
}