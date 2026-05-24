use crate::{
    BusinessState,
    prelude::*,
    features::*,
};

pub async fn init(pool: sqlx::PgPool) -> Result<BusinessState, anyhow::Error> {
    let pg_base_adapter = Arc::new(base::PostgresAdapter::new(pool));
    let base_service = base::Service::new(pg_base_adapter);

    Ok(BusinessState {
        base: Arc::new(base_service),
    })
}