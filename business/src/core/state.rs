use axum::extract::FromRef;

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
}