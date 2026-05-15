//! Lossless **animated** WebP: orthographic **wireframe** cube (edge length **0.5** in world space),
//! **\(R_y\)** rotation sampled over **`ANIMATED_CUBE_FRAME_COUNT`** frames.

use std::path::Path;

use glam::Mat4;

use thorus_forge::scene::cube::Cube;
use thorus_forge::wireframe::{draw_edges, model_matrix_still};
use thorus_forge::{
    ANIMATED_CUBE_FRAME_COUNT, Camera, FrameBuffer, Rgb, SCENE_HEIGHT, SCENE_WIDTH, WebpEncoder,
    output_webp_path_from_args,
};

/// Wireframe stroke: **cornflower blue** (reads clearly on black).
const WIRE_RGB: Rgb = Rgb(100, 149, 237);

/// Timestamp step between successive frames (**ms**); last frame duration uses the same spacing at **`finalize`**.
///
/// **`20 ms`** ⇒ **50 fps** (`1000 / 20`). With **`360`** frames, one full **`R_y`** lap is **\(360^\circ\)** at **\(1^\circ\)** per sample.
const FRAME_SPACING_MS: i32 = 20;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = output_webp_path_from_args();

    let camera = Camera::new(SCENE_WIDTH, SCENE_HEIGHT);
    let mut framebuffer = FrameBuffer::new(SCENE_WIDTH, SCENE_HEIGHT);

    let mut encoder = WebpEncoder::with_frame_spacing(SCENE_WIDTH, SCENE_HEIGHT, FRAME_SPACING_MS)?;

    for frame_index in 0..ANIMATED_CUBE_FRAME_COUNT {
        framebuffer.clear_black();

        let mut mesh = Cube::new();
        mesh.set_transform(model_matrix_y_lap(frame_index, ANIMATED_CUBE_FRAME_COUNT));
        draw_edges(&mut framebuffer, &camera, &mesh, WIRE_RGB);

        encoder.add_frame(&framebuffer)?;
    }

    encoder.write(Path::new(&out_path))?;

    println!(
        "Wrote {} ({ANIMATED_CUBE_FRAME_COUNT} frames, {}×{}, lossless)",
        out_path.to_string_lossy(),
        SCENE_WIDTH,
        SCENE_HEIGHT,
    );

    Ok(())
}

/// Full **`R_y`** lap sampled over **`lap_frames`** frames; left-multiplies **`wireframe::model_matrix_still`**.
fn model_matrix_y_lap(frame_index: u32, lap_frames: u32) -> Mat4 {
    let n = lap_frames.max(1) as f32;
    let t = (frame_index as f32 / n) * std::f32::consts::TAU;
    Mat4::from_rotation_y(t) * model_matrix_still()
}
