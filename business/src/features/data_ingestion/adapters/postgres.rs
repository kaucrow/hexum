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

    async fn do_sync_recipes(&self, recipes: Vec<Recipe>) -> Result<(), LocalError> {
        let mut tx: Transaction<'_, Postgres> = self.pool.begin().await?;

        // Storage Vecs for main recipes
        let mut recipe_ids = Vec::with_capacity(recipes.len());
        let mut external_ids = Vec::with_capacity(recipes.len());
        let mut names = Vec::with_capacity(recipes.len());
        let mut descriptions: Vec<Option<String>> = Vec::with_capacity(recipes.len());
        let mut instructions = Vec::with_capacity(recipes.len());
        let mut thumbnails = Vec::with_capacity(recipes.len());
        let mut videos = Vec::with_capacity(recipes.len());

        // Intermediate Vecs for tracking relationships before DB validation
        let mut raw_tags = Vec::new();
        let mut raw_ingredients = Vec::new();

        // Flatten and parse types
        for recipe in recipes {
            let local_uuid = Uuid::new_v4();

            // Parse external ID string into i32 safely
            let ext_id = recipe.id.parse()?;

            recipe_ids.push(local_uuid);
            external_ids.push(ext_id);
            names.push(recipe.name);
            descriptions.push(None);
            instructions.push(recipe.instructions);
            thumbnails.push(recipe.thumbnail_url);
            videos.push(recipe.video_url);

            // Temporarily couple child records with their integer external ID
            for tag in recipe.tags {
                raw_tags.push((ext_id, tag));
            }
            for (ing_name, measure) in recipe.ingredients {
                raw_ingredients.push((ext_id, ing_name, measure));
            }
        }

        // ──────────────────────────────────────────
        //  DB Call 1: Upsert Main Recipes
        // ──────────────────────────────────────────
        let upserted_rows = sqlx::query_as::<_, RecipeUpsertReturnDbRow>(sql(&QUERIES.data_ingestion.sync_recipes))
            .bind(&recipe_ids)
            .bind(&external_ids)
            .bind(&names)
            .bind(&descriptions)
            .bind(&instructions)
            .bind(&thumbnails as &[Option<String>])
            .bind(&videos as &[Option<String>])
            .fetch_all(&mut *tx)
            .await?;

        // Build an absolute ID lookup map based on what is actually in the DB right now
        let uuid_lookup_map: BTreeMap<i32, Uuid> = upserted_rows
            .into_iter()
            .map(|row| (row.external_id, row.id))
            .collect();

        // Build final child arrays using resolved UUID foreign keys
        let mut tag_ids = Vec::with_capacity(raw_tags.len());
        let mut tag_recipe_ids = Vec::with_capacity(raw_tags.len());
        let mut tag_names = Vec::with_capacity(raw_tags.len());

        for (ext_id, tag_name) in raw_tags {
            if let Some(&resolved_uuid) = uuid_lookup_map.get(&ext_id) {
                tag_ids.push(Uuid::new_v4());
                tag_recipe_ids.push(resolved_uuid);
                tag_names.push(tag_name);
            }
        }

        let mut ing_ids = Vec::with_capacity(raw_ingredients.len());
        let mut ing_recipe_ids = Vec::with_capacity(raw_ingredients.len());
        let mut ing_names = Vec::with_capacity(raw_ingredients.len());
        let mut ing_measures = Vec::with_capacity(raw_ingredients.len());

        for (ext_id, name, measure) in raw_ingredients {
            if let Some(&resolved_uuid) = uuid_lookup_map.get(&ext_id) {
                ing_ids.push(Uuid::new_v4());
                ing_recipe_ids.push(resolved_uuid);
                ing_names.push(name);
                ing_measures.push(measure);
            }
        }

        // ──────────────────────────────────────────
        //  DB Call 2: Batch Insert Tags
        // ──────────────────────────────────────────
        sqlx::query(sql(&QUERIES.data_ingestion.sync_tags))
            .bind(&tag_ids)
            .bind(&tag_recipe_ids)
            .bind(&tag_names)
        .execute(&mut *tx)
        .await?;

        // ──────────────────────────────────────────
        //  DB CALL 3: Batch Insert Ingredients
        // ──────────────────────────────────────────
        sqlx::query(sql(&QUERIES.data_ingestion.sync_ingredients))
            .bind(&ing_ids)
            .bind(&ing_recipe_ids)
            .bind(&ing_names)
            .bind(&ing_measures)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }
}

#[async_trait]
impl LocalRepository for PostgresAdapter {
    async fn sync_recipes(&self, recipes: Vec<Recipe>) -> Result<(), LocalRepositoryError> {
        Ok(self.do_sync_recipes(recipes).await?)
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error("{0}")]
    Logic(LocalRepositoryError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Parsing(#[from] std::num::ParseIntError),
}

impl From<LocalError> for LocalRepositoryError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::Logic(e) => e,
            LocalError::Sqlx(e) => LocalRepositoryError::Internal(e.to_string()),
            LocalError::Parsing(e) => LocalRepositoryError::Internal(e.to_string()),
        }
    }
}

#[derive(FromRow)]
pub struct RecipeUpsertReturnDbRow {
    id: Uuid,
    external_id: i32,
}