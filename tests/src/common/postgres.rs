use std::path::PathBuf;
use std::sync::LazyLock;

use serde::Deserialize;

pub(crate) static QUERIES: LazyLock<Queries> = LazyLock::new(|| {
    get_queries().expect("Failed to initialize test postgres queries.")
});

#[derive(Deserialize, Debug)]
pub(crate) struct Queries {
    pub common: self::internal::Common,
}

fn get_queries() -> Result<Queries, config::ConfigError> {
    let queries_directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("postgres");
    let filename = "queries.yaml";

    let settings = config::Config::builder()
        .add_source(config::File::from(queries_directory.join(filename)))
        .build()?;

    settings.try_deserialize::<Queries>()
}

mod internal {
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct Common {
        pub delete_user_by_email: String,
        pub get_user_id_by_email: String,
        pub is_user_verified: String,
        pub insert_user: String,
        pub insert_authenticator: String,
    }
}