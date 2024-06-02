use env_logger::Env;
use ipoc_rust::{constants::BUCKET_NAME, uploader::S3Uploader};
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        // aws logs are too much man
        .filter(Some("aws_config"), log::LevelFilter::Error)
        .init();

    // NOTE: might prefer loading from env instead
    let address = format!("127.0.0.1:{}", 1234);
    let listener = TcpListener::bind(address).expect("Failed to bind to port");

    let s3 = S3Uploader::new(BUCKET_NAME).await;

    ipoc_rust::run(listener, s3).await?.await
}
