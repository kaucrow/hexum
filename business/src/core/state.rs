use std::sync::Arc;

use axum::extract::FromRef;

use crate::features::{base, friends};
use platform::features::auth;

#[derive(Clone, FromRef)]
pub struct BusinessState {
    pub auth: Arc<dyn auth::UseCase>,
    pub base: Arc<dyn base::UseCase>,
    pub friends: Arc<dyn friends::UseCase>,
}