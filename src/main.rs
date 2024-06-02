use env_logger::Env;
use ipoc_rust::uploader::S3Uploader;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        // aws logs are too much man
        .filter(Some("aws_config"), log::LevelFilter::Error)
        .init();

    // NOTE: might prefer loading from env instead
    let address = format!("127.0.0.1:{}", 1234);
    let listener = TcpListener::bind(address)?;

    let s3 = S3Uploader::new("image-compress-tournament").await;

    ipoc_rust::run(listener, s3).await?.await
}
