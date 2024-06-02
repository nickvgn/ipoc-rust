use actix_web::web::{self};
use image::codecs::jpeg::JpegEncoder;
use image::io::Reader as ImageReader;
use image::{ExtendedColorType, ImageEncoder};
use std::io::{BufWriter, Cursor};

use fast_image_resize::images::Image;
use fast_image_resize::{IntoImageView, ResizeAlg, ResizeOptions, Resizer};

pub struct ImageProcessor {
    pub image_buffer: web::BytesMut,
    pub file_name: String,
}

impl ImageProcessor {
    pub fn new(image_buffer: web::BytesMut, file_name: &str) -> Self {
        Self {
            image_buffer,
            file_name: file_name.to_owned(),
        }
    }
    pub fn resize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let img = ImageReader::new(Cursor::new(&self.image_buffer))
            .with_guessed_format()?
            .decode()?;

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

        Ok(result_buf
            .into_inner()
            .expect("Getting inner buffer failed"))
    }
}
