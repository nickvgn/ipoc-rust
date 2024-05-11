use actix_web::{get, App, HttpServer, Responder};
use cpu_time::ProcessTime;
use image_compressor::compressor::Compressor;
use image_compressor::Factor;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use std::io::BufWriter;
use std::num::NonZeroU32;

use turbojpeg::image::codecs::jpeg::JpegEncoder;
use turbojpeg::image::io::Reader as ImageReader;
use turbojpeg::image::{ColorType, ImageEncoder, RgbImage};

use fast_image_resize as fr;

static SOURCE: &str = "/Users/nickvegean/Dev/image-processor-optimization-challenge/orange-boi.jpg";
static DEST: &str = "/Users/nickvegean/Dev/image-processor-optimization-challenge/compressed/";

// avg: ~2.5s ????
#[get("/resize-0")]
async fn get_resized_size_comp() -> impl Responder {
    let now = Instant::now();
    let source = PathBuf::from(SOURCE);

    println!(
        "Uncompressed Size: {}",
        source.symlink_metadata().unwrap().size() as f64 * 1e-6
    );

    let dest = PathBuf::from(DEST);
    let mut comp = Compressor::new(source, dest);
    comp.set_factor(Factor::new(80., 0.8));

    match comp.compress_to_jpg() {
        Ok(v) => {
            println!(
                "Compressed size: {}",
                v.symlink_metadata().unwrap().size() as f64 * 1e-6
            );
            println!("Success");
            let elapsed_time = now.elapsed();
            std::fs::remove_file(v).unwrap();

            format!("Elapsed time: {:?}", elapsed_time)
        }
        Err(e) => format!("Error {:?}", e),
    }
}

// avg: ~255s
#[get("/resize-1")]
async fn get_resized_size_turbo() -> impl Responder {
    let jpeg = std::fs::read(SOURCE);
    let mut count = 0u8;
    let mut elapsed_times = Vec::new();
    loop {
        count += 1;
        let elapsed_time = match &jpeg {
            Ok(jpeg) => {
                println!("Uncompressed Size: {}", jpeg.len() as f64 * 1e-6);

                let start = ProcessTime::try_now().expect("Getting process time failed");
                let image: RgbImage =
                    turbojpeg::decompress_image(&jpeg).expect("Decompression failed");
                let jpeg = turbojpeg::compress_image(&image, 80, turbojpeg::Subsamp::Sub2x2);
                let final_path = PathBuf::from(DEST).join(format!(
                    "orange-boi-turbo{}.jpg",
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_millis()
                        .to_string()
                ));

                let temp_path = format!(std::env::temp_dir().to_str().unwrap());

                match jpeg {
                    Ok(jpeg) => {
                        println!("Compressed size: {}", jpeg.len() as f64 * 1e-6);

                        if count == 1 {
                            // NOTE: run it without this file operation to get a more accurate time
                            std::fs::write(temp_dir, &jpeg).expect("Writing file failed");
                        }
                    }
                    Err(e) => panic!("Error {:?}", e),
                }

                let img = ImageReader::open(&final_path).unwrap().decode().unwrap();
                let width = NonZeroU32::new(img.width()).unwrap();
                let height = NonZeroU32::new(img.height()).unwrap();
                let mut src_image = fr::Image::from_vec_u8(
                    width,
                    height,
                    img.to_rgba8().into_raw(),
                    fr::PixelType::U8x4,
                )
                .unwrap();

                // Multiple RGB channels of source image by alpha channel
                // (not required for the Nearest algorithm)
                let alpha_mul_div = fr::MulDiv::default();
                alpha_mul_div
                    .multiply_alpha_inplace(&mut src_image.view_mut())
                    .unwrap();

                // Create container for data of destination image
                let dst_width = NonZeroU32::new(1024).expect("Invalid width");
                let dst_height = NonZeroU32::new(768).expect("Invalid height");
                let mut dst_image = fr::Image::new(dst_width, dst_height, src_image.pixel_type());

                // Get mutable view of destination image data
                let mut dst_view = dst_image.view_mut();

                // Create Resizer instance and resize source image
                // into buffer of destination image
                let mut resizer =
                    fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3));
                resizer
                    .resize(&src_image.view(), &mut dst_view)
                    .expect("Resizing failed");

                // Divide RGB channels of destination image by alpha
                alpha_mul_div
                    .divide_alpha_inplace(&mut dst_view)
                    .expect("Dividing failed");

                // Write destination image as PNG-file
                let mut result_buf = BufWriter::new(Vec::new());
                JpegEncoder::new(&mut result_buf)
                    .write_image(
                        dst_image.buffer(),
                        dst_width.get(),
                        dst_height.get(),
                        ColorType::Rgba8,
                    )
                    .expect("Writing image failed");

                let elapsed_time = start.try_elapsed().expect("Getting elapsed time failed");

                println!("Elapsed time: {:?}\n", elapsed_time);

                elapsed_time
            }
            Err(e) => panic!("Error {:?}", e),
        };
        elapsed_times.push(elapsed_time.as_millis());
        if count == 50 {
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(get_resized_size_comp)
            .service(get_resized_size_turbo)
    })
    .bind("127.0.0.1:1234")?
    .run()
    .await
}
