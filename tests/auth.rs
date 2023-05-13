use crate::helpers::spawn_app;
use base64::Engine;
use battlemon_ethereum::jwt::Claims;
use eyre::Result;
use jsonwebtoken::{Algorithm, DecodingKey};
use uuid::Uuid;

mod helpers;

#[tokio::test]
async fn web3_auth_works_correctly() -> Result<()> {
    let app = spawn_app().await;
    let user_address = app.user_address();
    let nonce: Uuid = app.get_nonce_for_user(&user_address).await?;
    let signature = app.sign(nonce.to_string().as_str()).await?;

    let auth_json = app
        .web3_auth(signature.to_string().as_str(), &user_address)
        .await?;

    let jwt = auth_json.get("jwt").unwrap().as_str().unwrap();
    let public_key_base64 = auth_json.pointer("/jwk/x").unwrap().as_str().unwrap();
    let public_key_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(public_key_base64)
        .unwrap();
    let decoding_key = DecodingKey::from_ed_der(&public_key_bytes);
    let _ = jsonwebtoken::decode::<Claims>(
        jwt,
        &decoding_key,
        &jsonwebtoken::Validation::new(Algorithm::EdDSA),
    )
    .unwrap();

    Ok(())
}
