use crate::{
    BusinessState,
    get_config,
    prelude::*,
    features::*,
};

pub async fn init(
    pool: sqlx::PgPool,
) -> Result<BusinessState, anyhow::Error> {
    let config = Arc::new(get_config()?);

    // ─── Base ─────────────────────────────────────────────────────────
    let pg_base_adapter = Arc::new(base::PostgresAdapter::new(pool.clone()));
    let base_service = Arc::new(base::Service::new(pg_base_adapter));

    Ok(BusinessState {
        config,
        base: base_service,
    })
}