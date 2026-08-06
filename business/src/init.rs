use crate::{
    BusinessState,
    prelude::*,
    features::*,
};
use platform::features::auth;

pub async fn init(
    pool: sqlx::PgPool,
    auth_service: Arc<dyn auth::UseCase>,
) -> Result<BusinessState, anyhow::Error> {
    let pg_base_adapter = Arc::new(base::PostgresAdapter::new(pool.clone()));
    let base_service = base::Service::new(pg_base_adapter);

    let pg_friends_adapter = Arc::new(friends::PostgresAdapter::new(pool));
    let friends_service = friends::Service::new(pg_friends_adapter);

    Ok(BusinessState {
        auth: auth_service,
        base: Arc::new(base_service),
        friends: Arc::new(friends_service),
    })
}