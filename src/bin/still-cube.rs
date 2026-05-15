//! Lossless WebP still: wireframe **cube** with edge length **0.5** in world space.

use std::path::Path;

use thorus_forge::scene::cube::Cube;
use thorus_forge::wireframe::{draw_edges, model_matrix_still};
use thorus_forge::{
    Camera, FrameBuffer, Rgb, SCENE_HEIGHT, SCENE_WIDTH, WebpEncoder, output_webp_path_from_args,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = output_webp_path_from_args();

    let mut framebuffer = FrameBuffer::new(SCENE_WIDTH, SCENE_HEIGHT);
    let camera = Camera::new(SCENE_WIDTH, SCENE_HEIGHT);

    let mut mesh = Cube::new();
    mesh.set_transform(model_matrix_still());
    draw_edges(&mut framebuffer, &camera, &mesh, Rgb::WHITE);

    let mut encoder = WebpEncoder::new(SCENE_WIDTH, SCENE_HEIGHT)?;
    encoder.add_frame(&framebuffer)?;
    encoder.write(Path::new(&out_path))?;

    println!(
        "Wrote {} ({SCENE_WIDTH}×{SCENE_HEIGHT}, lossless)",
        out_path.to_string_lossy()
    );

    Ok(())
}
