use std::env;

#[tokio::test]
async fn test_sync() {
    let (url, temp_dir) = crate::tests::helpers::spawn_test_app().await;
    let temp_save_dir = tempfile::tempdir().unwrap();
    let config = client::ClientConfig { api_url: url };
    let mut client = client::Client::new(config);
    let entry_result = client.create_entry().await;
    let uuid = entry_result.unwrap();
    let test_file = env::current_dir()
        .unwrap()
        .read_dir()
        .unwrap()
        .filter_map(|entry| {
            if let Ok(entry) = entry
                && let Ok(filetype) = entry.file_type()
                && filetype.is_file()
            {
                Some(entry.file_name())
            } else {
                None
            }
        })
        .next()
        .unwrap();

    client.upload(uuid, test_file).await;
    client.download(uuid, temp_save_dir.path()).await.unwrap();
    temp_dir.close().unwrap();
    temp_save_dir.close().unwrap()
}
