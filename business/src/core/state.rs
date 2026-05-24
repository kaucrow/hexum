use std::sync::Arc;

use axum::extract::FromRef;

use crate::features::base;

#[derive(Clone, FromRef)]
pub struct BusinessState {
    pub base: Arc<dyn base::UseCase>,
}