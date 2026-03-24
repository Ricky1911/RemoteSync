use ::config::{Config, File};

#[tokio::main]
async fn main() {
    let config_ = Config::builder()
        .add_source(File::with_name("config.toml"))
        .build()
        .expect("构建配置错误");

    let config: client::ClientConfig = config_.try_deserialize().expect("反序列化配置文件错误");
    let mut client = client::Client::new(config);
    let _ = client.create_entry().await;
}
