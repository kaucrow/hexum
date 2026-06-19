use std::sync::Arc;

use axum::extract::FromRef;

use crate::{
    Config,
    features::{base, search},
};

#[derive(Clone, FromRef)]
pub struct BusinessState {
    pub config: Arc<Config>,
    pub base: Arc<dyn base::UseCase>,
    pub search: Arc<dyn search::UseCase>,
}