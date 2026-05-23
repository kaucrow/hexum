pub use sqlx::postgres::{PgPool, PgPoolOptions};

use std::sync::OnceLock;

use crate::prelude::*;

pub static QUERIES: OnceLock<Queries> = OnceLock::new();

#[derive(Deserialize, Debug)]
pub struct Queries {
    pub user: self::internal::User,
    pub user_authenticator: self::internal::UserAuthenticator,
}

pub fn init() -> anyhow::Result<()> {
    let queries = get_queries()?;
    QUERIES.set(queries).expect("Failed to set global queries.");
    Ok(())
}

pub fn get_queries() -> Result<Queries, config::ConfigError> {
    let crate_assets_path = get_crate_assets_path();

    let queries_directory = crate_assets_path.join("postgres");

    let filename = "queries.yaml";

    let settings = config::Config::builder()
        .add_source(config::File::from(
            queries_directory.join(filename),
        ))
        .build()?;

    settings.try_deserialize::<Queries>()
}

mod internal {
    use crate::prelude::*;

    #[derive(Deserialize, Debug)]
    pub struct User {
        pub get_by_id: String,
        pub get_by_username: String,
        pub get_by_email: String,
        pub insert: String,
        pub delete_by_id: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct UserAuthenticator {
        pub get_by_user_id_and_provider: String,
        pub verify_local_by_user_id: String,
        pub insert: String,
    }
}