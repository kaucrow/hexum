use async_trait::async_trait;

use crate::prelude::*;
use super::*;

#[derive(Clone)]
pub struct Service {
    local_repo: Arc<dyn LocalRepository>,
}

impl Service {
    pub fn new(
        local_repo: Arc<dyn LocalRepository>,
    ) -> Self {
        Self { local_repo }
    }

    /// Verifies that the group exists and that the given user is its owner.
    async fn verify_group_ownership(&self, user_id: &Uuid, group_id: &Uuid) -> Result<(), UseCaseError> {
        let group = self.local_repo
            .get_group_by_id(group_id)
            .await?
            .ok_or(UseCaseError::GroupNotFound)?;

        if group.created_by_id != *user_id {
            return Err(UseCaseError::NotGroupOwner);
        }

        Ok(())
    }
}

#[async_trait]
impl UseCase for Service {
    async fn get_recipes_group(
        &self,
        group_id: &Uuid,
        user_id: &Uuid,
        recipes_limit: usize
    ) -> Result<Option<RecipesGroup>, UseCaseError> {
        let group = self.local_repo
            .get_group_by_id(group_id)
            .await?
            .ok_or(UseCaseError::GroupNotFound)?;

        if group.created_by_id != *user_id {
            return Err(UseCaseError::NotGroupOwner);
        }

        let recipes = self.local_repo
            .get_group_recipes(group_id, recipes_limit, 0).await?;

        let recipes_group = RecipesGroup {
            group_id: group.id,
            group_name: group.name,
            recipes,
        };

        Ok(Some(recipes_group))
    }

    async fn get_user_groups(
        &self,
        user_id: &Uuid,
    ) -> Result<Vec<Group>, UseCaseError> {
        let groups = self.local_repo
            .get_user_groups(user_id).await?;

        Ok(groups)
    }

    async fn get_user_recipe_groups(&self,
        user_id: &Uuid,
        groups_limit: usize,
        recipes_limit: usize
    ) -> Result<Vec<RecipesGroup>, UseCaseError> {
        let recipe_groups = self.local_repo
            .get_user_recipe_groups(user_id, groups_limit, recipes_limit).await?;

        Ok(recipe_groups)
    }

    async fn get_group_recipes(
        &self,
        user_id: &Uuid,
        group_id: &Uuid,
        recipes_limit: usize,
        offset: usize
    ) -> Result<Vec<RecipePreview>, UseCaseError> {
        self.verify_group_ownership(user_id, group_id).await?;

        let recipe_previews = self.local_repo
            .get_group_recipes(group_id, recipes_limit, offset).await?;

        Ok(recipe_previews)
    }

    async fn create_group(&self, name: &str, description: Option<String>, user_id: &Uuid) -> Result<Uuid, UseCaseError> {
        let group = Group::new(name.to_string(), description, user_id.clone());
        let created_group_id = self.local_repo.create_group(group).await?;

        Ok(created_group_id)
    }

    async fn delete_group(&self, group_id: &Uuid, user_id: &Uuid) -> Result<Option<Uuid>, UseCaseError> {
        let deleted_group_id = self.local_repo.delete_group(group_id, user_id).await?;

        Ok(deleted_group_id)
    }

    async fn add_recipe_to_group(&self, user_id: &Uuid, group_id: &Uuid, recipe_id: &Uuid) -> Result<(), UseCaseError> {
        self.verify_group_ownership(user_id, group_id).await?;

        self.local_repo.add_recipe(group_id, recipe_id).await?;

        Ok(())
    }

    async fn delete_recipe_from_group(&self, user_id: &Uuid, group_id: &Uuid, recipe_id: &Uuid) -> Result<(), UseCaseError> {
        self.verify_group_ownership(user_id, group_id).await?;

        self.local_repo.delete_recipe(group_id, recipe_id).await?;

        Ok(())
    }
}

impl From<LocalRepositoryError> for UseCaseError {
    fn from(e: LocalRepositoryError) -> Self {
        Self::Internal(e.to_string())
    }
}