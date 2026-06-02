use uuid::Uuid;
use async_trait::async_trait;
use thiserror::Error;

use super::*;

#[async_trait]
pub trait UseCase: Send + Sync + 'static {
    // ─── Getters ───
    async fn get_recipes_group(
        &self,
        group_id: &Uuid,
        user_id: &Uuid,
        recipes_limit: usize
    ) -> Result<Option<RecipesGroup>, UseCaseError>;

    async fn get_user_groups(
        &self,
        user_id: &Uuid,
    ) -> Result<Vec<Group>, UseCaseError>;

    async fn get_user_recipe_groups(
        &self,
        user_id: &Uuid,
        groups_limit: usize,
        recipes_limit: usize
    ) -> Result<Vec<RecipesGroup>, UseCaseError>;

    async fn get_group_recipes(
        &self,
        user_id: &Uuid,
        group_id: &Uuid,
        recipes_limit: usize,
        offset: usize
    ) -> Result<Vec<RecipePreview>, UseCaseError>;

    // ─── Commands ───
    async fn create_group(&self, name: &str, description: Option<String>, user_id: &Uuid) -> Result<Uuid, UseCaseError>;

    async fn delete_group(&self, group_id: &Uuid, user_id: &Uuid) -> Result<Option<Uuid>, UseCaseError>;

    async fn add_recipe_to_group(&self, user_id: &Uuid, group_id: &Uuid, recipe_id: &Uuid) -> Result<(), UseCaseError>;
    async fn delete_recipe_from_group(&self, user_id: &Uuid, group_id: &Uuid, recipe_id: &Uuid) -> Result<(), UseCaseError>;
}

#[derive(Error, Debug)]
pub enum UseCaseError {
    /// Group was not found.
    #[error("Group not found.")]
    GroupNotFound,

    /// User is not the owner of this group.
    #[error("You are not the owner of this group.")]
    NotGroupOwner,

    /// Group name is empty.
    #[error("Group name cannot be empty.")]
    EmptyName,

    /// Unexpected internal error.
    #[error("Group service: {0}.")]
    Internal(String),
}