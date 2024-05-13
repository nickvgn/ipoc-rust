use actix_web::{get, Responder};
use image_compressor::compressor::Compressor;
use image_compressor::Factor;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::time::Instant;

use crate::constants;

// NOTE: this was an earlier attempt; only for reference

// avg: ~2.5s ????
#[get("/resize-0")]
pub async fn get_resized_size_comp() -> impl Responder {
    let now = Instant::now();
    let source = PathBuf::from(constants::SOURCE);

    println!(
        "Uncompressed Size: {}",
        source.symlink_metadata().unwrap().size() as f64 * 1e-6
    );

    let dest = PathBuf::from(constants::DEST);
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
