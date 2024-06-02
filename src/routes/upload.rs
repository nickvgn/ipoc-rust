use actix_multipart::Multipart;
use actix_web::{web, Responder};
use cpu_time::ProcessTime;
use futures_util::TryStreamExt;

use crate::processor::ImageProcessor;
use crate::uploader::S3Uploader;
use crate::uploader::Upload;

pub async fn upload_to_s3(mut payload: Multipart, s3: web::Data<S3Uploader>) -> impl Responder {
    let start = ProcessTime::try_now().expect("Getting process time failed");

    while let Ok(Some(mut field)) = payload.try_next().await {
        let mut data = web::BytesMut::new();
        // Streaming data
        while let Ok(Some(chunk)) = field.try_next().await {
            data.extend_from_slice(&chunk);
        }

        let original_file_name = field
            .content_disposition()
            .get_filename()
            .expect("Getting file name failed");

        let buffer = ImageProcessor::new(data, original_file_name)
            .resize()
            .expect("Resizing image failed");

        match s3.upload(buffer, original_file_name).await {
            Ok(_) => log::info!("Object uploaded successfully"),
            Err(e) => log::error!("Error: {:?}", e),
        }
    }

    let elapsed_time = start.try_elapsed().expect("Getting elapsed time failed");

    log::info!(
        "Total time: {:?} for thread: {:?}",
        elapsed_time,
        std::thread::current().id()
    );

    "ok"
}
