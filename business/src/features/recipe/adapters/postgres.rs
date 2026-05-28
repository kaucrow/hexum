use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;
use anyhow::Result;

use crate::prelude::*;
use crate::postgres::*;
use super::*;

#[derive(Clone)]
pub struct PostgresAdapter {
    pub pool: PgPool,
}

impl PostgresAdapter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn do_get_search_candidates(&self, name: &str) -> Result<Vec<RecipeSearchCandidate>, LocalError> {
        let recipe_search_candidates = sqlx::query_as::<_, RecipeSearchDbRow>(sql(&QUERIES.recipe.search_many_id_by_name))
            .bind(name)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| RecipeSearchCandidate::from(row))
            .collect();

        Ok(recipe_search_candidates)
    }

    async fn do_get_recipes_by_id(&self, ids: &Vec<Uuid>) -> Result<Vec<Recipe>, LocalError> {
        let recipes = sqlx::query_as::<_, RecipeDbRow>(sql(&QUERIES.recipe.get_many_by_id))
            .bind(ids)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| Recipe::from(row))
            .collect();

        Ok(recipes)
    }
}

#[async_trait]
impl LocalRepository for PostgresAdapter {
    async fn get_search_candidates(&self, name: &str) -> Result<Vec<RecipeSearchCandidate>, LocalRepositoryError> {
        Ok(self.do_get_search_candidates(name).await?)
    }

    async fn get_recipes_by_id(&self, ids: &Vec<Uuid>) -> Result<Vec<Recipe>, LocalRepositoryError> {
        Ok(self.do_get_recipes_by_id(ids).await?)
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error("{0}")]
    Logic(LocalRepositoryError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

impl From<LocalError> for LocalRepositoryError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::Logic(e) => e,
            LocalError::Sqlx(e) => LocalRepositoryError::Internal(e.to_string()),
        }
    }
}

#[derive(FromRow)]
struct RecipeDbRow {
    id: Uuid,
    recipe_name: String,
    recipe_description: String,
    instructions: String,
    thumbnail_url: Option<String>,
    video_url: Option<String>,
    tags: sqlx::types::Json<Vec<String>>,
    ingredients: sqlx::types::Json<BTreeMap<String, String>>,
}

impl From<RecipeDbRow> for Recipe {
    fn from(row: RecipeDbRow) -> Self {
        Self {
            origin: RecipeOrigin::Local(row.id),
            name: row.recipe_name,
            description: Some(row.recipe_description),
            instructions: row.instructions,
            thumbnail_url: row.thumbnail_url,
            video_url: row.video_url,
            tags: row.tags.0,
            ingredients: row.ingredients.0,
        }
    }
}

#[derive(FromRow)]
struct RecipeSearchDbRow {
    id: Uuid,
    recipe_name: String,
}

impl From<RecipeSearchDbRow> for RecipeSearchCandidate {
    fn from(row: RecipeSearchDbRow) -> Self {
        Self {
            origin: RecipeOrigin::Local(row.id),
            recipe_name: row.recipe_name,
        }
    }
}