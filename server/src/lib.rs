mod config;
mod service;
#[cfg(test)]
mod tests;

use std::{net::TcpListener};

use actix_web::{App, HttpServer, dev::Server, web::Data};
pub use config::ServerConfig;

pub fn run(config: ServerConfig, listener: TcpListener) -> Result<Server, std::io::Error> {
    let config_data = Data::new(config);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::clone(&config_data))
            .service(service::upload_file)
            .service(service::download_file)
            .service(service::create_entry)
    })
    .listen(listener)?
    .run();
    Ok(server)
}
