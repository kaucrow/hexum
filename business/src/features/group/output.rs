use uuid::Uuid;
use async_trait::async_trait;
use thiserror::Error;

use crate::prelude::*;
use super::*;

// ────────────────────────────────────────────────
//  Local Repository
// ────────────────────────────────────────────────
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait LocalRepository: Send + Sync + 'static {
    // ─── Getters ───
    async fn get_group_by_id(&self, id: &Uuid) -> Result<Option<Group>, LocalRepositoryError>;

    async fn get_user_groups(
        &self,
        user_id: &Uuid,
    ) -> Result<Vec<Group>, LocalRepositoryError>;

    async fn get_user_recipe_groups(
        &self,
        user_id: &Uuid,
        groups_limit: usize,
        groups_offset: usize,
        recipes_limit: usize
    ) -> Result<(Vec<RecipesGroup>, usize), LocalRepositoryError>;

    async fn get_group_recipes(
        &self,
        group_id: &Uuid,
        recipes_limit: usize,
        offset: usize
    ) -> Result<(Vec<RecipePreview>, usize), LocalRepositoryError>;

    // ─── Commands ───
    async fn create_group(&self, group: Group) -> Result<Uuid, LocalRepositoryError>;
    async fn delete_group(&self, group_id: &Uuid, user_id: &Uuid) -> Result<Option<Uuid>, LocalRepositoryError>;

    async fn add_recipe(&self, group_id: &Uuid, recipe_id: &Uuid) -> Result<(), LocalRepositoryError>;
    async fn delete_recipe(&self, group_id: &Uuid, recipe_id: &Uuid) -> Result<(), LocalRepositoryError>;
}

#[derive(Error, Debug)]
pub enum LocalRepositoryError {
    #[error("Recipe local repository: {0}")]
    Internal(String),
}