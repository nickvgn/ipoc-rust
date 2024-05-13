use actix_web::{dev::Server, middleware::Logger, App, HttpServer};

pub mod routes;
pub mod constants;
pub mod configuration;

pub fn run() -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(routes::upload_to_s3)
    })
    .bind("127.0.0.1:1234")?
    .run();

    Ok(server)
}

