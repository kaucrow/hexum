use crate::prelude::*;
use strum::{Display, EnumString};
use local_ip_address::local_ip;
use anyhow::{Result, anyhow};

#[derive(Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub debug: bool,
    #[serde(default)]
    pub environment: Environment,
    pub api: internal::ApiConfig,
    pub frontend: internal::FrontendConfig,
    #[serde(rename = "postgresql")]
    pub postgres: internal::PostgresConfig,
    pub redis: internal::RedisConfig,
    pub email: internal::EmailConfig,
    pub oauth: internal::OAuthConfig,

    pub external_api: internal::ExternalApiConfig,
}

#[derive(Deserialize, Clone, Debug, Display, Default, PartialEq, EnumString)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    #[default]
    Development,
    Testing,
    Production,
}

#[derive(Deserialize, Clone, Debug, Display, Default)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ApiProtocol {
    #[default]
    Http,
    Https,
}

#[derive(Deserialize, Clone)]
#[serde(tag = "provider", rename_all = "lowercase")]
pub enum EmailSender {
    Smtp(self::internal::SmtpConfig),
    Resend(self::internal::ResendConfig),
}

pub fn get_config() -> Result<Config, config::ConfigError> {
    let root_path = get_root_path();

    let environment: String = std::env::var("APP_ENV")
        .unwrap_or_else(|_| "development".into());

    let config_directory = root_path.join(format!("config/{}", environment));

    let filename = "base.toml";

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

    let mut app_config = settings.try_deserialize::<Config>()?;

    app_config.environment = environment
        .to_lowercase()
        .parse()
        .unwrap_or_default();

    if environment == "production" {
        app_config.api.protocol = ApiProtocol::Https;
    } else {
        let local_ip = local_ip().unwrap_or("127.0.0.1".parse().unwrap()).to_string();

        app_config.api.protocol = ApiProtocol::Http;
        app_config.api.domain = local_ip.clone();
    }

    Ok(app_config)
}

mod internal {
    use super::*;

    #[derive(Deserialize, Clone)]
    pub struct ApiConfig {
        pub enable_dev_endpoints: bool,
        pub host: String,
        pub domain: String,
        pub path_suffix: String,
        pub port: u16,
        #[serde(default)]
        pub protocol: ApiProtocol,
        pub docs_endpoint: String,
        pub docs_users: HashMap<String, String>,
    }

    impl ApiConfig {
        pub fn url(&self) -> String {
            match self.protocol {
                ApiProtocol::Http => format!("http://{}{}:{}/", self.domain, self.path_suffix, self.port),
                ApiProtocol::Https => format!("https://{}{}/", self.domain, self.path_suffix),
            }
        }
    }

    #[derive(Deserialize, Clone)]
    pub struct FrontendConfig {
        pub url: String,
    }

    #[derive(Deserialize, Clone)]
    pub struct PostgresConfig {
        pub pool_max_conn: u32,
        pub host: String,
        pub port: u16,
        pub user: String,
        pub passwd: String,
        pub name: String,
    }

    impl PostgresConfig {
        pub fn url(&self) -> String {
            format!(
                "postgresql://{}:{}@{}:{}/{}",
                self.user, self.passwd, self.host, self.port, self.name
            )
        }
    }

    #[derive(Deserialize, Clone)]
    pub struct RedisConfig {
        pub host: String,
        pub port: u16,
        pub passwd: String,
        pub number: u32,
    }

    impl RedisConfig {
        pub fn url(&self) -> String {
            if self.passwd.is_empty() {
                format!("redis://{}:{}/{}", self.host, self.port, self.number)
            } else {
                format!(
                    "redis://:{}@{}:{}/{}",
                    self.passwd, self.host, self.port, self.number
                )
            }
        }
    }

    #[derive(Deserialize, Clone)]
    pub struct EmailConfig {
        #[serde(flatten)]
        pub sender: EmailSender,
        pub from: String,
    }

    impl EmailSender {
        pub fn smtp_config(&self) -> Result<&SmtpConfig> {
            match self {
                EmailSender::Smtp(cfg) => Ok(cfg),
                EmailSender::Resend(_) => Err(anyhow!("Expected SMTP configuration, but Resend was provided.")),
            }
        }

        pub fn resend_config(&self) -> Result<&ResendConfig> {
            match self {
                EmailSender::Smtp(_) => Err(anyhow!("Expected Resend configuration, but SMTP was provided.")),
                EmailSender::Resend(cfg) => Ok(cfg),
            }
        }
    }

    #[derive(Deserialize, Clone)]
    pub struct SmtpConfig {
        pub host: String,
        pub port: u16,
        pub user: String,
        pub passwd: String,
    }

    #[derive(Deserialize, Clone)]
    pub struct ResendConfig {
        pub api_key: String,
    }

    #[derive(Deserialize, Clone)]
    pub struct OAuthConfig {
        pub login_ui_endpoint: String,
        pub callback_endpoint: String,
        pub google: GoogleConfig,
        pub github: GitHubConfig,
    }

    impl OAuthConfig {
        pub fn login_ui_url(&self, frontend_url: &str) -> String {
            format!("{}{}", frontend_url, self.login_ui_endpoint)
        }

        pub fn redirect_url(&self, frontend_url: &str) -> String {
            format!("{}{}", frontend_url, self.callback_endpoint)
        }
    }

    #[derive(Deserialize, Clone)]
    pub struct GoogleConfig {
        pub login_endpoint: String,
        pub client_id: String,
        pub client_secret: String,
    }

    #[derive(Deserialize, Clone)]
    pub struct GitHubConfig {
        pub login_endpoint: String,
        pub client_id: String,
        pub client_secret: String,
    }

    #[derive(Deserialize, Clone)]
    pub struct ExternalApiConfig {
        pub url: String,
        pub key: String,
    }
}