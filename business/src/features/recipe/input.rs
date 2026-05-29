use uuid::Uuid;
use async_trait::async_trait;
use thiserror::Error;

use super::*;

#[async_trait]
pub trait UseCase: Send + Sync + 'static {
    // ─── Getters ───
    async fn search_recipe(
        &self,
        query: Option<&str>,
        tags: Option<&[String]>,
        limit: usize,
        page: usize,
        search_id: Option<Uuid>,
    ) -> Result<SearchResultsPage, UseCaseError>;

    async fn get_recipe_by_id(&self, id: Uuid) -> Result<Option<Recipe>, UseCaseError>;

    async fn get_search_tag_matches(&self, query: &str, limit: usize) -> Result<Vec<String>, UseCaseError>;
}

pub struct SearchResultsPage {
    pub items: Vec<RecipeSearchResult>,
    pub total_items: usize,
    pub search_id: Uuid,
}

#[derive(Error, Debug)]
pub enum UseCaseError {
    /// Neither a search query nor tags were provided.
    #[error("At least one of 'query' or 'tags' must be provided.")]
    MissingSearchParams,

    /// Unexpected internal error.
    #[error("Recipe service: {0}.")]
    Internal(String),
}