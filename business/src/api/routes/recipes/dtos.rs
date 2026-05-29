use crate::{
    prelude::*,
    api::*,
};

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct RecipeSearchQueryParams {
    /// The recipe's name (partial or complete).
    pub name: String,
    /// The pagination index.
    pub page: usize,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
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
        "totalItems": 100
    }
}))]
pub struct RecipeSearchResponse {
    pub recipes: Vec<RecipeSearchResultItem>,
    pub meta: RecipeSearchMeta,
}

impl RecipeSearchResponse {
    pub fn new(recipes: Vec<RecipeSearchResultItem>, total_items: usize) -> Self {
        Self {
            recipes,
            meta: RecipeSearchMeta {
                total_items,
            }
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct RecipeSearchResultItem {
    /// UUID for local & String for external
    pub id: String,
    /// "local" (from DB) or "external" (from API)
    pub origin: String,
    pub name: String,
    pub tags: Vec<String>,
    pub thumbnail_url: Option<String>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecipeSearchMeta {
    pub total_items: usize,
}