use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_s3 as s3;
use mockall::automock;
use s3::error::SdkError;
use s3::operation::put_object::{PutObjectError, PutObjectOutput};
use s3::primitives::ByteStream;

#[derive(Clone)]
pub struct S3Uploader {
    inner: s3::Client,
    pub bucket_name: &'static str,
}

#[automock]
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
            inner: s3::Client::new(&config),
            bucket_name,
        }
    }
    pub async fn upload(
        &self,
        buffer: Vec<u8>,
        key: &str,
    ) -> Result<PutObjectOutput, SdkError<PutObjectError>> {
        let body = ByteStream::from(buffer);
        self.inner
            .put_object()
            .bucket(self.bucket_name)
            .key(key)
            .body(body)
            .send()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_upload() {
        let key = "test-key";
        let etag = "test-etag";
        let buffer = vec![1, 2, 3, 4, 5];

        let mut mock = MockS3Uploader::default();

        mock.expect_upload()
            .with(eq(buffer.clone()), eq(key))
            .return_once(|_, _| Ok(PutObjectOutput::builder().e_tag(etag.to_string()).build()));

        let result = mock.upload(buffer, key).await;
        assert_eq!(result.unwrap().e_tag(), Some(etag));
    }
}
