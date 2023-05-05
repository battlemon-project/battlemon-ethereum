use eyre::{Result, WrapErr};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sqlx::postgres::PgConnectOptions;
use strum::{Display, EnumString};

#[derive(Deserialize, Clone, Debug)]
pub struct MainConfig {
    pub app: AppConfig,
    pub db: DatabaseConfig,
    pub jwt: Jwt,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
}

impl AppConfig {
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: Secret<String>,
    pub password: Secret<String>,
    pub db_name: String,
}

impl DatabaseConfig {
    pub fn without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username.expose_secret())
            .password(&self.password.expose_secret())
            .port(self.port)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.db_name)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Jwt {
    pub secret: Secret<String>,
}

pub fn load_config() -> Result<MainConfig> {
    let config_path = std::env::current_dir()
        .wrap_err("Failed to determine the current directory")?
        .join("config");

    let env: Environment = std::env::var("APP_ENV")
        .unwrap_or_else(|_| "local".into())
        .parse()
        .wrap_err("Failed to parse APP_ENV")?;

    let env_filename = format!("{}.toml", env.to_string());
    let config = config::Config::builder()
        .add_source(config::File::from(config_path.join("base.toml")))
        .add_source(config::File::from(config_path.join(env_filename)))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()
        .wrap_err("Failed to build config")?;

    config
        .try_deserialize()
        .wrap_err("Failed to deserialize config files into `Config`")
}

#[derive(Debug, Clone, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum Environment {
    Local,
    Production,
}
