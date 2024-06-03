pub mod configuration;
pub mod constants;
pub mod processor;
pub mod routes;
pub mod uploader;

use std::net::TcpListener;

use actix_web::web;
use actix_web::{dev::Server, middleware::Logger, App, HttpServer};

use uploader::S3Uploader;

pub async fn run(listener: TcpListener, s3: S3Uploader) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/health-check", web::get().to(routes::health_check))
            .route("/upload", web::post().to(routes::upload))
            // Set max payload size to 20 MB
            .app_data(web::PayloadConfig::new(12 * 1024 * 1024))
            // Wrap the s3 client in an Arc smart pointer, to share it across threads
            .app_data(web::Data::new(s3.clone()))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
