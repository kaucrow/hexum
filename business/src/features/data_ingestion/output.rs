use async_trait::async_trait;
use thiserror::Error;

use crate::prelude::*;
use super::{Recipe, domain};

// ────────────────────────────────────────────────
//  Local Repository
// ────────────────────────────────────────────────
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait LocalRepository: Send + Sync + 'static {
    // ─── Setters ───
    async fn sync_recipes(&self, recipes: Vec<Recipe>) -> Result<(), LocalRepositoryError>;
}

#[derive(Error, Debug)]
pub enum LocalRepositoryError {
    #[error("User repository: {0}")]
    Internal(String),
}

// ────────────────────────────────────────────────
//  External Repository
// ────────────────────────────────────────────────
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ExternalRepository: Send + Sync + 'static {
    async fn get_recipes_by_first_letter(&self, letter: char) -> Result<Vec<domain::Recipe>, ExternalRepositoryError>;
}

#[derive(Error, Debug)]
pub enum ExternalRepositoryError {
    #[error("Recipe external repository: {0}")]
    Network(String),
    #[error("Recipe external repository: {0}")]
    Serialization(String),
    #[error("Recipe external repository: {0}")]
    Internal(String),
}