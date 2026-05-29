use std::collections::BTreeMap;

use crate::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub tags: Vec<String>,
    pub ingredients: BTreeMap<String, String>,  // Ingredients' names & measures
    pub instructions: String,
    pub thumbnail_url: Option<String>,
    pub video_url: Option<String>,
}