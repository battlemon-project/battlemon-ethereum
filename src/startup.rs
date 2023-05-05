use crate::config::{DatabaseConfig, MainConfig};
use crate::routes::{setup_router, SharedState};
use axum::{routing::IntoMakeService, Router};
use eyre::{Result, WrapErr};
use hyper::{server::conn::AddrIncoming, Server};
use jsonwebtoken::EncodingKey;
use secrecy::ExposeSecret;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::TcpListener;
use tracing::{info, instrument};

type HyperServer = Server<AddrIncoming, IntoMakeService<Router>>;

#[derive(Debug)]
pub struct App {
    pub server: HyperServer,
    pub port: u16,
}

impl App {
    #[instrument(name = "Building Application", skip_all)]
    pub async fn build(config: MainConfig) -> Result<Self> {
        let db_pool = setup_db_pool(&config.db);
        sqlx::migrate!()
            .run(&db_pool)
            .await
            .wrap_err("Failed to run migrations")?;

        let app_address = config.app.address();
        info!("Binding address - {app_address} for app");
        let listener =
            TcpListener::bind(&app_address).wrap_err("Failed to bind address for app")?;
        let port = listener.local_addr()?.port();
        let shared_secret = EncodingKey::from_secret(config.jwt.secret.expose_secret().as_ref());
        let server = setup_server(listener, db_pool, shared_secret)?;
        Ok(Self { server, port })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    #[tracing::instrument(name = "Starting application", skip_all)]
    pub async fn run_until_stopped(self) -> Result<()> {
        self.server.await.wrap_err("Failed to run server")
    }
}

#[instrument(name = "Setup database pool", skip_all)]
pub fn setup_db_pool(config: &DatabaseConfig) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(config.with_db())
}

#[tracing::instrument(name = "Setup server", skip_all)]
pub fn setup_server(
    listener: TcpListener,
    db_pool: PgPool,
    shared_secret: EncodingKey,
) -> Result<HyperServer> {
    let state = SharedState {
        db_pool,
        shared_secret,
    };

    let router = setup_router(state);
    let server = axum::Server::from_tcp(listener)?.serve(router.into_make_service());

    Ok(server)
}
