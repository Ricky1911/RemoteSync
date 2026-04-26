use std::{net::SocketAddr, path::PathBuf};

use rsa::rand_core::{OsRng, RngCore};
use serde::Deserialize;

#[derive(Clone)]
pub struct ServerConfig {
    pub save_path: PathBuf,
    pub address: SocketAddr,
    pub secret_key: Vec<u8>,
}

#[derive(Deserialize, Clone)]
pub struct ServerConfigBuilder {
    pub save_path: PathBuf,
    pub address: Option<SocketAddr>,
    pub secret_key: Option<String>,
}

impl ServerConfigBuilder {
    pub fn new(save_path: PathBuf) -> Self {
        ServerConfigBuilder {
            save_path,
            address: None,
            secret_key: None,
        }
    }
    pub fn build(self) -> ServerConfig {
        let save_path = self.save_path;
        let address = self.address.unwrap_or(std::net::SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            0,
        ));
        let secret_key = if let Some(key) = self.secret_key {
            key.into_bytes()
        } else {
            let mut key = [0; 32];
            OsRng.fill_bytes(&mut key);
            key.to_vec()
        };
        ServerConfig {
            save_path,
            address,
            secret_key,
        }
    }
}
