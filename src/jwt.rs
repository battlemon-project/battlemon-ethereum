use chrono::{Duration, Utc};
use eyre::{Result, WrapErr};
use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
}

pub fn encode(key: &EncodingKey) -> Result<String> {
    let now = Utc::now();
    let expires_at = now + Duration::hours(1);
    let claims = Claims {
        exp: expires_at.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    jsonwebtoken::encode(&Header::default(), &claims, key).wrap_err("Failed to encode claims")
}
