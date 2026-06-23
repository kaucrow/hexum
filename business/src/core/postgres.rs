pub(crate) use sqlx::{
    FromRow,
    postgres::PgPool,
};
pub(crate) use platform::postgres::sql;

use std::sync::LazyLock;

use crate::prelude::*;

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("postgres/migrations")
    .dangerous_set_table_name("_business_migrations")
    .run(pool)
    .await
    .context("Business postgres migrations failed.")?;

    Ok(())
}

pub(crate) static QUERIES: LazyLock<Queries> = LazyLock::new(|| {
    get_queries().expect("Failed to initialize business postgres queries.")
});

#[derive(Deserialize, Debug)]
pub(crate) struct Queries {
    pub base: self::internal::Base,
    pub game: self::internal::Game,
    pub platform: self::internal::Platform,
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
    pub struct Base {
        pub ping: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct Game {
        pub get_full_data: String,
        pub get_many_by_ids: String,
        pub get_external_ids_by_internal_ids: String,
        pub upsert_from_external: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct Platform {
        pub get_all: String,
        pub get_external_ids_by_internal_ids: String,
        pub get_internal_ids_by_external_ids: String,
        pub upsert_many: String,
    }
}