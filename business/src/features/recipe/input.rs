use std::collections::BTreeMap;
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

    async fn get_recipe_by_id(&self, id: &Uuid) -> Result<Option<Recipe>, UseCaseError>;

    async fn get_popular_recipes(&self, limit: usize) -> Result<Vec<RecipePreview>, UseCaseError>;

    async fn get_latest_recipes(&self, limit: usize) -> Result<Vec<RecipePreview>, UseCaseError>;

    async fn get_search_tag_matches(&self, query: &str, limit: usize) -> Result<Vec<String>, UseCaseError>;

    async fn get_top_tags_recipes(&self, tags_limit: usize, recipes_limit: usize) -> Result<Vec<TagRecipes>, UseCaseError>;

    async fn get_latest_recipe_history(&self, user_id: &Uuid, limit: usize, offset: usize) -> Result<Vec<RecipePreview>, UseCaseError>;

    async fn get_recipes_created_by_user(&self, user_id: &Uuid, limit: usize, offset: usize) -> Result<UserCreatedRecipesPage, UseCaseError>;

    // ─── Commands ───
    async fn create_recipe(&self, input: CreateRecipeInput) -> Result<Recipe, UseCaseError>;

    async fn delete_recipe(&self, id: &Uuid, user_id: &Uuid) -> Result<Option<Uuid>, UseCaseError>;

    async fn record_recipe_history(&self, user_id: &Uuid, recipe_id: &Uuid) -> Result<(), UseCaseError>;
}

pub struct SearchResultsPage {
    pub items: Vec<RecipePreview>,
    pub total_items: usize,
    pub search_id: Uuid,
}

pub struct UserCreatedRecipesPage {
    pub items: Vec<RecipePreview>,
    pub total_items: usize,
}

pub struct CreateRecipeInput {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub ingredients: BTreeMap<String, String>,
    pub instructions: String,
    pub thumbnail_url: Option<String>,
    pub created_by: Uuid,
}

#[derive(Error, Debug)]
pub enum UseCaseError {
    /// Neither a search query nor tags were provided.
    #[error("At least one of 'query' or 'tags' must be provided.")]
    MissingSearchParams,

    /// Recipe name is empty.
    #[error("Recipe name cannot be empty.")]
    EmptyName,

    /// Recipe instructions are empty.
    #[error("Recipe instructions cannot be empty.")]
    EmptyInstructions,

    /// Unexpected internal error.
    #[error("Recipe service: {0}.")]
    Internal(String),
}