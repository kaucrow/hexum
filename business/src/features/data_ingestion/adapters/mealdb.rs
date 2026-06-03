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
}

#[async_trait]
impl ExternalRepository for MealdbAdapter {
    async fn get_recipes_by_first_letter(&self, letter: char) -> Result<Vec<Recipe>, ExternalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let url = format!("{}/{}/search.php", self.api_url, self.api_key);

            let response = self.client
                .get(&url)
                .query(&[("f", letter)])
                .send()
                .await
                .map_err(|e| LocalError::Logic(ExternalRepositoryError::Network(e.to_string())))?;

            let deserialized_response: MealdbRecipeResponse = response
                .json()
                .await
                .map_err(|e| {
                    error!("{:?}", e);
                    LocalError::Logic(ExternalRepositoryError::Serialization(e.to_string()))
                })?;

            let raw_recipes = deserialized_response.meals.unwrap_or_default();

            // Allocate Vec capacity
            let mut search_results = Vec::with_capacity(raw_recipes.len());

            // Map the single payload into both structures
            for item in raw_recipes {
                // ─── Build the full domain model used ───

                // Build the tags Vec
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

                // Put the ingredients into a BTreeMap
                let mut ingredients= BTreeMap::new();

                let raw_ingredients = [
                    item.strIngredient1, item.strIngredient2, item.strIngredient3, item.strIngredient4,
                    item.strIngredient5, item.strIngredient6, item.strIngredient7, item.strIngredient8,
                    item.strIngredient9, item.strIngredient10, item.strIngredient11, item.strIngredient12,
                    item.strIngredient13, item.strIngredient14, item.strIngredient15, item.strIngredient16,
                    item.strIngredient17, item.strIngredient18, item.strIngredient19, item.strIngredient20,
                ];

                let raw_measures = [
                    item.strMeasure1, item.strMeasure2, item.strMeasure3, item.strMeasure4,
                    item.strMeasure5, item.strMeasure6, item.strMeasure7, item.strMeasure8,
                    item.strMeasure9, item.strMeasure10, item.strMeasure11, item.strMeasure12,
                    item.strMeasure13, item.strMeasure14, item.strMeasure15, item.strMeasure16,
                    item.strMeasure17, item.strMeasure18, item.strMeasure19, item.strMeasure20,
                ];

                // Push ingredients & measures that are not an empty string
                for (ingredient, measure) in raw_ingredients.into_iter().zip(raw_measures.into_iter()) {
                    if let (Some(ingredient), Some(measure)) = (ingredient, measure) {
                        let ingredient_trim = ingredient.trim();
                        let measure_trim = measure.trim();

                        ingredients.insert(ingredient_trim.to_string(), measure_trim.to_string());
                    }
                }

                search_results.push(Recipe {
                    id: item.id,
                    name: item.name,
                    tags: combined_tags,
                    ingredients,
                    instructions: item.instructions,
                    thumbnail_url: item.image_url,
                    video_url: item.video_url,
                });
            }

            Ok(search_results)
        }.await;

        res.map_err(Into::into)
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error("{0}")]
    Logic(ExternalRepositoryError),
}

impl From<LocalError> for ExternalRepositoryError {
    fn from(e: LocalError) -> Self {
        ExternalRepositoryError::Internal(e.to_string())
    }
}

#[derive(Deserialize)]
struct MealdbRecipeResponse {
    meals: Option<Vec<MealdbMeal>>,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct MealdbMeal {
    #[serde(rename = "idMeal")]
    id: String,
    #[serde(rename = "strMeal")]
    name: String,
    #[serde(rename = "strCategory")]
    category: String,
    #[serde(rename = "strTags")]
    tags: Option<String>,
    #[serde(rename = "strInstructions")]
    instructions: String,
    #[serde(rename = "strMealThumb", deserialize_with = "empty_is_none")]
    image_url: Option<String>,
    #[serde(rename = "strYoutube", deserialize_with = "empty_is_none")]
    video_url: Option<String>,

    #[serde(deserialize_with = "empty_is_none")] pub strIngredient1: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strIngredient2: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strIngredient3: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strIngredient4: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strIngredient5: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strIngredient6: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strIngredient7: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strIngredient8: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strIngredient9: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strIngredient10: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strIngredient11: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strIngredient12: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strIngredient13: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strIngredient14: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strIngredient15: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strIngredient16: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strIngredient17: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strIngredient18: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strIngredient19: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strIngredient20: Option<String>,

    #[serde(deserialize_with = "empty_is_none")] pub strMeasure1: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strMeasure2: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strMeasure3: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strMeasure4: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strMeasure5: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strMeasure6: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strMeasure7: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strMeasure8: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strMeasure9: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strMeasure10: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strMeasure11: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strMeasure12: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strMeasure13: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strMeasure14: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strMeasure15: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strMeasure16: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strMeasure17: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strMeasure18: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strMeasure19: Option<String>,
    #[serde(deserialize_with = "empty_is_none")] pub strMeasure20: Option<String>,
}