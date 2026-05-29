use crate::{
    prelude::*,
    api::*,
};

use std::collections::BTreeMap;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Deserialize, IntoParams, ToSchema)]
#[serde(rename_all = "snake_case")]
#[into_params(parameter_in = Path)]
pub struct RecipePathParams {
    /// The Recipe's ID (UUID).
    #[schema(format = "uuid", example = "05639468-710b-44fe-9fc7-372514e95c37")]
    pub id: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "id": "05639468-710b-44fe-9fc7-372514e95c37",
    "origin": "external",
    "name": "Spaghetti Carbonara",
    "description": null,
    "tags": ["Pasta", "Italian"],
    "ingredients": {
        "Spaghetti": "200g",
        "Pancetta": "100g",
        "Egg Yolks": "3 large"
    },
    "instructions": "Cook pasta, fry pancetta, mix everything together with eggs and cheese off the heat.",
    "thumbnailUrl": "https://www.themealdb.com/images/media/meals/llc9is1557421634.jpg",
    "videoUrl": "https://www.youtube.com/watch?v=3AAdKl1UYZs"
}))]
pub struct RecipeResponse {
    /// The Recipe's ID (UUID).
    #[schema(format = "uuid", example = "05639468-710b-44fe-9fc7-372514e95c37")]
    pub id: String,

    /// Source origin system of the recipe, e.g., 'local' or 'external'.
    #[schema(example = "external")]
    pub origin: String,

    /// Full title of the dish.
    pub name: String,

    /// Optional summary text. Will be null if the recipe comes from an external provider.
    pub description: Option<String>,

    /// Categories associated with this dish.
    pub tags: Vec<String>,

    /// Map of ingredient names mapped to their respective required measurements.
    pub ingredients: BTreeMap<String, String>,

    /// Step-by-step cooking directions.
    pub instructions: String,

    /// Direct link to an image asset preview.
    pub thumbnail_url: Option<String>,

    /// Optional link to a video tutorial.
    pub video_url: Option<String>,
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[serde(rename_all = "snake_case")]
#[into_params(parameter_in = Query)]
pub struct RecipeSearchQueryParams {
    /// The recipe's name (partial or complete).
    #[param(example = "spa")]
    pub query: String,

    /// The pagination index.
    #[param(example = 0, minimum = 1)]
    pub page: usize,

    /// The max amount of recipes to fetch.
    #[param(example = 10)]
    pub limit: usize,

    /// The search session ID (UUID). Should have a value if the search session exists & be null otherwise.
    #[param(format = "uuid", example = json!(null))]
    pub search_id: Option<String>,
}

#[derive(Serialize, ToSchema)]
#[schema(example = json!({
    "recipes": [
        {
            "id": "52771",
            "origin": "external",
            "name": "Spaghetti Carbonara",
            "tags": ["Pasta", "Italian"],
            "thumbnailUrl": "https://www.themealdb.com/images/media/meals/llc9is1557421634.jpg",
        }
    ],
    "meta": {
        "totalItems": 100,
        "sessionId": "05639468-710b-44fe-9fc7-372514e95c37",
    }
}))]
pub struct RecipeSearchResponse {
    pub recipes: Vec<RecipeSearchResultItem>,
    pub meta: RecipeSearchMeta,
}

#[derive(Serialize, ToSchema)]
pub struct RecipeSearchResultItem {
    /// The recipe search result's ID (UUID).
    #[schema(format = "uuid")]
    pub id: String,

    /// "local" (from DB) or "external" (from API).
    pub origin: String,

    pub name: String,
    pub tags: Vec<String>,
    pub thumbnail_url: Option<String>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecipeSearchMeta {
    /// Amount of items that match the search.
    pub total_items: usize,

    /// The search session ID (UUID).
    #[schema(format = "uuid")]
    pub search_id: String,
}