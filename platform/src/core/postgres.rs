pub(crate) use sqlx::{
    FromRow,
    postgres::PgPool,
};

use std::sync::LazyLock;

use crate::prelude::*;

#[inline(always)]
pub fn sql(query: &String) -> sqlx::AssertSqlSafe<&str> {
    sqlx::AssertSqlSafe(query.as_str())
}

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("postgres/migrations")
    .dangerous_set_table_name("_platform_migrations")
    .run(pool)
    .await
    .context("Platform postgres migrations failed.")?;

    Ok(())
}

pub(crate) static QUERIES: LazyLock<Queries> = LazyLock::new(|| {
    get_queries().expect("Failed to initialize platform postgres queries.")
});

#[derive(Deserialize, Debug)]
pub(crate) struct Queries {
    pub user: self::internal::User,
    pub user_authenticator: self::internal::UserAuthenticator,
}

fn get_queries() -> Result<Queries, config::ConfigError> {
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
        pub update_email: String,
        pub update_data: String,
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