#[actix_web::main]
async fn main() -> std::io::Result<()> {
    ipoc_rust::run()?.await
}
