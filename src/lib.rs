use actix_web::web;
use actix_web::{dev::Server, middleware::Logger, App, HttpServer};
use aws::get_s3_client;

pub mod aws;
pub mod configuration;
pub mod constants;
pub mod routes;

pub async fn run() -> Result<Server, std::io::Error> {
    let client = get_s3_client().await;

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(routes::upload_to_s3)
            // Wrap the s3 client in an Arc smart pointer, to share it across threads
            .app_data(web::Data::new(client.clone()))
    })
    .bind("127.0.0.1:1234")?
    .run();

    Ok(server)
}
