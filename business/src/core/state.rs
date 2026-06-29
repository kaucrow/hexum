use axum::extract::FromRef;

use ::platform as platform_crate;

use crate::{
    prelude::*,
    Config,
    features::*,
};

#[derive(Clone, FromRef)]
pub struct BusinessState {
    pub config: Arc<Config>,
    pub base: Arc<dyn base::UseCase>,
    pub search: Arc<dyn search::UseCase>,
    pub platform: Arc<dyn platform::UseCase>,
    pub game: Arc<dyn game::UseCase>,
    pub auth: Arc<dyn platform_crate::features::auth::UseCase>,
    pub critic: Arc<dyn crate::features::critic::UseCase>,
    pub review: Arc<dyn crate::features::review::UseCase>,
}