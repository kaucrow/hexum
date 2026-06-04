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
    get_queries()
        .map_err(|e| {
            error!("{:?}", e);
            e
        })
        .expect("Failed to initialize business postgres queries.")
});

#[derive(Deserialize, Debug)]
pub(crate) struct Queries {
    pub base: internal::Base,
    pub data_ingestion: internal::DataIngestion,
    pub recipe: internal::Recipe,
    pub tag: internal::Tag,
    pub group: internal::Group,
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
        pub get_search_ids_by_query: String,
        pub get_search_ids_by_query_ilike: String,
        pub get_search_ids_by_tags: String,
        pub get_search_ids_by_query_and_tags: String,
        pub get_search_ids_by_query_and_tags_ilike: String,

        pub get_previews_by_id: String,

        pub get_random_previews: String,

        pub get_latest_previews: String,

        pub get_previews_by_tag_name: String,

        pub get_by_id: String,

        pub create: String,
        pub batch_insert_tags: String,
        pub batch_insert_ingredients: String,

        pub get_latest_history_by_user_id: String,
        pub record_history: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct Tag {
        pub get_search_matches: String,
        pub get_top_tag_names: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct Group {
        pub get_with_latest_recipes_by_user_id: String,
        pub get_recipes_by_group_id: String,

        pub get_by_id: String,
        pub get_by_user_id: String,

        pub create: String,
        pub delete: String,

        pub add_recipe: String,
        pub delete_recipe: String,
    }
}