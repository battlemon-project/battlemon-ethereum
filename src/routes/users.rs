use crate::address::ToHex;
use crate::routes::{json_error, json_success, JsonResponse};
use axum::{
    extract::Path,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use ethers::prelude::Address;
use eyre::WrapErr;
use sqlx::{PgPool, Postgres, Transaction};
use thiserror::Error;
use tracing::{error, instrument};
use uuid::Uuid;

#[instrument(name = "Set nonce endpoint handler", err(Debug), skip(db_pool))]
pub async fn set_nonce_for_address(
    Path(user_id): Path<String>,
    State(db_pool): State<PgPool>,
) -> Result<impl IntoResponse, UserError> {
    let nonce = Uuid::new_v4();
    let user_id: Address = user_id.parse().wrap_err("Failed to parse user id")?;
    let mut tx = db_pool
        .begin()
        .await
        .wrap_err("Failed to start sql transaction")?;

    upsert_nonce_for_user_db(user_id.to_hex(), &nonce, &mut tx)
        .await
        .wrap_err("Failed to upsert nonce for user")?;

    tx.commit()
        .await
        .wrap_err("Failed to commit sql transaction")?;

    Ok(json_success(nonce))
}

#[instrument(name = "Store nonce for address into database", skip(tx))]
async fn upsert_nonce_for_user_db(
    user_id: String,
    nonce: &Uuid,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        insert into users(user_id, nonce)
        values ($1, $2)
        on conflict (user_id)
        do update set nonce = $2
        "#,
        user_id,
        nonce,
    )
    .execute(&mut *tx)
    .await?;

    Ok(())
}

#[derive(Error, Debug)]
pub enum UserError {
    #[error("Internal server error: {0}")]
    UnexpectedError(#[from] eyre::Report),
}

impl IntoResponse for UserError {
    fn into_response(self) -> Response {
        let status_code = match self {
            UserError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status_code, json_error(self)).into_response()
    }
}
