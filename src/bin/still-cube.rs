//! Lossless WebP still: wireframe **cube** with edge length **0.5** in world space.

use std::env;
use std::ffi::OsString;
use std::path::Path;

use thorus_forge::scene::cube;
use thorus_forge::{Camera, FrameBuffer, Rgb, WebpEncoder};

const SCENE_WIDTH: u32 = 800;
const SCENE_HEIGHT: u32 = 600;

const DEFAULT_OUT_PATH: &str = "scene.webp";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = output_file_name();

    let mut framebuffer = FrameBuffer::new(SCENE_WIDTH, SCENE_HEIGHT);
    let camera = Camera::new(SCENE_WIDTH, SCENE_HEIGHT);
    cube::draw_wireframe(&mut framebuffer, &camera, Rgb::WHITE);

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
    env::args_os()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_OUT_PATH.into())
}
