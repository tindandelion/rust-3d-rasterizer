//! Minimal export: one lossless WebP frame (800×600) — radial “flower” from the center.

mod framebuffer;
mod ortho_projection;
mod webp_encoder;

use std::env;
use std::f64::consts::TAU;
use std::ffi::OsString;
use std::path::Path;

use framebuffer::{FrameBuffer, Point, Rgb};
use webp_encoder::WebpEncoder;

const SCENE_WIDTH: u32 = 800;
const SCENE_HEIGHT: u32 = 600;
/// Spokes around the circle; higher counts give a smoother outline.
const FLOWER_RAY_COUNT: u32 = 48;
const DEFAULT_OUT_PATH: &str = "scene.webp";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = output_file_name();

    let mut framebuffer = FrameBuffer::new(SCENE_WIDTH, SCENE_HEIGHT);
    draw_flower(&mut framebuffer, Rgb::WHITE);

    let mut encoder = WebpEncoder::new(SCENE_WIDTH, SCENE_HEIGHT)?;
    encoder.add_frame(&framebuffer)?;
    encoder.write(Path::new(&out_path))?;

    println!(
        "Wrote {} ({SCENE_WIDTH}×{SCENE_HEIGHT}, lossless)",
        out_path.to_string_lossy()
    );

    Ok(())
}

fn draw_flower(fb: &mut FrameBuffer, color: Rgb) {
    let cx = SCENE_WIDTH / 2;
    let cy = SCENE_HEIGHT / 2;
    let center = Point(cx, cy);

    let max_r = cx
        .min(SCENE_WIDTH - 1 - cx)
        .min(cy.min(SCENE_HEIGHT - 1 - cy));
    let radius = max_r.saturating_sub(8) as f64;

    let cx_f = cx as f64;
    let cy_f = cy as f64;
    let n = FLOWER_RAY_COUNT as f64;

    for i in 0..FLOWER_RAY_COUNT {
        let theta = TAU * (i as f64) / n;
        let end_x = (cx_f + radius * theta.cos()).round() as u32;
        let end_y = (cy_f + radius * theta.sin()).round() as u32;
        fb.draw_line(center, Point(end_x, end_y), color);
    }
}

fn output_file_name() -> OsString {
    let out_path: OsString = env::args_os()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_OUT_PATH.into());
    out_path
}
