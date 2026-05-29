pub(crate) use sqlx::{FromRow, Postgres, Transaction, postgres::PgPool};
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
    pub base: internal::Base,
    pub data_ingestion: internal::DataIngestion,
    pub recipe: internal::Recipe,
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
    pub struct DataIngestion {
        pub sync_recipes: String,
        pub sync_tags: String,
        pub sync_ingredients: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct Recipe {
        pub search_many_by_name: String,
        pub get_many_by_id: String,
    }
}