use std::collections::BTreeMap;
use uuid::Uuid;

use crate::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Recipe {
    pub origin: RecipeOrigin,
    pub name: String,
    pub description: Option<String>,            // No description on external recipes
    pub tags: Vec<String>,
    pub ingredients: BTreeMap<String, String>,  // Ingredients' names & measures
    pub instructions: String,
    pub thumbnail_url: Option<String>,
    pub video_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecipeSearchResult {
    pub origin: RecipeOrigin,
    pub name: String,
    pub tags: Vec<String>,
    pub thumbnail_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RecipeOrigin {
    Local(Uuid),
    External(String),
}