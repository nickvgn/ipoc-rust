use std::io::{BufWriter, Cursor};

use actix_web::web::{self};
use anyhow::{anyhow, Context};
use image::codecs::jpeg::JpegEncoder;
use image::io::Reader as ImageReader;
use image::{ExtendedColorType, ImageEncoder};

use fast_image_resize::images::Image;
use fast_image_resize::{IntoImageView, ResizeAlg, ResizeOptions, Resizer};

pub struct ImageProcessor {
    pub image_bytes: web::BytesMut,
    pub file_name: String,
}

impl ImageProcessor {
    pub fn new(image_bytes: web::BytesMut, file_name: &str) -> Self {
        Self {
            image_bytes,
            file_name: file_name.to_owned(),
        }
    }
    pub fn resize(&self) -> anyhow::Result<Vec<u8>> {
        let img = ImageReader::new(Cursor::new(&self.image_bytes))
            .with_guessed_format()?
            .decode()?;

        let dst_width = 1024;
        let dst_height = 768;

        let mut dst_image = match img.pixel_type() {
            Some(pixel_type) => Image::new(dst_width, dst_height, pixel_type),
            None => return Err(anyhow!("Getting pixel type failed")),
        };

        // Create Resizer instance and resize source image
        // into buffer of destination image
        let mut resizer = Resizer::new();

        resizer.resize(
            &img,
            &mut dst_image,
            &ResizeOptions {
                algorithm: ResizeAlg::Nearest,
                ..ResizeOptions::default()
            },
        )?;

        // Write destination image as JPEG-file
        let mut result_buf = BufWriter::new(Vec::new());
        JpegEncoder::new_with_quality(&mut result_buf, 80).write_image(
            dst_image.buffer(),
            dst_width,
            dst_height,
            ExtendedColorType::Rgb8,
        )?;

        Ok(result_buf.into_inner()?)
    }
}
