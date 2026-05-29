use uuid::Uuid;
use async_trait::async_trait;
use thiserror::Error;

use super::*;

#[async_trait]
pub trait UseCase: Send + Sync + 'static {
    async fn search_recipe(
        &self, query: &str, limit: usize, page: usize, search_id: Option<Uuid>,
    ) -> Result<SearchResultsPage, UseCaseError>;

    async fn get_search_tag_matches(&self, query: &str, limit: usize) -> Result<Vec<String>, UseCaseError>;
}

pub struct SearchResultsPage {
    pub items: Vec<RecipeSearchResult>,
    pub total_items: usize,
    pub search_id: Uuid,
}

#[derive(Error, Debug)]
pub enum UseCaseError {
    /// Unexpected internal error.
    #[error("Recipe service: {0}.")]
    Internal(String),
}