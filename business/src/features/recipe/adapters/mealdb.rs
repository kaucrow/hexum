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

    async fn do_get_recipe_search_results(&self, name: &str) -> Result<Vec<RecipeSearchResult>, LocalError> {
        let url = format!("{}/{}/search.php", self.api_url, self.api_key);

        let response = self.client
            .get(&url)
            .query(&[("s", name)])
            .send()
            .await
            .map_err(|e| LocalError::Logic(ExternalRepositoryError::Network(e.to_string())))?;

        let deserialized_response: MealdbRecipeResponse = response
            .json()
            .await
            .map_err(|e| LocalError::Logic(ExternalRepositoryError::Serialization(e.to_string())))?;

        let raw_recipes = deserialized_response.meals.unwrap_or_default();

        // Allocate Vec capacity
        let mut search_results = Vec::with_capacity(raw_recipes.len());

        // Map the single payload into both structures
        for item in raw_recipes {
            // Build the full domain model used for hydration and caching
            let mut combined_tags: Vec<String> = Vec::new();

            // Push the category into the tags
            let category_trim = item.category.trim();
            if !category_trim.is_empty() {
                combined_tags.push(category_trim.to_string());
            }

            // If the response tags is not null, add them to the tags list
            if let Some(tags) = item.tags {
                // Split tags by commas
                let parsed_tags = tags
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty());

                combined_tags.extend(parsed_tags);
            }

            search_results.push(RecipeSearchResult {
                origin: RecipeOrigin::External(item.id),
                name: item.name,
                thumbnail_url: item.image_url,
                tags: combined_tags,
            });
        }

        Ok(search_results)
    }
}

#[async_trait]
impl ExternalRepository for MealdbAdapter {
    async fn get_recipe_search_results(&self, name: &str) -> Result<Vec<RecipeSearchResult>, ExternalRepositoryError> {
        Ok(self.do_get_recipe_search_results(name).await?)
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
struct MealdbRecipeResponse {
    meals: Option<Vec<MealdbMeal>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MealdbMeal {
    #[serde(rename = "idMeal")]
    id: String,
    #[serde(rename = "strMeal")]
    name: String,
    #[serde(rename = "strCategory")]
    category: String,
    #[serde(rename = "strTags")]
    tags: Option<String>,
    #[serde(rename = "strMealThumb", deserialize_with = "empty_is_none")]
    image_url: Option<String>,
}