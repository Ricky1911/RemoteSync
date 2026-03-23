mod models;
mod service;
use actix_web::{self, App, HttpServer, web::Data};
use config::{Config, File};
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config_ = Config::builder()
        .add_source(File::with_name("config.toml"))
        .build()
        .expect("构建配置错误");

    let config: models::AppConfig = config_.try_deserialize().expect("反序列化配置文件错误");
    let config_data = Data::new(config);
    HttpServer::new(move || {
        App::new()
            .app_data(Data::clone(&config_data))
            .service(service::post_file)
            .service(service::get_file)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
