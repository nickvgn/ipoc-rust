use std::io::{BufWriter, Cursor};

use actix_web::web::{self};
use anyhow::bail;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::io::Reader as ImageReader;
use image::{ColorType, ImageEncoder, ImageFormat};

use fast_image_resize::images::Image;
use fast_image_resize::{IntoImageView, ResizeAlg, ResizeOptions, Resizer};

const RESIZE_WIDTH: u32 = 1024;

pub struct ImageProcessor {
    pub image_bytes: web::BytesMut,
}

// NOTE: should we skip processing if the image is already the right size?
// NOTE: should we have a file type guard?
impl ImageProcessor {
    pub fn new(image_bytes: web::BytesMut) -> Self {
        Self { image_bytes }
    }
    fn get_resized_dimensions(img: &image::DynamicImage) -> (u32, u32) {
        let aspect_ratio = img.width() as f32 / img.height() as f32;
        let dst_width = RESIZE_WIDTH;
        let dst_height = (dst_width as f32 / aspect_ratio) as u32;

        (dst_width, dst_height)
    }
    pub fn resize(&self) -> anyhow::Result<(Vec<u8>, ImageFormat)> {
        let img = ImageReader::new(Cursor::new(&self.image_bytes))
            .with_guessed_format()?
            .decode()?;

        let (dst_width, dst_height) = Self::get_resized_dimensions(&img);

        let mut dst_image = match img.pixel_type() {
            Some(pixel_type) => Image::new(dst_width, dst_height, pixel_type),
            None => bail!("Pixel type not found"),
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

        let format: ImageFormat;

        match img.color() {
            // NOTE: jpeg does not support alpha channel
            ColorType::Rgba8 | ColorType::Rgba32F | ColorType::La8 | ColorType::La16 => {
                format = ImageFormat::Png;
                PngEncoder::new(&mut result_buf).write_image(
                    dst_image.buffer(),
                    dst_width,
                    dst_height,
                    img.color().into(),
                )?;
            }
            _ => {
                format = ImageFormat::Jpeg;
                JpegEncoder::new_with_quality(&mut result_buf, 80).write_image(
                    dst_image.buffer(),
                    dst_width,
                    dst_height,
                    img.color().into(),
                )?;
            }
        }

        Ok((result_buf.into_inner()?, format))
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    fn read_image(file_path: &str) -> web::BytesMut {
        let bytes_vec = std::fs::read(Path::new(file_path)).unwrap();
        let mut image_bytes = web::BytesMut::new();
        image_bytes.extend_from_slice(&bytes_vec);
        image_bytes
    }

    fn check_is_image_resized(bytes_vec: Vec<u8>) {
        let img = ImageReader::new(Cursor::new(bytes_vec))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();

        assert_eq!(img.width(), RESIZE_WIDTH);
    }

    #[test]
    fn resize_jpg() {
        let image_bytes = read_image("./tests/fixtures/tofu-rice.jpg");
        let procesor = ImageProcessor::new(image_bytes);
        let result = procesor.resize();
        check_is_image_resized(result.unwrap().0);
    }

    #[test]
    fn resize_webp() {
        let image_bytes = read_image("./tests/fixtures/steak-dinner.webp");
        let procesor = ImageProcessor::new(image_bytes);
        let result = procesor.resize();
        check_is_image_resized(result.unwrap().0);
    }

    #[test]
    fn resize_png() {
        let image_bytes = read_image("./tests/fixtures/nasa-4928x3279.png");
        let procesor = ImageProcessor::new(image_bytes);
        let result = procesor.resize();
        check_is_image_resized(result.unwrap().0);
    }
}
