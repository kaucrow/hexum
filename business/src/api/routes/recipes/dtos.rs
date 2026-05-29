use crate::{
    prelude::*,
    api::*,
};

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

    /// The amount of recipes to fetch.
    #[param(example = 10)]
    pub limit: usize,

    /// The search session ID (UUID). Should have a value if the search session exists & be null otherwise.
    #[param(example = json!(null))]
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
    /// UUID.
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
    pub search_id: String,
}