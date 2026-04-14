use ::config::{Config, File};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config_ = Config::builder()
        .add_source(File::with_name("config.toml"))
        .build()
        .expect("Error on building config");

    let config: client::config::ClientConfig =
        config_.try_deserialize().expect("Error on deserializing config file");
    let mut client = client::network::Client::init(config).await;
    client.create_entry().await?;
    Ok(())
}
