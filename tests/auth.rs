use crate::helpers::spawn_app;
use eyre::Result;
use uuid::Uuid;

mod helpers;

#[tokio::test]
async fn web3_auth_works_correctly() -> Result<()> {
    let app = spawn_app().await;
    let user_address = app.user_address();
    let nonce: Uuid = app.get_nonce_for_user(&user_address).await?;
    let signature = app.sign(nonce.to_string().as_str()).await?;

    let jwt = app
        .web3_auth(signature.to_string().as_str(), &user_address)
        .await?;

    Ok(())
}
