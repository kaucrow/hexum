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
}

#[async_trait]
impl LocalRepository for PostgresAdapter {
    async fn get_group_by_id(&self, id: &Uuid) -> Result<Option<Group>, LocalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let group = sqlx::query_as::<_, GroupDbRow>(sql(&QUERIES.group.get_by_id))
                .bind(id)
                .fetch_optional(&self.pool)
                .await?
                .map(|row| Group::from(row));

            Ok(group)
        }.await;

        res.map_err(Into::into)
    }

    async fn get_user_groups(
        &self,
        user_id: &Uuid,
    ) -> Result<Vec<Group>, LocalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let groups = sqlx::query_as::<_, GroupDbRow>(sql(&QUERIES.group.get_by_user_id))
                .bind(user_id)
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|row| Group::from(row))
                .collect();

            Ok(groups)
        }.await;

        res.map_err(Into::into)
    }

    async fn get_user_recipe_groups(
        &self,
        user_id: &Uuid,
        groups_limit: usize,
        groups_offset: usize,
        recipes_limit: usize
    ) -> Result<(Vec<RecipesGroup>, usize), LocalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let rows =
                sqlx::query_as::<_, RecipesGroupCountDbRow>(sql(&QUERIES.group.get_with_latest_recipes_by_user_id))
                    .bind(user_id)
                    .bind(groups_limit as i64)
                    .bind(groups_offset as i64)
                    .bind(recipes_limit as i64)
                    .fetch_all(&self.pool)
                    .await?;

            let total_groups = rows.first().map(|r| r.total_groups_count as usize).unwrap_or(0);

            let recipe_groups = rows
                .into_iter()
                .map(|row| RecipesGroup::try_from(row))
                .collect::<Result<Vec<RecipesGroup>, LocalError>>()?;

            Ok((recipe_groups, total_groups))
        }.await;

        res.map_err(Into::into)
    }

    async fn get_group_recipes(
        &self,
        group_id: &Uuid,
        recipes_limit: usize,
        offset: usize
    ) -> Result<(Vec<RecipePreview>, usize), LocalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let rows =
                sqlx::query_as::<_, GroupRecipePreviewCountDbRow>(sql(&QUERIES.group.get_recipes_by_group_id))
                    .bind(group_id)
                    .bind(recipes_limit as i64)
                    .bind(offset as i64)
                    .fetch_all(&self.pool)
                    .await?;

            let total_count = rows.first().map(|r| r.total_count as usize).unwrap_or(0);

            let recipes = rows
                .into_iter()
                .map(|row| RecipePreview::try_from(row))
                .collect::<Result<Vec<RecipePreview>, LocalError>>()?;

            Ok((recipes, total_count))
        }.await;

        res.map_err(Into::into)
    }

    async fn create_group(&self, group: Group) -> Result<Uuid, LocalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let created_group_id = sqlx::query_as::<_, GroupOperationDbRow>(sql(&QUERIES.group.create))
                .bind(group.id)
                .bind(group.name)
                .bind(group.description)
                .bind(group.created_by_id)
                .fetch_one(&self.pool)
                .await?
                .id;

            Ok(created_group_id)
        }.await;

        res.map_err(Into::into)
    }

    async fn delete_group(&self, group_id: &Uuid, user_id: &Uuid) -> Result<Option<Uuid>, LocalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let deleted_group =
                sqlx::query_as::<_, GroupOperationDbRow>(sql(&QUERIES.group.delete))
                    .bind(group_id)
                    .bind(user_id)
                    .fetch_optional(&self.pool)
                    .await?;

            Ok(deleted_group.map(|row| row.id))
        }.await;

        res.map_err(Into::into)
    }

    async fn add_recipe(&self, group_id: &Uuid, recipe_id: &Uuid) -> Result<(), LocalRepositoryError> {
        let res: Result<_, LocalError> = async {
            sqlx::query(sql(&QUERIES.group.add_recipe))
                .bind(group_id)
                .bind(recipe_id)
                .execute(&self.pool)
                .await?;

            Ok(())
        }.await;

        res.map_err(Into::into)
    }

    async fn delete_recipe(&self, group_id: &Uuid, recipe_id: &Uuid) -> Result<(), LocalRepositoryError> {
        let res: Result<_, LocalError> = async {
            sqlx::query(sql(&QUERIES.group.delete_recipe))
                .bind(group_id)
                .bind(recipe_id)
                .execute(&self.pool)
                .await?;

            Ok(())
        }.await;

        res.map_err(Into::into)
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
struct RecipesGroupCountDbRow {
    group_id: Uuid,
    group_name: String,
    #[sqlx(json)]
    recipes: Vec<RecipePreviewDbRow>,
    total_groups_count: i64,
    total_recipes_count: i64,
}

impl TryFrom<RecipesGroupCountDbRow> for RecipesGroup {
    type Error = LocalError;
    fn try_from(row: RecipesGroupCountDbRow) -> Result<Self, Self::Error> {
        Ok(Self {
            group_id: row.group_id,
            group_name: row.group_name,
            recipes: row
                .recipes
                .into_iter()
                .map(|recipe_preview_row| RecipePreview::try_from(recipe_preview_row))
                .collect::<Result<Vec<_>, _>>()?,
            total_recipes: row.total_recipes_count as usize,
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

#[derive(FromRow)]
struct GroupRecipePreviewCountDbRow {
    id: Uuid,
    origin: String,
    recipe_name: String,
    tags: Vec<String>,
    thumbnail_url: Option<String>,
    total_count: i64,
}

impl TryFrom<GroupRecipePreviewCountDbRow> for RecipePreview {
    type Error = LocalError;
    fn try_from(row: GroupRecipePreviewCountDbRow) -> Result<Self, Self::Error> {
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

#[derive(FromRow)]
struct GroupOperationDbRow {
    id: Uuid,
}