use actix_web::http::StatusCode;
use actix_web::web;
use actix_web::web::Json;
use actix_web::ResponseError;
use cpu_time::ProcessTime;
use futures_util::TryStreamExt;
use serde::Serialize;
use thiserror::Error;

use crate::processor::ImageProcessor;
use crate::uploader::S3Uploader;
use crate::uploader::Upload;

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("An internal error occurred. Please try again later.")]
    InternalError,
}

impl ResponseError for HttpError {
    fn status_code(&self) -> StatusCode {
        match *self {
            HttpError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadResponse {
    s3_url: String,
}

pub async fn upload(
    mut body: web::Payload,
    s3: web::Data<S3Uploader>,
) -> actix_web::Result<Json<UploadResponse>, HttpError> {
    let start = ProcessTime::try_now().expect("Getting start time failed");

    let mut bytes = web::BytesMut::new();
    // Streaming data
    while let Ok(Some(chunk)) = body.try_next().await {
        bytes.extend_from_slice(&chunk);
    }

    let processor = ImageProcessor::new(bytes);
    let (buffer, format) = processor.resize().map_err(|e| {
        log::error!("Error resizing image: {:?}", e);
        HttpError::InternalError
    })?;

    // NOTE: need to figure out naming
    let file_name = format!("{}.{}", "test", format.extensions_str()[0]);

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

            Ok(web::Json(upload_response))
        }
        Err(e) => {
            log::error!("Error uploading image: {:?}", e);
            Err(HttpError::InternalError)
        }
    }
}
