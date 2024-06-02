use actix_multipart::{Field, Multipart};
use actix_web::{post, web, Responder};
use cpu_time::ProcessTime;
use futures_util::TryStreamExt;
use std::io::{BufWriter, Cursor};

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use image::codecs::jpeg::JpegEncoder;
use image::io::Reader as ImageReader;
use image::{ExtendedColorType, ImageEncoder};

use fast_image_resize::images::Image;
use fast_image_resize::{CpuExtensions, IntoImageView, ResizeAlg, ResizeOptions, Resizer};

use crate::constants;

// avg: ~26ms with Nearest algorithm
#[post("/upload")]
pub async fn upload_to_s3(mut payload: Multipart) -> impl Responder {
    let start = ProcessTime::try_now().expect("Getting process time failed");

    while let Ok(Some(field)) = payload.try_next().await {
        println!("Processing field: {:?}", field);
        process_image(field).await.expect("Processing image failed");
    }

    let elapsed_time = start.try_elapsed().expect("Getting elapsed time failed");
    println!("Elapsed time: {:?}", elapsed_time);

    "ok"
}

async fn process_image(mut field: Field) -> Result<&'static str, Box<dyn std::error::Error>> {
    let mut data = web::BytesMut::new();

    let file_name = field.content_disposition().get_filename().unwrap();
    println!("Uploading file: {}", file_name);

    let final_path = PathBuf::from(constants::DEST).join(format!(
        "{}-{}.jpg",
        &file_name.split('.').next().expect("Invalid file name"),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
    ));

    // Streaming data
    while let Ok(Some(chunk)) = field.try_next().await {
        data.extend_from_slice(&chunk);
    }

    let start_reader = ProcessTime::try_now().expect("Getting process time failed");
    let img = ImageReader::new(Cursor::new(data))
        .with_guessed_format()?
        .decode()?;

    let elapsed_time_reader = start_reader
        .try_elapsed()
        .expect("Getting elapsed time failed");
    println!("Reading Time time: {:?}", elapsed_time_reader);

    let start = ProcessTime::try_now().expect("Getting process time failed");

    let dst_width = 1024;
    let dst_height = 768;
    // Create container for data of destination image
    let mut dst_image = Image::new(
        dst_width,
        dst_height,
        img.pixel_type().expect("Getting pixel type failed"),
    );

    // Create Resizer instance and resize source image
    // into buffer of destination image
    let mut resizer = Resizer::new();

    resizer
        .resize(
            &img,
            &mut dst_image,
            &ResizeOptions {
                algorithm: ResizeAlg::Nearest,
                ..ResizeOptions::default()
            },
        )
        .expect("Resizing failed");

    // Write destination image as JPEG-file
    let mut result_buf = BufWriter::new(Vec::new());
    JpegEncoder::new_with_quality(&mut result_buf, 80)
        .write_image(
            dst_image.buffer(),
            dst_width,
            dst_height,
            ExtendedColorType::Rgb8,
        )
        .expect("Writing image failed");

    let elapsed_time = start.try_elapsed().expect("Getting elapsed time failed");
    println!("Processing Time time: {:?}", elapsed_time);

    std::fs::write(final_path, result_buf.get_ref()).expect("Writing file failed");

    Ok("ok")
}

//let mut elapsed_times = Vec::new();

//let start = ProcessTime::try_now().expect("Getting process time failed");

//let elapsed_time = start.try_elapsed().expect("Getting elapsed time failed");
//
//println!("Elapsed time: {:?}", elapsed_time);
//
//elapsed_times.push(elapsed_time.as_millis());
//
//let avg_time = elapsed_times.iter().sum::<u128>() / elapsed_times.len() as u128;
//
//println!("\nAverage Image Processing Time: {:?}ms\n", avg_time);

//elapsed_times
//    .iter()
//    .map(|x| x.to_string())
//    .collect::<Vec<String>>()
//    .join(", ")

//let final_path = PathBuf::from(constants::DEST).join(format!(
//    "orange-boi-turbo{}.jpg",
//    SystemTime::now()
//        .duration_since(UNIX_EPOCH)
//        .expect("Time went backwards")
//        .as_millis()
//));

//let repeat_count = if let Ok(config) = configuration::get_config() {
//    config.repeat_count
//} else {
//    0
//};

//if count == 1 {
//    match configuration::get_config() {
//        Ok(config) if config.write_image != 0 => {
//            println!("Saving file...");
//
//            std::fs::write(
//                &final_path,
//                result_buf
//                    .into_inner()
//                    .expect("Getting inner buffer failed"),
//            )
//            .expect("Writing file failed");
//
//            println!(
//                "Image File Size: {}KB\n",
//                fs::read(&final_path).unwrap().len() as f64 * 0.001
//            );
//        }
//        Err(e) => {
//            println!("Error: {:?}", e);
//        }
//        _ => (),
//    }
//}
