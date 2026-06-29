use strum::{Display, EnumString};

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct CriticApplication {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: ApplicationStatus,
    pub applied_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewed_by: Option<Uuid>,
    /// Populated when joining with platform.user (for list queries)
    pub username: Option<String>,
    /// Populated when joining with platform.user (for list queries)
    pub email: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Display, EnumString)]
#[serde(rename_all = "lowercase")]
pub enum ApplicationStatus {
    #[strum(serialize = "pending")]
    Pending,
    #[strum(serialize = "approved")]
    Approved,
    #[strum(serialize = "rejected")]
    Rejected,
}