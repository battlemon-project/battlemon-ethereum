use axum::{
    extract::FromRef,
    http::Request,
    Router,
    routing::{get, post},
};
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestId, RequestId},
    ServiceBuilderExt,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::{instrument, Level};
use uuid::Uuid;

pub use auth::*;
pub use healthcheck::*;
pub use users::*;

use crate::jwt::Jwt;

mod auth;
mod healthcheck;
mod users;

#[instrument(name = "Setup routes", skip_all)]
pub fn setup_router(state: SharedState) -> Router {
    let request_id_layer = ServiceBuilder::new()
        .set_x_request_id(MakeRequestUuid)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    DefaultMakeSpan::new()
                        .include_headers(true)
                        .level(Level::INFO),
                )
                .on_response(DefaultOnResponse::new().include_headers(true)),
        )
        .propagate_x_request_id();

    Router::new()
        .route("/healthcheck", get(healthcheck))
        .route("/users/:user_id/nonce", get(set_nonce_for_address))
        .route("/web3_auth", post(web3_auth))
        .with_state(state)
        .layer(request_id_layer)
}

#[derive(Clone, FromRef)]
pub struct SharedState {
    pub jwt: Jwt,
    pub db_pool: PgPool,
}

#[derive(Clone)]
struct MakeRequestUuid;

impl MakeRequestId for MakeRequestUuid {
    fn make_request_id<B>(&mut self, _: &Request<B>) -> Option<RequestId> {
        let request_id = Uuid::new_v4().to_string();

        Some(RequestId::new(request_id.parse().unwrap()))
    }
}
