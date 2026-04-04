use std::path::PathBuf;

use serde::Deserialize;
use url::Url;

#[derive(Deserialize)]
pub struct ClientConfig {
    pub api_url: Url,
    pub username: String,
    pub password: String,
    pub public_pem: PathBuf,
    pub private_pem: PathBuf,
}
