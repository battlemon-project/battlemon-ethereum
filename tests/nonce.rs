mod helpers;

use eyre::{Result, WrapErr};
use helpers::spawn_app;
use std::collections::HashSet;

#[tokio::test]
async fn every_nonce_for_user_is_unique() -> Result<()> {
    let app = spawn_app().await;
    let user_id = app.user_address();
    let mut nonces = HashSet::new();
    let expected_quantity_of_nonce = 10;
    for _ in 0..expected_quantity_of_nonce {
        let nonce = app.get_nonce_for_user(&user_id).await?;
        nonces.insert(nonce);
    }

    assert_eq!(
        expected_quantity_of_nonce,
        nonces.len(),
        "Collision of nonces occurred"
    );

    Ok(())
}

#[tokio::test]
async fn nonces_stored_in_database_correctly() -> Result<()> {
    let app = spawn_app().await;
    let user_id = app.user_address();
    for _ in 0..10 {
        let nonce = app.get_nonce_for_user(&user_id).await?;
        let row = sqlx::query!(
            r#"
            select nonce from users
            where user_id = $1 
            "#,
            user_id
        )
        .fetch_one(&app.db_pool)
        .await
        .wrap_err("Failed to fetch stored nonce from database")?;

        assert_eq!(nonce, row.nonce, "Nonces are not equal");
    }

    Ok(())
}
