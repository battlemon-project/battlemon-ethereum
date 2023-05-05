use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use ethers::prelude::{Address, Signature, SignatureError};
use eyre::{Report, Result, WrapErr};
use jsonwebtoken::EncodingKey;
use serde::Deserialize;
use sqlx::PgPool;
use thiserror::Error;
use tracing::instrument;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Payload {
    pub user_id: String,
    pub signature: String,
}

pub struct ValidatedPayload {
    pub user_id: Address,
    pub signature: Signature,
}

impl TryFrom<Payload> for ValidatedPayload {
    type Error = String;

    #[instrument(name = "Validating payload", skip_all)]
    fn try_from(Payload { user_id, signature }: Payload) -> Result<Self, Self::Error> {
        let user_id = user_id
            .parse()
            .map_err(|e| format!("Failed to validate user_id: {e}"))?;
        let signature = signature
            .parse()
            .map_err(|e| format!("Failed to validate signature: {e}"))?;

        Ok(Self { user_id, signature })
    }
}

#[instrument(name = "Web3 auth", skip_all, err(Debug))]
pub async fn web3_auth(
    State(db_pool): State<PgPool>,
    State(key): State<EncodingKey>,
    Json(payload): Json<Payload>,
) -> Result<(), AuthError> {
    let ValidatedPayload { user_id, signature } =
        payload.try_into().map_err(AuthError::Validation)?;
    let nonce = get_user_nonce_db(&user_id, &db_pool)
        .await
        .wrap_err("Failed to get nonce for user")?;

    signature
        .verify(nonce.to_string(), user_id)
        .map_err(AuthError::SignatureVerification)?;

    Ok(())
}

#[instrument(name = "Get nonce for user from database", skip(db_pool))]
async fn get_user_nonce_db(user_id: &Address, db_pool: &PgPool) -> Result<Uuid, sqlx::Error> {
    let ret = sqlx::query!(
        r#"
        select nonce from users where user_id = $1
       "#,
        format!("0x{user_id:x}")
    )
    .fetch_one(db_pool)
    .await?;

    Ok(ret.nonce)
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Signature verification error: {0}")]
    SignatureVerification(#[from] SignatureError),
    #[error(transparent)]
    Unexpected(#[from] Report),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        match self {
            AuthError::Validation(e) => (StatusCode::BAD_REQUEST, format!("{e}")),
            AuthError::SignatureVerification(e) => (StatusCode::UNAUTHORIZED, format!("{e}")),
            AuthError::Unexpected(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal server error: {e}"),
            ),
        }
        .into_response()
    }
}
