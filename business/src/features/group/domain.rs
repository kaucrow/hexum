pub use crate::features::recipe::{
    RecipePreview, RecipeOrigin
};

use crate::{
    prelude::*,
};

pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_by_id: Uuid,
}

impl Group {
    pub fn new(name: String, description: Option<String>, created_by_id: Uuid) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name,
            description,
            created_by_id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecipesGroup {
    pub group_id: Uuid,
    pub group_name: String,
    pub recipes: Vec<RecipePreview>,
    pub total_recipes: usize,
}