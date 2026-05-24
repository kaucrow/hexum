use std::sync::Arc;

use axum::extract::FromRef;

use crate::{
    Config,
    features::{user, auth},
};

#[derive(Clone, FromRef)]
pub struct PlatformState {
    pub config: Arc<Config>,
    pub auth: Arc<dyn auth::UseCase>,
    pub user: Arc<dyn user::UseCase>,
}