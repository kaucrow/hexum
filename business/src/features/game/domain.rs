use crate::prelude::*;

/// Preview game item for popular games listing.
/// `id` is `Some` if the game exists in the internal DB, `None` if from external API.
#[derive(Debug, Clone)]
pub struct PopularGameItem {
    pub id: Option<Uuid>,
    pub external_id: u64,
    pub name: String,
    pub cover_url: Option<String>,
}

pub struct Game {
    pub id: Uuid,
    pub external_id: Option<u64>,
    pub name: String,
    pub first_release_date: Option<DateTime<Utc>>,
    pub cover_url: Option<String>,
    pub rating: Option<f64>,
    pub aggregated_rating: Option<f64>,
    pub total_rating_count: Option<i32>,
    pub summary: Option<String>,
    pub platforms: Vec<Platform>,
    pub genres: Vec<Genre>,
    pub companies: Vec<GameCompany>,
}

pub struct Platform {
    pub id: Uuid,
    pub external_id: Option<u64>,
    pub name: String,
    pub generation: Option<u8>,
}

pub struct Genre {
    pub id: Uuid,
    pub external_id: Option<u64>,
    pub name: String,
}

pub struct Company {
    pub id: Uuid,
    pub external_id: Option<u64>,
    pub name: String,
}

pub struct GameCompany {
    pub company: Company,
    pub role: CompanyRole,
}

pub enum CompanyRole {
    Developer,
    Publisher,
}