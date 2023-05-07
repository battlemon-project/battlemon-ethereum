use chrono::{Duration, Utc};
use eyre::{Result, WrapErr};
use jsonwebtoken::{DecodingKey, EncodingKey};

use crate::routes::Claims;

#[derive(Clone)]
pub struct Jwt {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl Jwt {
    pub fn new(key: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(key.as_bytes()),
            decoding_key: DecodingKey::from_secret(key.as_bytes()),
        }
    }

    pub fn encode(&self, user_id: String) -> Result<String> {
        let now = Utc::now();
        let expires_at = now + Duration::hours(1);
        let claims = Claims {
            sub: user_id,
            exp: expires_at.timestamp(),
            iat: now.timestamp(),
        };

        jsonwebtoken::encode(&Default::default(), &claims, &self.encoding_key)
            .wrap_err("Failed to encode claims")
    }

    pub fn decode(&self, token: &str) -> Result<Claims> {
        jsonwebtoken::decode(token, &self.decoding_key, &Default::default())
            .map(|decoded| decoded.claims)
            .wrap_err("Failed to decode token")
    }
}
