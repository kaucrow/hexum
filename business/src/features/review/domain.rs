use strum::{Display, EnumString};

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct Review {
    pub id: Uuid,
    pub user_id: Uuid,
    pub game_id: Uuid,
    pub review_type: ReviewType,
    pub rating: i32,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Display, EnumString)]
#[serde(rename_all = "lowercase")]
pub enum ReviewType {
    #[strum(serialize = "user")]
    User,
    #[strum(serialize = "critic")]
    Critic,
}