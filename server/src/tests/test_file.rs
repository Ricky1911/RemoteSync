use fake::Fake;
use fake::Faker;
use reqwest::StatusCode;
use std::path::PathBuf;
use uuid::Uuid;

#[tokio::test]
async fn test_sync() {
    let (url, temp_dir) = crate::tests::helpers::spawn_test_app().await;
    let temp_save_dir = tempfile::tempdir().unwrap();
    let username: String = Faker.fake();
    let password: String = Faker.fake();
    let public_pem: PathBuf = temp_save_dir.path().join(Uuid::new_v4().to_string());
    let private_pem: PathBuf = temp_save_dir.path().join(Uuid::new_v4().to_string());
    client::create_user(
        username.clone(),
        password.clone(),
        url.clone(),
        public_pem.clone(),
        private_pem.clone(),
    )
    .await
    .unwrap();
    let config = client::ClientConfig {
        api_url: url,
        username,
        password,
        public_pem,
        private_pem,
    };
    let mut client = client::Client::init(config).await;
    let entry_result = client.create_entry().await;
    let uuid = entry_result.unwrap();
    let test_file = temp_save_dir.path().join(Uuid::new_v4().to_string());
    // let data: Vec<u8> = Faker.fake();
    let data: Vec<u8> = std::iter::repeat_with(|| 0u8)
        .take(1024 * 1024 * 200)
        .collect();
    println!("{}", data.len());
    std::fs::write(&test_file, &data).unwrap();
    client.upload(uuid, test_file).await.unwrap();
    let path = client.download(uuid, temp_save_dir.path()).await.unwrap();
    let data_download = std::fs::read(path).unwrap();
    assert!(data == data_download);
    temp_dir.close().unwrap();
    temp_save_dir.close().unwrap()
}

#[tokio::test]
async fn test_auth() {
    let (url, temp_dir) = crate::tests::helpers::spawn_test_app().await;
    let response = reqwest::Client::new()
        .get(url.join("entry").unwrap())
        .send()
        .await
        .unwrap();
    assert!(response.status() == StatusCode::UNAUTHORIZED);
    temp_dir.close().unwrap()
}
