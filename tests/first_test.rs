mod helpers;
use eyre::{ensure, Result};

use helpers::spawn_app;

#[tokio::test]
async fn healthcheck() -> Result<()> {
    let app = spawn_app().await;
    let response = app.get("healthcheck", None).await?;
    assert!(
        response.status().is_success(),
        "The status of response for healthcheck is not 200 OK"
    );

    Ok(())
}
