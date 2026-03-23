use std::path::PathBuf;

use serde::Deserialize;


#[derive(Deserialize)]
pub struct AppConfig {
    pub save_path: PathBuf
}