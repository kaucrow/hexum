use async_trait::async_trait;
use thiserror::Error;

use super::{Recipe, RecipeSearchResult};

#[async_trait]
pub trait UseCase: Send + Sync + 'static {
    async fn search_by_name(&self, name: &str, page: usize) -> Result<Vec<RecipeSearchResult>, UseCaseError>;
}

#[derive(Error, Debug)]
pub enum UseCaseError {
    /// Unexpected internal error.
    #[error("Recipe service: {0}.")]
    Internal(String),
}