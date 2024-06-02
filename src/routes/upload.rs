use actix_multipart::{Field, Multipart};
use actix_web::{post, web, Responder};
use cpu_time::ProcessTime;
use futures_util::TryStreamExt;
use image::codecs::jpeg::JpegEncoder;
use image::io::Reader as ImageReader;
use image::{ExtendedColorType, ImageEncoder};
use std::io::{BufWriter, Cursor};

use s3::error::SdkError;
use s3::operation::put_object::{PutObjectError, PutObjectOutput};
use s3::primitives::ByteStream;

use fast_image_resize::images::Image;
use fast_image_resize::{IntoImageView, ResizeAlg, ResizeOptions, Resizer};

use aws_sdk_s3 as s3;

// avg: ~26ms with Nearest algorithm
#[post("/upload")]
pub async fn upload_to_s3(mut payload: Multipart, client: web::Data<s3::Client>) -> impl Responder {
    // NOTE: i just use this to test connection to aws cli
    // show_buckets(false, &client, "us-east-1")
    //     .await
    //     .expect("Listing buckets failed");

    let start = ProcessTime::try_now().expect("Getting process time failed");

    while let Ok(Some(field)) = payload.try_next().await {
        let (buffer, file_name) = process_image(field).await.expect("Processing image failed");
        let put_object_output =
            upload_object(&client, "image-compress-tournament", buffer, &file_name).await;

        match put_object_output {
            Ok(_) => println!("Object uploaded successfully"),
            Err(e) => println!("Error: {:?}", e),
        }
    }

    let elapsed_time = start.try_elapsed().expect("Getting elapsed time failed");
    println!(
        "Total time: {:?} for thread: {:?}",
        elapsed_time,
        std::thread::current().id()
    );

    "ok"
}

async fn process_image(mut field: Field) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let mut data = web::BytesMut::new();

    // TODO: do we want to keep the original file name?
    let file_name = field
        .content_disposition()
        .get_filename()
        .unwrap()
        .to_owned();

    // let final_path = PathBuf::from(constants::DEST).join(format!(
    //     "{}-{}.jpg",
    //     &file_name.split('.').next().expect("Invalid file name"),
    //     SystemTime::now()
    //         .duration_since(UNIX_EPOCH)
    //         .expect("Time went backwards")
    //         .as_millis()
    // ));

    // Streaming data
    while let Ok(Some(chunk)) = field.try_next().await {
        data.extend_from_slice(&chunk);
    }

    let img = ImageReader::new(Cursor::new(data))
        .with_guessed_format()?
        .decode()?;

    // let start = ProcessTime::try_now().expect("Getting process time failed");

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

    // let elapsed_time = start.try_elapsed().expect("Getting elapsed time failed");
    // println!("Processing time: {:?}", elapsed_time);

    // std::fs::write(final_path, result_buf.get_ref()).expect("Writing file failed");
    let image_data = (
        result_buf
            .into_inner()
            .expect("Getting inner buffer failed"),
        file_name,
    );

    Ok(image_data)
}

async fn upload_object(
    client: &s3::Client,
    bucket_name: &str,
    buffer: Vec<u8>,
    key: &str,
) -> Result<PutObjectOutput, SdkError<PutObjectError>> {
    let body = ByteStream::from(buffer);
    client
        .put_object()
        .bucket(bucket_name)
        .key(key)
        .body(body)
        .send()
        .await
}

async fn _show_buckets(strict: bool, client: &s3::Client, region: &str) -> Result<(), s3::Error> {
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
