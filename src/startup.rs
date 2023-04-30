use crate::config::{DatabaseConfig, MainConfig};
use eyre::{Result, WrapErr};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::instrument;

pub struct App {}

impl App {
    #[instrument(name = "Building Application", skip_all)]
    pub async fn build(config: MainConfig) -> Result<Self> {
        let pool = get_db_pool(&config.db);
        sqlx::migrate!()
            .run(&pool)
            .await
            .wrap_err("Failed to run migrations")?;

        Ok(Self {})
    }
}

#[instrument(name = "Getting database pool", skip_all)]
pub fn get_db_pool(config: &DatabaseConfig) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(config.with_db())
}
