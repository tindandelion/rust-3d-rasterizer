//! Minimal export: one lossless WebP frame (800×600) — centered rectangle (400×200 px) with diagonal cross.

mod framebuffer;
mod webp_encoder;

use std::env;
use std::ffi::OsString;
use std::path::Path;

use framebuffer::{FrameBuffer, Point, Rgb};
use webp_encoder::WebpEncoder;

const SCENE_WIDTH: u32 = 800;
const SCENE_HEIGHT: u32 = 600;
const RECT_WIDTH: u32 = 400;
const RECT_HEIGHT: u32 = 200;
const DEFAULT_OUT_PATH: &str = "scene.webp";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = output_file_name();

    let mut framebuffer = FrameBuffer::new(SCENE_WIDTH, SCENE_HEIGHT);
    draw_rectangle(&mut framebuffer, Rgb::WHITE);

    let mut encoder = WebpEncoder::new(SCENE_WIDTH, SCENE_HEIGHT)?;
    encoder.add_frame(&framebuffer)?;
    encoder.write(Path::new(&out_path))?;

    println!(
        "Wrote {} ({SCENE_WIDTH}×{SCENE_HEIGHT}, lossless)",
        out_path.to_string_lossy()
    );

    Ok(())
}

fn draw_rectangle(fb: &mut FrameBuffer, color: Rgb) {
    let cx = SCENE_WIDTH / 2;
    let cy = SCENE_HEIGHT / 2;
    let half_w = RECT_WIDTH / 2;
    let half_h = RECT_HEIGHT / 2;
    let left = cx.saturating_sub(half_w);
    let top = cy.saturating_sub(half_h);

    let tl = Point(left, top);
    let right = left + RECT_WIDTH - 1;
    let bottom = top + RECT_HEIGHT - 1;

    let tr = Point(right, top);
    let br = Point(right, bottom);
    let bl = Point(left, bottom);

    fb.draw_line(tl, tr, color);
    fb.draw_line(tr, br, color);
    fb.draw_line(br, bl, color);
    fb.draw_line(bl, tl, color);

    fb.draw_line(tl, br, color);
    fb.draw_line(tr, bl, color);
}

fn output_file_name() -> OsString {
    let out_path: OsString = env::args_os()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_OUT_PATH.into());
    out_path
}
