use std::collections::BTreeMap;
use uuid::Uuid;
use strum::{Display, EnumString};

use crate::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Recipe {
    pub id: Uuid,
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
    pub id: Uuid,
    pub origin: RecipeOrigin,
    pub name: String,
    pub tags: Vec<String>,
    pub thumbnail_url: Option<String>,
}

#[derive(Serialize, Deserialize, Display, Debug, Clone, EnumString)]
#[derive()]
pub enum RecipeOrigin {
    #[strum(to_string = "local")]
    Local,

    #[strum(to_string = "external")]
    External,
}