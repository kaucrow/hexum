use anyhow::Result;
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

    async fn do_get_recipe_search_ids(
        &self,
        query: Option<&str>,
        tags: Option<&[String]>,
    ) -> Result<Vec<Uuid>, LocalError> {
        match (query, tags) {
            (Some(q), None) => {
                // ─── Query-only Search ───
                let sql_query = if q.chars().count() <= 3 {
                    &QUERIES.recipe.get_search_ids_by_query_ilike
                } else {
                    &QUERIES.recipe.get_search_ids_by_query
                };

                let recipe_search_ids = sqlx::query_as::<_, RecipeSearchIdDbRow>(sql(sql_query))
                    .bind(q)
                    .fetch_all(&self.pool)
                    .await?
                    .into_iter()
                    .map(|row| row.id)
                    .collect();

                Ok(recipe_search_ids)
            }
            (None, Some(t)) => {
                // ─── Tags-only Search ───

                let t: Vec<String> = t
                    .into_iter()
                    .map(|t| t.to_lowercase())
                    .collect();

                self.do_get_recipe_search_ids_by_tags(&t).await
            }
            (Some(q), Some(t)) => {
                // ─── Combined Query + Tags Search ───

                let t: Vec<String> = t
                    .into_iter()
                    .map(|t| t.to_lowercase())
                    .collect();

                self.do_get_recipe_search_ids_by_query_and_tags(q, &t).await
            }
            (None, None) => {
                // ─── Neither: Return Empty ───
                Ok(Vec::new())
            }
        }
    }

    async fn do_get_recipe_search_ids_by_tags(
        &self,
        tags: &[String],
    ) -> Result<Vec<Uuid>, LocalError> {
        let recipe_search_ids =
            sqlx::query_as::<_, RecipeSearchIdDbRow>(sql(&QUERIES.recipe.get_search_ids_by_tags))
                .bind(tags)
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|row| row.id)
                .collect();

        Ok(recipe_search_ids)
    }

    async fn do_get_recipe_search_ids_by_query_and_tags(
        &self,
        query: &str,
        tags: &[String],
    ) -> Result<Vec<Uuid>, LocalError> {
        let sql_query = if query.chars().count() <= 3 {
            &QUERIES.recipe.get_search_ids_by_query_and_tags_ilike
        } else {
            &QUERIES.recipe.get_search_ids_by_query_and_tags
        };

        let recipe_search_ids = sqlx::query_as::<_, RecipeSearchIdDbRow>(sql(sql_query))
            .bind(query)
            .bind(tags)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| row.id)
            .collect();

        Ok(recipe_search_ids)
    }

    async fn do_get_recipe_search_data_by_ids(
        &self,
        ids: &Vec<Uuid>,
    ) -> Result<Vec<RecipeSearchResult>, LocalError> {
        let mut recipe_search_results =
            sqlx::query_as::<_, RecipeSearchDbRow>(sql(&QUERIES.recipe.get_search_results_by_id))
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

    async fn do_get_recipe_by_id(&self, id: &Uuid) -> Result<Option<Recipe>, LocalError> {
        let recipe = sqlx::query_as::<_, RecipeDbRow>(sql(&QUERIES.recipe.get_by_id))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .map(|row| Recipe::try_from(row))
            .transpose()?;

        Ok(recipe)
    }

    async fn do_get_tag_search_matches(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<String>, LocalError> {
        let tag_search_results =
            sqlx::query_as::<_, TagDbRow>(sql(&QUERIES.tag.get_search_matches))
                .bind(query)
                .bind(limit as i64)
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|row| row.name)
                .collect();

        Ok(tag_search_results)
    }
}

#[async_trait]
impl LocalRepository for PostgresAdapter {
    async fn get_recipe_search_ids<'a>(
        &self,
        query: Option<&'a str>,
        tags: Option<&'a [String]>,
    ) -> Result<Vec<Uuid>, LocalRepositoryError> {
        Ok(self.do_get_recipe_search_ids(query, tags).await?)
    }

    async fn get_recipe_search_data_by_ids(
        &self,
        ids: &Vec<Uuid>,
    ) -> Result<Vec<RecipeSearchResult>, LocalRepositoryError> {
        Ok(self.do_get_recipe_search_data_by_ids(ids).await?)
    }

    async fn get_recipe_by_id(&self, id: &Uuid) -> Result<Option<Recipe>, LocalRepositoryError> {
        Ok(self.do_get_recipe_by_id(id).await?)
    }

    async fn get_tag_search_matches(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<String>, LocalRepositoryError> {
        Ok(self.do_get_tag_search_matches(query, limit).await?)
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
    origin: String,
    recipe_name: String,
    recipe_description: Option<String>,
    instructions: String,
    thumbnail_url: Option<String>,
    video_url: Option<String>,
    tags: sqlx::types::Json<Vec<String>>,
    ingredients: sqlx::types::Json<BTreeMap<String, String>>,
}

impl TryFrom<RecipeDbRow> for Recipe {
    type Error = LocalError;

    fn try_from(row: RecipeDbRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            origin: row.origin.parse()?,
            name: row.recipe_name,
            description: row.recipe_description,
            instructions: row.instructions,
            thumbnail_url: row.thumbnail_url,
            video_url: row.video_url,
            tags: row.tags.0,
            ingredients: row.ingredients.0,
        })
    }
}

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

#[derive(FromRow)]
struct TagDbRow {
    name: String,
}
