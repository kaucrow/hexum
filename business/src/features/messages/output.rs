use chrono::{DateTime, Utc};

use crate::prelude::*;

// ─── Domain types ───

#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

// ─── Output port ───

#[async_trait]
pub trait Port: Send + Sync + 'static {
    async fn insert_message(
        &self,
        id: Uuid,
        sender_id: Uuid,
        receiver_id: Uuid,
        content: &str,
    ) -> Result<Message, PortError>;

    async fn get_conversation(
        &self,
        user_id: Uuid,
        friend_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Message>, PortError>;

}

// ─── Error types ───

#[derive(Error, Debug)]
pub enum PortError {
    #[error("Messages repository: {0}")]
    Internal(String),
}