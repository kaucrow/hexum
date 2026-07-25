use crate::prelude::*;

#[derive(Deserialize, Clone)]
pub struct Config {}

pub fn get_config() -> Result<Config, config::ConfigError> {
    let root_path = platform::get_root_path();

    let environment: String = std::env::var("APP_ENV")
        .unwrap_or_else(|_| "development".into());

    let config_directory = root_path.join(format!("config/{}", environment));

    let filename = "business.toml";

    let settings = config::Config::builder()
        .add_source(config::File::from(
            config_directory.join(filename),
        ))
        .add_source(
            config::Environment::with_prefix("HEXUM")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    let app_config = settings.try_deserialize::<Config>()?;

    Ok(app_config)
}

mod internal {}