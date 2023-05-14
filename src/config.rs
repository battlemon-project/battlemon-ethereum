use crate::jwt::Jwt;
use base64::Engine;
use eyre::{Result, WrapErr};
use jsonwebtoken::{
    jwk::{
        AlgorithmParameters, CommonParameters, EllipticCurve, Jwk, OctetKeyPairParameters,
        OctetKeyPairType, PublicKeyUse,
    },
    DecodingKey, EncodingKey,
};
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
    pub fn jwt(&self) -> Result<Jwt> {
        let pkcs8v2_keypair_base64_encoded = self.key_pair.expose_secret();
        let key_pair_bytes = base64::engine::general_purpose::STANDARD
            .decode(pkcs8v2_keypair_base64_encoded)
            .wrap_err("Failed to decode keypair")?;
        let key_pair = Ed25519KeyPair::from_pkcs8_maybe_unchecked(&key_pair_bytes)
            .wrap_err("Failed to create `Ed25519KeyPair` from source")?;
        let encoding_key = EncodingKey::from_ed_der(&key_pair_bytes);
        let public_key = key_pair.public_key().as_ref();
        let decoding_key = DecodingKey::from_ed_der(public_key);
        let base64encoded_public_key =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(public_key);
        let jwk = Jwk {
            common: CommonParameters {
                public_key_use: Some(PublicKeyUse::Signature),
                ..Default::default()
            },
            algorithm: AlgorithmParameters::OctetKeyPair(OctetKeyPairParameters {
                key_type: OctetKeyPairType::OctetKeyPair,
                curve: EllipticCurve::Ed25519,
                x: base64encoded_public_key,
            }),
        };

        Ok(Jwt::new(encoding_key, decoding_key, jwk))
    }
}

pub fn load_config() -> Result<MainConfig> {
    let config_path = std::env::current_dir()
        .wrap_err("Failed to determine the current directory")?
        .join("config");

    let current_environment: Environment = std::env::var("APP_ENV")
        .unwrap_or_else(|_| "local".to_owned())
        .parse()
        .wrap_err("Failed to parse APP_ENV")?;

    let file_source_setup =
        config::File::from(config_path.join(current_environment.to_string())).required(false);
    let env_vars_source_setup = config::Environment::with_prefix("APP")
        .prefix_separator("_")
        .separator("__");
    let config = config::Config::builder()
        .add_source(file_source_setup)
        .add_source(env_vars_source_setup)
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
