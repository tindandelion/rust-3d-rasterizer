//! Minimal export: one opaque-black frame as lossless WebP (project target size 800×600).

mod framebuffer;
mod webp_encoder;

use std::env;
use std::ffi::OsString;
use std::path::Path;

use framebuffer::FrameBuffer;
use webp_encoder::WebpEncoder;

const SCENE_WIDTH: u32 = 800;
const SCENE_HEIGHT: u32 = 600;
const DEFAULT_OUT_PATH: &str = "scene.webp";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = output_file_name();

    let framebuffer = FrameBuffer::new(SCENE_WIDTH, SCENE_HEIGHT);
    let mut encoder = WebpEncoder::new(SCENE_WIDTH, SCENE_HEIGHT)?;

    encoder.add_frame(&framebuffer)?;
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
