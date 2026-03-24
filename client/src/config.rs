use serde::Deserialize;
use url::Url;

#[derive(Deserialize)]
pub struct ClientConfig {
    pub api_url: Url,
}
