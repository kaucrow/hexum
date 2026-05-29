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

    async fn do_get_recipe_search_ids(&self, query: &str) -> Result<Vec<Uuid>, LocalError> {
        let sql_query = if query.chars().count() <= 3 {
            &QUERIES.recipe.get_search_ids_ilike
        } else {
            &QUERIES.recipe.get_search_ids
        };

        let recipe_search_ids = sqlx::query_as::<_, RecipeSearchIdDbRow>(sql(sql_query))
            .bind(query)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| row.id)
            .collect();

        Ok(recipe_search_ids)
    }

    async fn do_get_recipe_search_data_by_ids(
        &self,
        ids: &Vec<Uuid>
    ) -> Result<Vec<RecipeSearchResult>, LocalError> {
        let mut recipe_search_results = sqlx::query_as::<_, RecipeSearchDbRow>(sql(&QUERIES.recipe.get_search_results_by_id))
            .bind(ids)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| RecipeSearchResult::try_from(row))
            .collect::<Result<Vec<_>, _>>()?;

        // Sort back into the original order
        recipe_search_results.sort_by_key(|r| ids.iter().position(|&id| id == r.id));

        Ok(recipe_search_results)
    }
}

#[async_trait]
impl LocalRepository for PostgresAdapter {
    async fn get_recipe_search_ids(&self, query: &str) -> Result<Vec<Uuid>, LocalRepositoryError> {
        Ok(self.do_get_recipe_search_ids(query).await?)
    }

    async fn get_recipe_search_data_by_ids(
        &self,
        ids: &Vec<Uuid>
    ) -> Result<Vec<RecipeSearchResult>, LocalRepositoryError>
    {
        Ok(self.do_get_recipe_search_data_by_ids(ids).await?)
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

/*
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
*/

#[derive(FromRow)]
struct RecipeSearchIdDbRow {
    id: Uuid,
}

#[derive(FromRow)]
struct RecipeSearchDbRow {
    id: Uuid,
    origin: String,
    recipe_name: String,
    tags: Vec<String>,
    thumbnail_url: Option<String>,
}

impl TryFrom<RecipeSearchDbRow> for domain::RecipeSearchResult {
    type Error = LocalError;
    fn try_from(row: RecipeSearchDbRow) -> Result<Self, Self::Error> {
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