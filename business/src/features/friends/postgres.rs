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
    async fn list_strangers(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserSummary>, PortError> {
        let res: Result<_, LocalError> = async {
            let rows = sqlx::query_as::<_, UserSummaryDbRow>(
                sql(&QUERIES.friends.list_strangers),
            )
            .bind(user_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

            let users = rows.into_iter().map(UserSummary::from).collect();
            Ok(users)
        }
        .await;

        res.map_err(Into::into)
    }

    async fn create_request(
        &self,
        id: Uuid,
        sender_id: Uuid,
        receiver_id: Uuid,
    ) -> Result<FriendRequest, PortError> {
        let res: Result<_, LocalError> = async {
            let row = sqlx::query_as::<_, FriendRequestDbRow>(
                sql(&QUERIES.friends.insert_request),
            )
            .bind(id)
            .bind(sender_id)
            .bind(receiver_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                if is_unique_violation(&e) {
                    LocalError::Logic(PortError::Conflict(
                        ConflictError::PendingRequestExists,
                    ))
                } else {
                    LocalError::Sqlx(e)
                }
            })?;

            Ok(FriendRequest::from(row))
        }
        .await;

        res.map_err(Into::into)
    }

    async fn get_sent_requests(
        &self,
        sender_id: Uuid,
    ) -> Result<Vec<FriendRequest>, PortError> {
        let res: Result<_, LocalError> = async {
            let rows = sqlx::query_as::<_, FriendRequestDbRow>(
                sql(&QUERIES.friends.get_sent_requests),
            )
            .bind(sender_id)
            .fetch_all(&self.pool)
            .await?;

            let requests = rows.into_iter().map(FriendRequest::from).collect();
            Ok(requests)
        }
        .await;

        res.map_err(Into::into)
    }

    async fn get_received_requests(
        &self,
        receiver_id: Uuid,
    ) -> Result<Vec<FriendRequest>, PortError> {
        let res: Result<_, LocalError> = async {
            let rows = sqlx::query_as::<_, FriendRequestDbRow>(
                sql(&QUERIES.friends.get_received_requests),
            )
            .bind(receiver_id)
            .fetch_all(&self.pool)
            .await?;

            let requests = rows.into_iter().map(FriendRequest::from).collect();
            Ok(requests)
        }
        .await;

        res.map_err(Into::into)
    }

    async fn accept_request(
        &self,
        request_id: Uuid,
        receiver_id: Uuid,
    ) -> Result<Option<FriendRequest>, PortError> {
        let res: Result<_, LocalError> = async {
            let row = sqlx::query_as::<_, FriendRequestDbRow>(
                sql(&QUERIES.friends.accept_request),
            )
            .bind(request_id)
            .bind(receiver_id)
            .fetch_optional(&self.pool)
            .await?
            .map(FriendRequest::from);

            Ok(row)
        }
        .await;

        res.map_err(Into::into)
    }

    async fn reject_request(
        &self,
        request_id: Uuid,
        receiver_id: Uuid,
    ) -> Result<Option<FriendRequest>, PortError> {
        let res: Result<_, LocalError> = async {
            let row = sqlx::query_as::<_, FriendRequestDbRow>(
                sql(&QUERIES.friends.reject_request),
            )
            .bind(request_id)
            .bind(receiver_id)
            .fetch_optional(&self.pool)
            .await?
            .map(FriendRequest::from);

            Ok(row)
        }
        .await;

        res.map_err(Into::into)
    }

    async fn get_friends(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<UserSummary>, PortError> {
        let res: Result<_, LocalError> = async {
            let rows = sqlx::query_as::<_, UserSummaryDbRow>(
                sql(&QUERIES.friends.get_friends),
            )
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;

            let users = rows.into_iter().map(UserSummary::from).collect();
            Ok(users)
        }
        .await;

        res.map_err(Into::into)
    }
}

// ─── Error types ───

#[derive(Error, Debug)]
pub enum LocalError {
    #[error("{0}")]
    Logic(PortError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

impl From<LocalError> for PortError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::Logic(e) => e,
            LocalError::Sqlx(e) => {
                if is_unique_violation(&e) {
                    PortError::Conflict(ConflictError::PendingRequestExists)
                } else {
                    PortError::Internal(e.to_string())
                }
            }
        }
    }
}

// ─── Helper ───

fn is_unique_violation(e: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = e {
        db_err.code().as_deref() == Some("23505")
    } else {
        false
    }
}

// ─── Database rows ───

#[derive(sqlx::FromRow)]
pub struct UserSummaryDbRow {
    pub id: Uuid,
    pub username: String,
    pub profile_picture_url: Option<String>,
    pub last_msg_id: Option<Uuid>,
    pub last_msg_sender_id: Option<Uuid>,
    pub last_msg_content: Option<String>,
    pub last_msg_created_at: Option<sqlx::types::chrono::DateTime<chrono::Utc>>,
}

impl From<UserSummaryDbRow> for UserSummary {
    fn from(row: UserSummaryDbRow) -> Self {
        let last_message = match (
            row.last_msg_id,
            row.last_msg_sender_id,
            row.last_msg_content,
            row.last_msg_created_at,
        ) {
            (Some(id), Some(sender_id), Some(content), Some(created_at)) => {
                Some(LastMessage {
                    id,
                    sender_id,
                    content,
                    created_at,
                })
            }
            _ => None,
        };

        Self {
            id: row.id,
            username: row.username,
            profile_picture_url: row.profile_picture_url,
            last_message,
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct FriendRequestDbRow {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub sender_username: String,
    pub receiver_id: Uuid,
    pub receiver_username: String,
    pub status: String,
    pub created_at: sqlx::types::chrono::DateTime<chrono::Utc>,
    pub updated_at: sqlx::types::chrono::DateTime<chrono::Utc>,
}

impl From<FriendRequestDbRow> for FriendRequest {
    fn from(row: FriendRequestDbRow) -> Self {
        let status = match row.status.as_str() {
            "accepted" => FriendRequestStatus::Accepted,
            "rejected" => FriendRequestStatus::Rejected,
            _ => FriendRequestStatus::Pending,
        };

        Self {
            id: row.id,
            sender_id: row.sender_id,
            sender_username: row.sender_username,
            receiver_id: row.receiver_id,
            receiver_username: row.receiver_username,
            status,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}