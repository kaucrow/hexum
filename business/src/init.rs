use crate::{
    BusinessState,
    prelude::*,
    features::*,
    ws::connection::ConnectionManager,
};
use platform::features::auth;

pub async fn init(
    pool: sqlx::PgPool,
    auth_service: Arc<dyn auth::UseCase>,
    upload_dir: String,
) -> Result<BusinessState, anyhow::Error> {
    let pg_base_adapter = Arc::new(base::PostgresAdapter::new(pool.clone()));
    let base_service = base::Service::new(pg_base_adapter);

    let pg_friends_adapter = Arc::new(friends::PostgresAdapter::new(pool.clone()));
    let friends_service = friends::Service::new(pg_friends_adapter.clone());

    let pg_messages_adapter = Arc::new(messages::PostgresAdapter::new(pool));
    let messages_service = messages::Service::new(pg_messages_adapter, pg_friends_adapter);

    let connection_manager = Arc::new(ConnectionManager::new());

    Ok(BusinessState {
        auth: auth_service,
        base: Arc::new(base_service),
        friends: Arc::new(friends_service),
        messages: Arc::new(messages_service),
        connection_manager,
        upload_dir,
    })
}