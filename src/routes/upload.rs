use std::time;

use actix_web::HttpResponse;
use actix_web::{web, Responder};
use cpu_time::ProcessTime;
use futures_util::TryStreamExt;
use serde::Serialize;

use crate::processor::ImageProcessor;
use crate::uploader::S3Uploader;
use crate::uploader::Upload;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UploadResponse {
    s3_url: String,
}

// TODO: Custom error handling

pub async fn upload(mut body: web::Payload, s3: web::Data<S3Uploader>) -> impl Responder {
    let start = ProcessTime::try_now().expect("Getting start time failed");

    let mut bytes = web::BytesMut::new();
    // Streaming data
    while let Ok(Some(chunk)) = body.try_next().await {
        bytes.extend_from_slice(&chunk);
    }

    // NOTE: need to change this to a more unique name?
    let file_name = format!(
        "{}.jpg",
        time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    let buffer = ImageProcessor::new(bytes, &file_name).resize();

    let buffer = match buffer {
        Ok(buffer) => buffer,
        Err(e) => {
            log::error!("Error resizing image: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    match s3.upload(buffer, &file_name).await {
        Ok(_) => {
            let elapsed_time = start.try_elapsed().expect("Getting elapsed time failed");

            log::info!(
                "Total time: {:?} for thread: {:?}",
                elapsed_time,
                std::thread::current().id()
            );

            let upload_response = UploadResponse {
                s3_url: format!(
                    "https://{}.s3.{}.amazonaws.com/{}",
                    s3.bucket_name, "ap-southeast-1", file_name,
                ),
            };

            HttpResponse::Ok().json(upload_response)
        }
        Err(e) => {
            log::error!("Error uploading image: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
