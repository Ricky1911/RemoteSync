use std::{net::SocketAddr, path::PathBuf};

use serde::Deserialize;


#[derive(Deserialize)]
pub struct ServerConfig {
    pub save_path: PathBuf,
    pub address: Option<SocketAddr>
}