//! Minimal export: one opaque-black frame as lossless WebP (project target size 800×600).

mod webp_encoder;

use std::env;
use std::ffi::OsString;
use std::path::Path;

use webp_encoder::WebpEncoder;

const SCENE_WIDTH: u32 = 800;
const SCENE_HEIGHT: u32 = 600;
const DEFAULT_OUT_PATH: &str = "scene.webp";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = output_file_name();

    let pixel_count = SCENE_WIDTH as usize * SCENE_HEIGHT as usize;
    let mut rgba = vec![0u8; pixel_count * 4];
    for px in rgba.chunks_exact_mut(4) {
        px[3] = 255;
    }

    let mut encoder = WebpEncoder::new(SCENE_WIDTH, SCENE_HEIGHT)?;
    encoder.add_frame(&rgba)?;
    encoder.write(Path::new(&out_path))?;

    println!(
        "Wrote {} ({SCENE_WIDTH}×{SCENE_HEIGHT}, lossless)",
        out_path.to_string_lossy()
    );
    Ok(())
}

fn output_file_name() -> OsString {
    let out_path: OsString = env::args_os()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_OUT_PATH.into());
    out_path
}
