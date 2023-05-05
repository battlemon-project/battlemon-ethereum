use battlemon_auth::{config, startup::App, telemetry};
use eyre::{Result, WrapErr};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = telemetry::get_subscriber("auth".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber).wrap_err("Failed to init tracing subscriber")?;
    info!("Loading application config");
    let config = config::load_config().wrap_err("Failed to load app config")?;
    let app = App::build(config).await?;
    app.run_until_stopped().await?;

    Ok(())
}
