use crate::prelude::*;
use crate::postgres::*;
use super::*;

#[derive(Clone)]
pub struct PostgresAdapter {
    pub pool: PgPool,
}

impl PostgresAdapter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Port for PostgresAdapter {
    async fn insert_message(
        &self,
        id: Uuid,
        sender_id: Uuid,
        receiver_id: Uuid,
        content: &str,
    ) -> Result<Message, PortError> {
        let res: Result<_, LocalError> = async {
            let row = sqlx::query_as::<_, MessageDbRow>(
                sql(&QUERIES.messages.insert),
            )
            .bind(id)
            .bind(sender_id)
            .bind(receiver_id)
            .bind(content)
            .fetch_one(&self.pool)
            .await?;

            Ok(Message::from(row))
        }
        .await;

        res.map_err(Into::into)
    }

    async fn get_conversation(
        &self,
        user_id: Uuid,
        friend_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Message>, PortError> {
        let res: Result<_, LocalError> = async {
            let rows = sqlx::query_as::<_, MessageDbRow>(
                sql(&QUERIES.messages.get_conversation),
            )
            .bind(user_id)
            .bind(friend_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

            let messages = rows.into_iter().map(Message::from).collect();
            Ok(messages)
        }
        .await;

        res.map_err(Into::into)
    }

}

// ─── Error types ───

#[derive(Error, Debug)]
pub enum LocalError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

impl From<LocalError> for PortError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::Sqlx(e) => PortError::Internal(e.to_string()),
        }
    }
}

// ─── Database rows ───

#[derive(sqlx::FromRow)]
pub struct MessageDbRow {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub content: String,
    pub created_at: sqlx::types::chrono::DateTime<chrono::Utc>,
}

impl From<MessageDbRow> for Message {
    fn from(row: MessageDbRow) -> Self {
        Self {
            id: row.id,
            sender_id: row.sender_id,
            receiver_id: row.receiver_id,
            content: row.content,
            created_at: row.created_at,
        }
    }
}