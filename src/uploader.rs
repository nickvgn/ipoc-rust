use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_s3 as s3;
use s3::error::SdkError;
use s3::operation::put_object::{PutObjectError, PutObjectOutput};
use s3::primitives::ByteStream;

pub struct S3Uploader {
    client: s3::Client,
    bucket_name: &'static str,
}

impl S3Uploader {
    pub async fn new(bucket_name: &'static str) -> S3Uploader {
        let region_provider = RegionProviderChain::default_provider().or_else("ap-southeast-1");

        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            // loads from aws cli profile
            // NOTE: need to use aws_config::load_from_env() to load from env variables instead
            .profile_name("dev")
            .load()
            .await;

        S3Uploader {
            client: s3::Client::new(&config),
            bucket_name,
        }
    }
}

#[trait_variant::make(IntFactory: Send)]
pub trait Upload {
    async fn upload(
        &self,
        buffer: Vec<u8>,
        key: &str,
    ) -> Result<PutObjectOutput, SdkError<PutObjectError>>;
}

impl Clone for S3Uploader {
    fn clone(&self) -> Self {
        S3Uploader {
            client: self.client.clone(),
            bucket_name: self.bucket_name,
        }
    }
}

// NOTE: this can just be in the impl block; but im just being fancy here to f around with rust
impl Upload for S3Uploader {
    async fn upload(
        &self,
        buffer: Vec<u8>,
        key: &str,
    ) -> Result<PutObjectOutput, SdkError<PutObjectError>> {
        let body = ByteStream::from(buffer);
        self.client
            .put_object()
            .bucket(self.bucket_name)
            .key(key)
            .body(body)
            .send()
            .await
    }
}

// NOTE: i just use this to test connection to aws cli
async fn _show_buckets(client: s3::Client, strict: bool, region: &str) -> Result<(), s3::Error> {
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
