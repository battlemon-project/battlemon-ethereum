use chrono::{Duration, Utc};
use eyre::Report;
use jsonwebtoken::Header;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
}

#[derive(Error, Debug)]
pub enum JwtError {
    #[error(transparent)]
    UnexpectedError(#[from] Report),
}

pub fn encode() -> Result<String, JwtError> {
    let now = Utc::now();
    let expires_at = now + Duration::hours(1);
    let claims = Claims {
        exp: expires_at.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let jwt = jsonwebtoken::encode(&Header::default(), &claims, )
    todo!()
}

pub fn verify() -> Result<(), JwtError> {
    todo!()
}
