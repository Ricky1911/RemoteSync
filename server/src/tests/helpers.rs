use std::net::TcpListener;
use tempfile::TempDir;
use url::Url;

pub async fn spawn_test_app() -> (Url, TempDir) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let temp_dir = tempfile::tempdir().unwrap();
    let config = crate::ServerConfig {
        save_path: temp_dir.path().to_path_buf(),
        address: listener.local_addr().unwrap(),
        secret_key: "test-secret-key".as_bytes().to_vec(),
    };
    let server = crate::run(config, listener).expect("failed to bind address");
    let _ = tokio::spawn(server);
    (
        Url::parse(&format!("http://127.0.0.1:{}", port)).unwrap(),
        temp_dir,
    )
}
