use crate::routes::Claims;
use chrono::{Duration, Utc};
use eyre::{Result, WrapErr};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};

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

    pub fn encode(&self) -> Result<String> {
        let now = Utc::now();
        let expires_at = now + Duration::hours(1);
        let claims = Claims {
            exp: expires_at.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        jsonwebtoken::encode(&Header::default(), &claims, &self.encoding_key)
            .wrap_err("Failed to encode claims")
    }

    pub fn decode(&self, token: &str) -> Result<Claims> {
        jsonwebtoken::decode(
            token,
            &self.decoding_key,
            &Validation::new(Algorithm::HS256),
        )
        .map(|decoded| decoded.claims)
        .wrap_err("Failed to decode token")
    }
}
