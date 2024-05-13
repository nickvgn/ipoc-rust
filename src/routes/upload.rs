use actix_web::{get, Responder};
use cpu_time::ProcessTime;
use std::fs;
use std::io::BufWriter;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use std::num::NonZeroU32;

use turbojpeg::image::{codecs::jpeg::JpegEncoder, io::Reader as ImageReader};
use turbojpeg::image::{ColorType, ImageEncoder};

use crate::{configuration, constants};
use fast_image_resize as fr;

// avg: ~30ms with Nearest algorithm
#[get("/upload")]
pub async fn upload_to_s3() -> impl Responder {
    let mut count = 0u8;
    let mut elapsed_times = Vec::new();
    let img = ImageReader::open(constants::SOURCE)
        .unwrap()
        .decode()
        .expect("Decoding failed");
    let final_path = PathBuf::from(constants::DEST).join(format!(
        "orange-boi-turbo{}.jpg",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
            .to_string()
    ));

    let repeat_count = if let Ok(config) = configuration::get_config() {
        config.repeat_count
    } else {
        0
    };

    loop {
        count += 1;
        let start = ProcessTime::try_now().expect("Getting process time failed");

        let width = NonZeroU32::new(img.width()).expect("Invalid width");
        let height = NonZeroU32::new(img.height()).expect("Invalid height");

        let src_image = fr::Image::from_vec_u8(
            width,
            height,
            img.to_rgba8().into_raw(),
            fr::PixelType::U8x4,
        )
        .expect("Creating image from buffer failed");

        // Create container for data of destination image
        let dst_width = NonZeroU32::new(1024).expect("Invalid width");
        let dst_height = NonZeroU32::new(768).expect("Invalid height");
        let mut dst_image = fr::Image::new(dst_width, dst_height, src_image.pixel_type());

        // Get mutable view of destination image data
        let mut dst_view = dst_image.view_mut();

        // Create Resizer instance and resize source image
        // into buffer of destination image
        let mut resizer = fr::Resizer::new(fr::ResizeAlg::Nearest);

        resizer
            .resize(&src_image.view(), &mut dst_view)
            .expect("Resizing failed");

        // Write destination image as JPEG-file
        let mut result_buf = BufWriter::new(Vec::new());
        JpegEncoder::new_with_quality(&mut result_buf, 80)
            .write_image(
                dst_image.buffer(),
                dst_width.get(),
                dst_height.get(),
                ColorType::Rgba8,
            )
            .expect("Writing image failed");

        if count == 1 {
            match configuration::get_config() {
                Ok(config) if config.write_image != 0 => {
                    println!("Saving file...");

                    std::fs::write(
                        &final_path,
                        result_buf
                            .into_inner()
                            .expect("Getting inner buffer failed"),
                    )
                    .expect("Writing file failed");

                    println!(
                        "Image File Size: {}KB\n",
                        fs::read(&final_path).unwrap().len() as f64 * 0.001
                    );
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                }
                _ => ()
            }
        }

        let elapsed_time = start.try_elapsed().expect("Getting elapsed time failed");

        println!("Elapsed time: {:?}", elapsed_time);

        elapsed_times.push(elapsed_time.as_millis());
        if count >= repeat_count {
            break;
        }
    }

    let avg_time = elapsed_times.iter().sum::<u128>() / elapsed_times.len() as u128;

    println!("\nAverage Image Processing Time: {:?}ms\n", avg_time);

    elapsed_times
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(", ")
}
