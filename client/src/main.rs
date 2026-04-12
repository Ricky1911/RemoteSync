use ::config::{Config, File};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config_ = Config::builder()
        .add_source(File::with_name("config.toml"))
        .build()
        .expect("构建配置错误");

    let config: client::config::ClientConfig =
        config_.try_deserialize().expect("反序列化配置文件错误");
    let mut client = client::network::Client::init(config).await;
    client.create_entry().await?;
    Ok(())
}
