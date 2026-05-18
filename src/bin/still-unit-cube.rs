//! Lossless WebP still: **filled faceted** cube at **identity** pose — same pipeline as **`still-cube`**
//! (orthographic camera, **`draw_faces`**, **`CUBE_FACE_PALETTE`**), but **no** model matrix
//! (**edge length 1** in world space, see **`Cube::default`**).

use std::path::Path;

use thorus_forge::draw_faces;
use thorus_forge::scene::cube::Cube;
use thorus_forge::{
    Camera, FrameBuffer, SCENE_HEIGHT, SCENE_WIDTH, WebpEncoder, output_webp_path_from_args,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = output_webp_path_from_args();

    let mut framebuffer = FrameBuffer::new(SCENE_WIDTH, SCENE_HEIGHT);
    let camera = Camera::new(SCENE_WIDTH, SCENE_HEIGHT);

    let mesh = Cube::default();
    draw_faces(&mut framebuffer, &camera, &mesh);

    let mut encoder = WebpEncoder::new(SCENE_WIDTH, SCENE_HEIGHT)?;
    encoder.add_frame(&framebuffer)?;
    encoder.write(Path::new(&out_path))?;

    println!(
        "Wrote {} ({SCENE_WIDTH}×{SCENE_HEIGHT}, lossless)",
        out_path.to_string_lossy()
    );

    Ok(())
}
