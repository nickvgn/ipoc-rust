use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_s3 as s3;

pub async fn get_s3_client() -> s3::Client {
    let region_provider = RegionProviderChain::default_provider().or_else("ap-southeast-1");

    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        // loads from aws cli profile
        .profile_name("dev")
        .load()
        .await;

    s3::Client::new(&config)
}
