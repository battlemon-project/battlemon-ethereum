mod helpers;

use std::collections::HashSet;
use uuid::Uuid;

use helpers::spawn_app;

#[tokio::test]
async fn nonce_for_user_works_correctly_many_times() {
    let app = spawn_app().await;
    let user_id = app.test_user.id();
    let mut nonces = HashSet::new();
    let expected_quantity_of_nonce = 10;
    for _ in 0..expected_quantity_of_nonce {
        let response = app
            .get(&format!("users/{user_id}/nonce"), None)
            .await
            .unwrap();

        let status = response.status();
        assert!(
            status.is_success(),
            r#"
        Failed to get nonce for user `{user_id}`.
        The status of response is `{status}`.
        Error body: {}
        "#,
            response.text().await.unwrap()
        );

        let nonce: Uuid = response
            .json()
            .await
            .expect("Failed to deserialize body into `Uuid`");

        nonces.insert(nonce);
    }

    assert_eq!(
        expected_quantity_of_nonce,
        nonces.len(),
        "Collision of nonces occurred"
    );
}

#[tokio::test]
async fn nonces_stored_in_database_correctly() {
    let app = spawn_app().await;
    let user_id = app.test_user.id();
    let expected_quantity_of_nonce = 10;
    for _ in 0..expected_quantity_of_nonce {
        let response = app
            .get(&format!("users/{user_id}/nonce"), None)
            .await
            .unwrap();

        let status = response.status();
        assert!(
            status.is_success(),
            r#"
        Failed to get nonce for user `{user_id}`.
        The status of response is `{status}`.
        Error body: {}
        "#,
            response.text().await.unwrap()
        );

        let expected_nonce: Uuid = response
            .json()
            .await
            .expect("Failed to deserialize body into `Uuid`");

        let row = sqlx::query!(
            r#"
            select nonce from users
            where user_id = $1 
            "#,
            app.test_user.id()
        )
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch stored nonce from database");

        assert_eq!(expected_nonce, row.nonce, "Nonces are not equal");
    }
}
