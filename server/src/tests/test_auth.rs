use reqwest::StatusCode;

use crate::tests::helpers::*;

#[tokio::test]
async fn test_unauthorized() {
    let (url, temp_dir) = spawn_test_app().await;
    let response = reqwest::Client::new()
        .get(url.join("entry").unwrap())
        .send()
        .await
        .unwrap();
    assert!(response.status() == StatusCode::UNAUTHORIZED);
    temp_dir.close().unwrap()
}

#[tokio::test]
async fn test_authorized() {
    let (url, temp_dir) = spawn_test_app().await;
    let temp_save_dir = tempfile::tempdir().unwrap();
    let test_user = create_test_user(&url, temp_save_dir.path()).await;
    let mut client = create_test_client(url, test_user).await;
    assert!(client.create_entry().await.is_ok());
    temp_dir.close().unwrap();
    temp_save_dir.close().unwrap();
}
