mod service;
use actix_web::{self, App, HttpServer};
#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || App::new().service(service::post_file))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
