use actix_web::{dev::Server, middleware::Logger, App, HttpServer};
use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_s3 as s3;

pub mod configuration;
pub mod constants;
pub mod routes;

pub async fn run() -> Result<Server, std::io::Error> {
    let region_provider = RegionProviderChain::default_provider().or_else("ap-southeast-1");

    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        // loads from aws cli profile
        .profile_name("dev")
        .load()
        .await;

    let client = s3::Client::new(&config);

    // to test connection
    show_buckets(true, &client, "ap-southeast-1").await.unwrap();

    // TODO: pass in the client to the app instance as shared data; to be used by each worker thread; possibly need to clone the client

    let server = HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(routes::upload_to_s3)
    })
    .bind("127.0.0.1:1234")?
    .run();

    Ok(server)
}

async fn show_buckets(strict: bool, client: &s3::Client, region: &str) -> Result<(), s3::Error> {
    let resp = client.list_buckets().send().await?;
    let buckets = resp.buckets();
    let num_buckets = buckets.len();

    let mut in_region = 0;

    for bucket in buckets {
        if strict {
            let r = client
                .get_bucket_location()
                .bucket(bucket.name().unwrap_or_default())
                .send()
                .await?;

            if r.location_constraint().unwrap().as_ref() == region {
                println!("{}", bucket.name().unwrap_or_default());
                in_region += 1;
            }
        } else {
            println!("{}", bucket.name().unwrap_or_default());
        }
    }

    println!();
    if strict {
        println!(
            "Found {} buckets in the {} region out of a total of {} buckets.",
            in_region, region, num_buckets
        );
    } else {
        println!("Found {} buckets in all regions.", num_buckets);
    }

    Ok(())
}
