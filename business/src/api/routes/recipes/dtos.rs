use crate::{
    prelude::*,
    api::*,
};

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct RecipeSearchQueryParams {
    /// The recipe's name (partial or complete).
    pub name: String,
    /// The pagination index.
    pub page: usize,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "id": "52771",
    "origin": "external",
    "name": "Spaghetti Carbonara",
    "tags": ["Pasta", "Italian"],
    "thumbnailUrl": "https://www.themealdb.com/images/media/meals/llc9is1557421634.jpg",
}))]
pub struct RecipeSearchResponse {
    /// UUID for local & String for external
    pub id: String,
    /// "local" (from DB) or "external" (from API)
    pub origin: String,
    pub name: String,
    pub tags: Vec<String>,
    pub thumbnail_url: Option<String>,
}