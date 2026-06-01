use std::sync::Arc;

use axum::extract::FromRef;

use platform::{Config, features::auth};

use crate::features::*;

#[derive(Clone, FromRef)]
pub struct BusinessState {
    pub config: Arc<Config>,
    pub auth: Arc<dyn auth::UseCase>,
    pub base: Arc<dyn base::UseCase>,
    pub data_ingestion: Arc<dyn data_ingestion::UseCase>,
    pub recipe: Arc<dyn recipe::UseCase>,
}