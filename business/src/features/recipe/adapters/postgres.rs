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

    async fn get_recipe_search_ids_by_tags(
        &self,
        tags: &[String],
    ) -> Result<Vec<Uuid>, LocalError> {
        let recipe_search_ids =
            sqlx::query_as::<_, RecipeIdDbRow>(sql(&QUERIES.recipe.get_search_ids_by_tags))
                .bind(tags)
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|row| row.id)
                .collect();

        Ok(recipe_search_ids)
    }

    async fn get_recipe_search_ids_by_query_and_tags(
        &self,
        query: &str,
        tags: &[String],
    ) -> Result<Vec<Uuid>, LocalError> {
        let sql_query = if query.chars().count() <= 3 {
            &QUERIES.recipe.get_search_ids_by_query_and_tags_ilike
        } else {
            &QUERIES.recipe.get_search_ids_by_query_and_tags
        };

        let recipe_search_ids = sqlx::query_as::<_, RecipeIdDbRow>(sql(sql_query))
            .bind(query)
            .bind(tags)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| row.id)
            .collect();

        Ok(recipe_search_ids)
    }
}

#[async_trait]
impl LocalRepository for PostgresAdapter {
    async fn get_recipe_search_ids<'a>(
        &self,
        query: Option<&'a str>,
        tags: Option<&'a [String]>,
    ) -> Result<Vec<Uuid>, LocalRepositoryError> {
        let res: Result<_, LocalError> = async {
            match (query, tags) {
                (Some(q), None) => {
                    // ─── Query-only Search ───
                    let sql_query = if q.chars().count() <= 3 {
                        &QUERIES.recipe.get_search_ids_by_query_ilike
                    } else {
                        &QUERIES.recipe.get_search_ids_by_query
                    };

                    let recipe_search_ids = sqlx::query_as::<_, RecipeIdDbRow>(sql(sql_query))
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

                    self.get_recipe_search_ids_by_tags(&t).await
                }
                (Some(q), Some(t)) => {
                    // ─── Combined Query + Tags Search ───

                    let t: Vec<String> = t
                        .into_iter()
                        .map(|t| t.to_lowercase())
                        .collect();

                    self.get_recipe_search_ids_by_query_and_tags(q, &t).await
                }
                (None, None) => {
                    // ─── Neither: Return Empty ───
                    Ok(Vec::new())
                }
            }
        }.await;

        res.map_err(Into::into)
    }

    async fn get_recipe_previews_by_ids(
        &self,
        ids: &Vec<Uuid>,
    ) -> Result<Vec<RecipePreview>, LocalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let mut recipe_search_results =
                sqlx::query_as::<_, RecipePreviewDbRow>(sql(&QUERIES.recipe.get_previews_by_id))
                    .bind(ids)
                    .fetch_all(&self.pool)
                    .await?
                    .into_iter()
                    .map(|row| RecipePreview::try_from(row))
                    .collect::<Result<Vec<_>, _>>()?;

            // Sort back into the original order
            recipe_search_results.sort_by_key(|r| ids.iter().position(|&id| id == r.id));

            Ok(recipe_search_results)
        }.await;

        res.map_err(Into::into)
    }

    async fn get_random_recipe_previews(&self, limit: usize) -> Result<Vec<RecipePreview>, LocalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let recipes =
                sqlx::query_as::<_, RecipePreviewDbRow>(sql(&QUERIES.recipe.get_random_previews))
                    .bind(limit as i64)
                    .fetch_all(&self.pool)
                    .await?
                    .into_iter()
                    .map(|row| RecipePreview::try_from(row))
                    .collect::<Result<Vec<_>, _>>()?;

            Ok(recipes)
        }.await;

        res.map_err(Into::into)
    }

    async fn get_latest_recipe_previews(&self, limit: usize) -> Result<Vec<RecipePreview>, LocalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let recipes =
                sqlx::query_as::<_, RecipePreviewDbRow>(sql(&QUERIES.recipe.get_latest_previews))
                    .bind(limit as i64)
                    .fetch_all(&self.pool)
                    .await?
                    .into_iter()
                    .map(|row| RecipePreview::try_from(row))
                    .collect::<Result<Vec<_>, _>>()?;

            Ok(recipes)
        }.await;

        res.map_err(Into::into)
    }

    async fn get_recipe_by_id(&self, id: &Uuid) -> Result<Option<Recipe>, LocalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let recipe = sqlx::query_as::<_, RecipeDbRow>(sql(&QUERIES.recipe.get_by_id))
                .bind(id)
                .fetch_optional(&self.pool)
                .await?
                .map(|row| Recipe::try_from(row))
                .transpose()?;

            Ok(recipe)
        }.await;

        res.map_err(Into::into)
    }

    async fn get_tag_search_matches(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<String>, LocalRepositoryError> {
        let res: Result<_, LocalError> = async {
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
        }.await;

        res.map_err(Into::into)
    }

    async fn get_top_tag_names(&self, limit: usize) -> Result<Vec<String>, LocalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let names =
                sqlx::query_as::<_, TagDbRow>(sql(&QUERIES.tag.get_top_tag_names))
                    .bind(limit as i64)
                    .fetch_all(&self.pool)
                    .await?
                    .into_iter()
                    .map(|row| row.name)
                    .collect();

            Ok(names)
        }.await;

        res.map_err(Into::into)
    }

    async fn get_recipe_previews_by_tag_name(&self, tag_name: &str, limit: usize) -> Result<Vec<RecipePreview>, LocalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let recipes =
                sqlx::query_as::<_, RecipePreviewDbRow>(sql(&QUERIES.recipe.get_previews_by_tag_name))
                    .bind(tag_name)
                    .bind(limit as i64)
                    .fetch_all(&self.pool)
                    .await?
                    .into_iter()
                    .map(|row| RecipePreview::try_from(row))
                    .collect::<Result<Vec<_>, _>>()?;

            Ok(recipes)
        }.await;

        res.map_err(Into::into)
    }

    async fn get_latest_recipe_history(&self, user_id: &Uuid, limit: usize, offset: usize) -> Result<Vec<RecipePreview>, LocalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let recipe_history =
                sqlx::query_as::<_, RecipePreviewDbRow>(sql(&QUERIES.recipe.get_latest_history_by_user_id))
                    .bind(user_id)
                    .bind(limit as i64)
                    .bind(offset as i64)
                    .fetch_all(&self.pool)
                    .await?
                    .into_iter()
                    .map(|row| RecipePreview::try_from(row))
                    .collect::<Result<Vec<_>, _>>()?;

            Ok(recipe_history)
        }.await;

        res.map_err(Into::into)
    }

    async fn get_recipe_previews_by_creator(&self, user_id: &Uuid, limit: usize, offset: usize) -> Result<(Vec<RecipePreview>, usize), LocalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let rows = sqlx::query_as::<_, RecipePreviewCountDbRow>(sql(&QUERIES.recipe.get_created_by_user))
                .bind(user_id)
                .bind(limit as i64)
                .bind(offset as i64)
                .fetch_all(&self.pool)
                .await?;

            let total_count = rows.first().map(|r| r.total_count as usize).unwrap_or(0);

            let recipes = rows
                .into_iter()
                .map(|row| -> Result<RecipePreview, LocalError> {
                    let origin: RecipeOrigin = row.origin.parse()?;
                    Ok(RecipePreview {
                        id: row.id,
                        origin,
                        name: row.recipe_name,
                        tags: row.tags,
                        thumbnail_url: row.thumbnail_url,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;

            Ok((recipes, total_count))
        }.await;

        res.map_err(Into::into)
    }

    async fn create_recipe(&self, data: CreateRecipeData) -> Result<Recipe, LocalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let mut tx: Transaction<'_, Postgres> = self.pool.begin().await?;

            // Insert the main recipe row
            sqlx::query(sql(&QUERIES.recipe.create))
                .bind(data.id)
                .bind(&data.name)
                .bind(&data.description)
                .bind(&data.instructions)
                .bind(&data.thumbnail_url as &Option<String>)
                .bind(data.created_by)
                .execute(&mut *tx)
                .await?;

            // Batch insert tags
            if !data.tags.is_empty() {
                let tag_ids: Vec<Uuid> = (0..data.tags.len()).map(|_| Uuid::new_v4()).collect();
                let tag_recipe_ids = vec![data.id; data.tags.len()];

                sqlx::query(sql(&QUERIES.recipe.batch_insert_tags))
                    .bind(&tag_ids)
                    .bind(&tag_recipe_ids)
                    .bind(&data.tags)
                    .execute(&mut *tx)
                    .await?;
            }

            // Batch insert ingredients
            if !data.ingredients.is_empty() {
                let ing_ids: Vec<Uuid> = (0..data.ingredients.len()).map(|_| Uuid::new_v4()).collect();
                let ing_recipe_ids = vec![data.id; data.ingredients.len()];
                let ing_names: Vec<String> = data.ingredients.keys().cloned().collect();
                let ing_measures: Vec<String> = data.ingredients.values().cloned().collect();

                sqlx::query(sql(&QUERIES.recipe.batch_insert_ingredients))
                    .bind(&ing_ids)
                    .bind(&ing_recipe_ids)
                    .bind(&ing_names)
                    .bind(&ing_measures)
                    .execute(&mut *tx)
                    .await?;
            }

            tx.commit().await?;

            Ok(Recipe {
                id: data.id,
                origin: RecipeOrigin::Local,
                name: data.name,
                description: data.description,
                tags: data.tags,
                ingredients: data.ingredients,
                instructions: data.instructions,
                thumbnail_url: data.thumbnail_url,
                video_url: None,
                created_by: Some(data.created_by),
            })
        }.await;

        res.map_err(Into::into)
    }

    async fn delete_recipe(&self, recipe_id: &Uuid, user_id: &Uuid) -> Result<Option<Uuid>, LocalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let deleted_recipe_id = sqlx::query_as::<_, RecipeIdDbRow>(sql(&QUERIES.recipe.delete))
                .bind(recipe_id)
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?
                .map(|row| row.id);

            Ok(deleted_recipe_id)
        }.await;

        res.map_err(Into::into)

    }

    async fn record_recipe_history(&self, user_id: &Uuid, recipe_id: &Uuid) -> Result<(), LocalRepositoryError> {
        let res: Result<_, LocalError> = async {
            sqlx::query(sql(&QUERIES.recipe.record_history))
                .bind(user_id)
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
    created_by: Option<Uuid>,
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
            created_by: row.created_by,
        })
    }
}

#[derive(FromRow)]
struct RecipeIdDbRow {
    id: Uuid,
}

#[derive(FromRow)]
struct RecipePreviewDbRow {
    id: Uuid,
    origin: String,
    recipe_name: String,
    tags: Vec<String>,
    thumbnail_url: Option<String>,
}

#[derive(FromRow)]
struct RecipePreviewCountDbRow {
    id: Uuid,
    origin: String,
    recipe_name: String,
    tags: Vec<String>,
    thumbnail_url: Option<String>,
    total_count: i64,
}

impl TryFrom<RecipePreviewDbRow> for domain::RecipePreview {
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
struct TagDbRow {
    name: String,
}