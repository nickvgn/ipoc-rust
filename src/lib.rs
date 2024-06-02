pub mod configuration;
pub mod processor;
pub mod routes;
pub mod uploader;

use actix_web::web;
use actix_web::{dev::Server, middleware::Logger, App, HttpServer};
use uploader::S3Uploader;

pub async fn run(
    listener: std::net::TcpListener,
    s3: S3Uploader,
) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/upload", web::post().to(routes::upload_to_s3))
            // Wrap the s3 client in an Arc smart pointer, to share it across threads
            .app_data(web::Data::new(s3.clone()))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
