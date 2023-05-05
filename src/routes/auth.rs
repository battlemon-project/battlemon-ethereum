use crate::jwt::Jwt;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use ethers::prelude::{Address, Signature, SignatureError};
use eyre::{Report, Result, WrapErr};
use serde::Deserialize;
use sqlx::{PgPool, Postgres, Transaction};
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
    State(jwt): State<Jwt>,
    State(db_pool): State<PgPool>,
    Json(payload): Json<Payload>,
) -> Result<String, AuthError> {
    let ValidatedPayload { user_id, signature } =
        payload.try_into().map_err(AuthError::Validation)?;

    let user_id_string = format!("0x{user_id:x}");
    let nonce = get_user_nonce_db(&user_id_string, &db_pool)
        .await
        .wrap_err("Failed to get nonce for user")?;

    signature
        .verify(nonce.to_string(), user_id)
        .map_err(AuthError::SignatureVerification)?;

    let jwt_token = jwt.encode()?;
    let mut tx = db_pool
        .begin()
        .await
        .wrap_err("Failed to start transaction")?;
    store_jwt_token_db(&user_id_string, &jwt_token, &mut tx)
        .await
        .wrap_err("Failed to store jwt token into database")?;
    tx.commit().await.wrap_err("Failed to commit transaction")?;

    Ok(jwt_token)
}

#[instrument(name = "Get nonce for user from database", skip(db_pool))]
async fn get_user_nonce_db(user_id: &str, db_pool: &PgPool) -> Result<Uuid, sqlx::Error> {
    let ret = sqlx::query!(
        r#"
        select nonce from users where user_id = $1
       "#,
        user_id
    )
    .fetch_one(db_pool)
    .await?;

    Ok(ret.nonce)
}

#[instrument(name = "Store JWT token in database", skip(tx, jwt_token))]
async fn store_jwt_token_db(
    user_id: &str,
    jwt_token: &str,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<(), sqlx::Error> {
    todo!()
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
