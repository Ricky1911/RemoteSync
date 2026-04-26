use fake::{Fake as _, Faker};

use uuid::Uuid;

use crate::tests::helpers::*;

#[tokio::test]
async fn test_upload_and_download() {
    let (url, temp_dir) = spawn_test_app().await;
    let temp_save_dir = tempfile::tempdir().unwrap();
    let test_user = create_test_user(&url, temp_save_dir.path()).await;
    let mut client = create_test_client(url, test_user).await;
    let entry_result = client.create_entry().await;
    let uuid = entry_result.unwrap();

    let test_file = temp_save_dir.path().join(Uuid::new_v4().to_string());
    let data: Vec<u8> = std::iter::repeat_with(|| 0u8)
        .take(1024 * 1024 * 200)
        .collect();
    std::fs::write(&test_file, &data).unwrap();
    let key = client::file::crypto::generate_aes_keys();
    client.upload(uuid, test_file, &key).await.unwrap();
    let (path, key_return) = client.download(uuid, temp_save_dir.path()).await.unwrap();
    assert!(key == key_return);
    let data_download = std::fs::read(path).unwrap();
    assert!(data == data_download);

    let test_file = temp_save_dir.path().join(Uuid::new_v4().to_string());
    let data: Vec<u8> = Faker.fake();
    std::fs::write(&test_file, &data).unwrap();
    let key = client::file::crypto::generate_aes_keys();
    client.upload(uuid, test_file, &key).await.unwrap();
    let (path, key_return) = client.download(uuid, temp_save_dir.path()).await.unwrap();
    assert!(key == key_return);
    let data_download = std::fs::read(path).unwrap();
    assert!(data == data_download);

    temp_dir.close().unwrap();
    temp_save_dir.close().unwrap()
}
