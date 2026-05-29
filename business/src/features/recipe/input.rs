use async_trait::async_trait;
use thiserror::Error;

use super::*;

#[async_trait]
pub trait UseCase: Send + Sync + 'static {
    async fn search_recipe_by_name(&self, name: &str, page: usize) -> Result<RecipeSearchResult, UseCaseError>;
}

pub struct RecipeSearchResult {
    pub items: Vec<domain::RecipeSearchResult>,
    pub total_items: usize,
}

#[derive(Error, Debug)]
pub enum UseCaseError {
    /// Unexpected internal error.
    #[error("Recipe service: {0}.")]
    Internal(String),
}