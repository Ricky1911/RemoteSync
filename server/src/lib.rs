pub mod config;
mod middleware;
mod schema;
mod service;
#[cfg(test)]
mod tests;

use actix_web::{App, HttpServer, dev::Server, web::Data};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use dotenv::dotenv;
use std::net::TcpListener;

type DbPool = Pool<ConnectionManager<PgConnection>>;

fn build_dp_pool(database_url: &str) -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .expect("unable to connect to database")
}

pub fn run(
    config: crate::config::ServerConfig,
    listener: TcpListener,
) -> Result<Server, std::io::Error> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").unwrap();
    let db_pool = Data::new(build_dp_pool(&database_url));
    let config_data = Data::new(config);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(crate::middleware::AuthMiddleware {
                secret_key: config_data.secret_key.clone(),
            })
            .app_data(Data::clone(&config_data))
            .app_data(Data::clone(&db_pool))
            .service(service::login)
            .service(service::post_user)
            .service(service::upload_file)
            .service(service::download_file)
            .service(service::create_entry)
            .service(service::query_update)
    })
    .listen(listener)?
    .run();
    Ok(server)
}
