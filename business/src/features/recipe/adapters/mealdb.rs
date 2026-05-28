use async_trait::async_trait;
use thiserror::Error;
use reqwest::Client;
use anyhow::Result;

use crate::prelude::*;
use super::*;

#[derive(Clone)]
pub struct MealdbAdapter {
    client: Client,
    api_url: String,
    api_key: String,
}

impl MealdbAdapter {
    pub fn new(api_url: String, api_key: String) -> Self {
        let client = Client::new();
        Self { client, api_url, api_key }
    }

    async fn do_search_by_name(&self, name: &str) -> Result<RecipeSearchResult, LocalError> {
        let url = format!("{}/{}/recipes/search.php?s={}", self.api_url, self.api_key, name);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| LocalError::Logic(ExternalRepositoryError::Network(e.to_string())))?;

        let raw_recipes: Vec<MealdbRecipe> = response
            .json()
            .await
            .map_err(|e| LocalError::Logic(ExternalRepositoryError::Serialization(e.to_string())))?;

        // Allocate Vec capacity
        let mut candidates = Vec::with_capacity(raw_recipes.len());
        let mut full_recipes = Vec::with_capacity(raw_recipes.len());

        // Map the single payload into both structures
        for item in raw_recipes {
            // Build the lightweight pointer used for sorting/merging later
            candidates.push(RecipeSearchCandidate {
                origin: RecipeOrigin::External(item.id.clone()),
                recipe_name: item.name.clone(),
            });

            // Build the full domain model used for hydration and caching
            let mut combined_tags: Vec<String> = Vec::new();

            // Push the category into the tags
            let category_trim = item.category.trim();
            if !category_trim.is_empty() {
                combined_tags.push(category_trim.to_string());
            }

            // Split tags by commas
            let parsed_tags = item.tags
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty());

            combined_tags.extend(parsed_tags);

            // Put the ingredients into a BTreeMap
            let mut ingredients= BTreeMap::new();

            let raw_ingredients = [
                item.str_ingredient_1, item.str_ingredient_2, item.str_ingredient_3, item.str_ingredient_4,
                item.str_ingredient_5, item.str_ingredient_6, item.str_ingredient_7, item.str_ingredient_8,
                item.str_ingredient_9, item.str_ingredient_10, item.str_ingredient_11, item.str_ingredient_12,
                item.str_ingredient_13, item.str_ingredient_14, item.str_ingredient_15, item.str_ingredient_16,
                item.str_ingredient_17, item.str_ingredient_18, item.str_ingredient_19, item.str_ingredient_20,
            ];

            let raw_measures = [
                item.str_measure_1, item.str_measure_2, item.str_measure_3, item.str_measure_4,
                item.str_measure_5, item.str_measure_6, item.str_measure_7, item.str_measure_8,
                item.str_measure_9, item.str_measure_10, item.str_measure_11, item.str_measure_12,
                item.str_measure_13, item.str_measure_14, item.str_measure_15, item.str_measure_16,
                item.str_measure_17, item.str_measure_18, item.str_measure_19, item.str_measure_20,
            ];

            // Push ingredients & measures that are not an empty string
            for (ingredient, measure) in raw_ingredients.into_iter().zip(raw_measures.into_iter()) {
                if !ingredient.is_empty() {
                    let ingredient_trim = ingredient.trim();
                    let measure_trim = measure.trim();

                    ingredients.insert(ingredient_trim.to_string(), measure_trim.to_string());
                }
            }

            full_recipes.push(Recipe {
                origin: RecipeOrigin::External(item.id),
                name: item.name,
                description: None,
                instructions: item.instructions,
                thumbnail_url: item.image_url,
                video_url: item.video_url,
                tags: combined_tags,
                ingredients,
            });
        }

        Ok(ExternalSearchResponse {
            candidates,
            full_recipes,
        })
    }
}

#[async_trait]
impl ExternalRepository for MealdbAdapter {
    async fn search_by_name(&self, name: &str) -> Result<ExternalSearchResponse, ExternalRepositoryError> {
        Ok(self.do_search_by_name(name).await?)
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error("{0}")]
    Logic(ExternalRepositoryError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

impl From<LocalError> for ExternalRepositoryError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::Logic(e) => e,
            LocalError::Sqlx(e) => ExternalRepositoryError::Internal(e.to_string()),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MealdbRecipe {
    #[serde(rename = "idMeal")]
    id: String,
    #[serde(rename = "strMeal")]
    name: String,
    #[serde(rename = "strCategory")]
    category: String,
    #[serde(rename = "strTags")]
    tags: String,
    #[serde(rename = "strInstructions")]
    instructions: String,
    #[serde(rename = "strMealThumb", deserialize_with = "empty_is_none")]
    image_url: Option<String>,
    #[serde(rename = "strYoutube", deserialize_with = "empty_is_none")]
    video_url: Option<String>,

    pub str_ingredient_1: String, pub str_ingredient_2: String,
    pub str_ingredient_3: String, pub str_ingredient_4: String,
    pub str_ingredient_5: String, pub str_ingredient_6: String,
    pub str_ingredient_7: String, pub str_ingredient_8: String,
    pub str_ingredient_9: String, pub str_ingredient_10: String,
    pub str_ingredient_11: String, pub str_ingredient_12: String,
    pub str_ingredient_13: String, pub str_ingredient_14: String,
    pub str_ingredient_15: String, pub str_ingredient_16: String,
    pub str_ingredient_17: String, pub str_ingredient_18: String,
    pub str_ingredient_19: String, pub str_ingredient_20: String,

    pub str_measure_1: String, pub str_measure_2: String,
    pub str_measure_3: String, pub str_measure_4: String,
    pub str_measure_5: String, pub str_measure_6: String,
    pub str_measure_7: String, pub str_measure_8: String,
    pub str_measure_9: String, pub str_measure_10: String,
    pub str_measure_11: String, pub str_measure_12: String,
    pub str_measure_13: String, pub str_measure_14: String,
    pub str_measure_15: String, pub str_measure_16: String,
    pub str_measure_17: String, pub str_measure_18: String,
    pub str_measure_19: String, pub str_measure_20: String,
}