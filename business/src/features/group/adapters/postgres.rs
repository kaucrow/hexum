use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

use super::*;
use crate::postgres::*;
use crate::prelude::*;

#[derive(Clone)]
pub struct PostgresAdapter {
    pub pool: PgPool,
}

impl PostgresAdapter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn do_get_group_by_id(&self, group_id: Uuid) -> Result<Option<Group>, LocalError> {
        let group = sqlx::query_as::<_, GroupDbRow>(sql(&QUERIES.group.get_by_id))
            .bind(group_id)
            .fetch_optional(&self.pool)
            .await?
            .map(|row| Group::from(row));

        Ok(group)
    }

    async fn do_get_user_recipe_groups(&self,
        user_id: Uuid,
        groups_limit: usize,
        recipes_limit: usize
    ) -> Result<Vec<RecipesGroup>, LocalError> {
        let recipe_groups =
            sqlx::query_as::<_, RecipesGroupDbRow>(sql(&QUERIES.group.get_with_latest_recipes_by_user_id))
                .bind(user_id)
                .bind(groups_limit as i64)
                .bind(recipes_limit as i64)
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|row| RecipesGroup::try_from(row))
                .collect::<Result<Vec<RecipesGroup>, LocalError>>()?;

        Ok(recipe_groups)
    }

    async fn do_get_group_recipes(
        &self,
        group_id: Uuid,
        recipes_limit: usize,
        offset: usize
    ) -> Result<Vec<RecipePreview>, LocalError> {
        let recipe_groups =
            sqlx::query_as::<_, RecipePreviewDbRow>(sql(&QUERIES.group.get_recipes_by_group_id))
                .bind(group_id)
                .bind(recipes_limit as i64)
                .bind(offset as i64)
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|row| RecipePreview::try_from(row))
                .collect::<Result<Vec<RecipePreview>, LocalError>>()?;

        Ok(recipe_groups)
    }

    async fn do_create_group(&self, group: Group) -> Result<(), LocalError> {
        sqlx::query(sql(&QUERIES.group.create))
            .bind(group.id)
            .bind(group.name)
            .bind(group.description)
            .bind(group.created_by_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn do_add_recipe(&self, group_id: Uuid, recipe_id: Uuid) -> Result<(), LocalError> {
        sqlx::query(sql(&QUERIES.group.add_recipe))
            .bind(group_id)
            .bind(recipe_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn do_delete_recipe(&self, group_id: Uuid, recipe_id: Uuid) -> Result<(), LocalError> {
        sqlx::query(sql(&QUERIES.group.delete_recipe))
            .bind(group_id)
            .bind(recipe_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl LocalRepository for PostgresAdapter {
    async fn get_group_by_id(&self, id: Uuid) -> Result<Option<Group>, LocalRepositoryError> {
        Ok(self.do_get_group_by_id(id).await?)
    }

    async fn get_user_recipe_groups(
        &self,
        user_id: Uuid,
        groups_limit: usize,
        recipes_limit: usize
    ) -> Result<Vec<RecipesGroup>, LocalRepositoryError> {
        Ok(self.do_get_user_recipe_groups(user_id, groups_limit, recipes_limit).await?)
    }

    async fn get_group_recipes(
        &self,
        group_id: Uuid,
        recipes_limit: usize,
        offset: usize
    ) -> Result<Vec<RecipePreview>, LocalRepositoryError> {
        Ok(self.do_get_group_recipes(group_id, recipes_limit, offset).await?)
    }

    async fn create_group(&self, group: Group) -> Result<(), LocalRepositoryError> {
        Ok(self.do_create_group(group).await?)
    }

    async fn add_recipe(&self, group_id: Uuid, recipe_id: Uuid) -> Result<(), LocalRepositoryError> {
        Ok(self.do_add_recipe(group_id, recipe_id).await?)
    }

    async fn delete_recipe(&self, group_id: Uuid, recipe_id: Uuid) -> Result<(), LocalRepositoryError> {
        Ok(self.do_delete_recipe(group_id, recipe_id).await?)
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error("{0}")]
    Logic(LocalRepositoryError),
    #[error(transparent)]
    Parsing(#[from] strum::ParseError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

impl From<LocalError> for LocalRepositoryError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::Logic(e) => e,
            LocalError::Parsing(e) => LocalRepositoryError::Internal(e.to_string()),
            LocalError::Sqlx(e) => LocalRepositoryError::Internal(e.to_string()),
        }
    }
}

#[derive(FromRow)]
struct GroupDbRow {
    id: Uuid,
    group_name: String,
    group_description: Option<String>,
    created_by: Uuid,
}

impl From<GroupDbRow> for Group {
    fn from(row: GroupDbRow) -> Self {
        Self {
            id: row.id,
            name: row.group_name,
            description: row.group_description,
            created_by_id: row.created_by,
        }
    }
}

#[derive(FromRow)]
struct RecipesGroupDbRow {
    group_id: Uuid,
    group_name: String,
    #[sqlx(json)]
    recipes: Vec<RecipePreviewDbRow>,
}

impl TryFrom<RecipesGroupDbRow> for RecipesGroup {
    type Error = LocalError;
    fn try_from(row: RecipesGroupDbRow) -> Result<Self, Self::Error> {
        Ok(Self {
            group_id: row.group_id,
            group_name: row.group_name,
            recipes: row
                .recipes
                .into_iter()
                .map(|recipe_preview_row| RecipePreview::try_from(recipe_preview_row))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[derive(FromRow, Deserialize)]
struct RecipePreviewDbRow {
    id: Uuid,
    origin: String,
    recipe_name: String,
    tags: Vec<String>,
    thumbnail_url: Option<String>,
}

impl TryFrom<RecipePreviewDbRow> for RecipePreview {
    type Error = LocalError;
    fn try_from(row: RecipePreviewDbRow) -> Result<Self, Self::Error> {
        let origin: RecipeOrigin = row.origin.parse()?;

        Ok(Self {
            id: row.id,
            origin,
            name: row.recipe_name,
            tags: row.tags,
            thumbnail_url: row.thumbnail_url,
        })
    }
}