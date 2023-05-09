use chrono::{Duration, Utc};
use eyre::{Result, WrapErr};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Jwt {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl Jwt {
    pub fn new(encoding_key: EncodingKey, decoding_key: DecodingKey) -> Self {
        Self {
            encoding_key,
            decoding_key,
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

        jsonwebtoken::encode(&Header::new(Algorithm::EdDSA), &claims, &self.encoding_key)
            .wrap_err("Failed to encode claims")
    }

    pub fn decode(&self, token: &str) -> Result<Claims> {
        jsonwebtoken::decode(
            token,
            &self.decoding_key,
            &Validation::new(Algorithm::EdDSA),
        )
        .map(|decoded| decoded.claims)
        .wrap_err("Failed to decode token")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

impl Claims {
    pub fn expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }
}
