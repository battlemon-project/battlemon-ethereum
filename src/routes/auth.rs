use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    extract::{Json, State},
    headers::{Authorization, authorization::Bearer},
    http::request::Parts,
    http::StatusCode,
    RequestPartsExt,
    response::{IntoResponse, Response}, TypedHeader,
};
use chrono::Utc;
use ethers::prelude::{Address, Signature, SignatureError};
use eyre::{Report, Result, WrapErr};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use thiserror::Error;
use tracing::instrument;
use uuid::Uuid;

use crate::jwt::Jwt;
use crate::routes::SharedState;

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

    let jwt_token = jwt.encode(user_id_string.clone())?;
    let mut tx = db_pool
        .begin()
        .await
        .wrap_err("Failed to start transaction")?;
    storing_jwt_token_db(&user_id_string, &jwt_token, &mut tx)
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

#[instrument(name = "Storing JWT token in database", skip(tx, jwt_token))]
async fn storing_jwt_token_db(
    user_id: &str,
    jwt_token: &str,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        update users
        set jwt_token = $1
        where user_id = $2
        "#,
        jwt_token,
        user_id,
    )
    .execute(&mut *tx)
    .await?;

    Ok(())
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

pub struct User(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
    SharedState: FromRef<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let bearer: TypedHeader<Authorization<Bearer>> = parts
            .extract()
            .await
            .map_err(|_| AuthError::InvalidAuthToken)?;

        let SharedState { jwt, .. } = SharedState::from_ref(state);

        let claims = jwt.decode(bearer.token())?;
        if claims.expired() {
            return Err(AuthError::ExpiredAuthToken);
        }

        Ok(User(claims.sub))
    }
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Signature verification error: {0}")]
    SignatureVerification(#[from] SignatureError),
    #[error("Header doesn't contain correct type of auth token")]
    InvalidAuthToken,
    #[error("Expired auth token")]
    ExpiredAuthToken,
    #[error("Unexpected error: {0}")]
    Unexpected(#[from] Report),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        match self {
            AuthError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AuthError::SignatureVerification(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            AuthError::InvalidAuthToken => (StatusCode::BAD_REQUEST, self.to_string()),
            AuthError::ExpiredAuthToken => (StatusCode::UNAUTHORIZED, self.to_string()),
            AuthError::Unexpected(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        }
        .into_response()
    }
}
