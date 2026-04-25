use std::net::TcpListener;

use config::{Config, File};
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config_ = Config::builder()
        .add_source(File::with_name("config.toml"))
        .build()
        .expect("Error on building config");

    let config: server::ServerConfig = config_
        .try_deserialize()
        .expect("Error on deserializing config file");
    let address = config.address;
    let listener = TcpListener::bind(address)?;
    println!("Server started, listening to {}", listener.local_addr()?);
    server::run(config, listener)?.await
}
