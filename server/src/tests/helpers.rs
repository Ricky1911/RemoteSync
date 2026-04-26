use fake::{Fake as _, Faker};
use std::{
    net::TcpListener,
    path::{Path, PathBuf},
};
use tempfile::TempDir;
use url::Url;
use uuid::Uuid;

pub async fn spawn_test_app() -> (Url, TempDir) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let temp_dir = tempfile::tempdir().unwrap();
    let config = crate::config::ServerConfigBuilder::new(temp_dir.path().to_path_buf()).build();
    let server = crate::run(config, listener).expect("failed to bind address");
    let _ = tokio::spawn(server);
    (
        Url::parse(&format!("http://127.0.0.1:{}", port)).unwrap(),
        temp_dir,
    )
}

pub struct TestUser {
    pub username: String,
    pub password: String,
    pub public_pem: PathBuf,
    pub private_pem: PathBuf,
}

pub async fn create_test_user(url: &Url, temp_dir: &Path) -> TestUser {
    let username: String = Faker.fake();
    let password: String = Faker.fake();
    let public_pem: PathBuf = temp_dir.join(Uuid::new_v4().to_string());
    let private_pem: PathBuf = temp_dir.join(Uuid::new_v4().to_string());
    client::network::create_user(
        username.clone(),
        password.clone(),
        url,
        public_pem.as_path(),
        private_pem.as_path(),
    )
    .await
    .unwrap();
    TestUser {
        username,
        password,
        public_pem,
        private_pem,
    }
}

pub async fn create_test_client(api_url: Url, test_user: TestUser) -> client::network::Client {
    let config = client::config::ClientConfig {
        api_url,
        username: test_user.username,
        password: test_user.password,
        public_pem: test_user.public_pem,
        private_pem: test_user.private_pem,
    };
    client::network::Client::init(config).await
}
