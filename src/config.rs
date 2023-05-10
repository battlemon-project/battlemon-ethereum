use base64::Engine;
use eyre::{Result, WrapErr};
use jsonwebtoken::{DecodingKey, EncodingKey};
use ring::signature::{Ed25519KeyPair, KeyPair};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sqlx::postgres::PgConnectOptions;
use strum::{Display, EnumString};

#[derive(Deserialize, Clone, Debug)]
pub struct MainConfig {
    pub app: AppConfig,
    pub db: DatabaseConfig,
    pub secrets: SecretsConfig,
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
            .username(self.username.expose_secret())
            .password(self.password.expose_secret())
            .port(self.port)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.db_name)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct SecretsConfig {
    pub key_pair: Secret<String>,
}

impl SecretsConfig {
    pub fn jwt_keys(&self) -> Result<(EncodingKey, DecodingKey)> {
        let pkcs8v2_keypair_base64_encoded = self.key_pair.expose_secret();
        let key_pair_bytes = base64::engine::general_purpose::STANDARD
            .decode(pkcs8v2_keypair_base64_encoded)
            .wrap_err("Failed to decode keypair")?;
        let key_pair = Ed25519KeyPair::from_pkcs8_maybe_unchecked(&key_pair_bytes)
            .wrap_err("Failed to create `Ed25519KeyPair` from source")?;
        let encoding_key = EncodingKey::from_ed_der(&key_pair_bytes);
        let decoding_key = DecodingKey::from_ed_der(key_pair.public_key().as_ref());

        Ok((encoding_key, decoding_key))
    }
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
