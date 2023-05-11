mod helpers;

use uuid::Uuid;

use helpers::spawn_app;

#[tokio::test]
async fn nonce_for_new_user_works_correctly() {
    let app = spawn_app().await;
    let user_id = app.test_user.id();
    let response = app
        .get(&format!("users/{user_id}/nonce"), None)
        .await
        .unwrap();

    let status = response.status();
    assert!(
        status.is_success(),
        "Failed to get nonce for user `{user_id}`. The status of response is `{status}`.\n
        Error body: {}",
        response.text().await.unwrap()
    );

    let _: Uuid = response.json().await.unwrap();
}
