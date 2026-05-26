//! Lossless WebP still: **filled** **octasphere** (`unit_sphere` seed) — uniform **`CUBE_ALBEDO`** blue, **`DiffuseLight`**, back-face culled; **0.5** uniform scale in world space.

use std::path::Path;

use glam::{Mat3, Mat4, Vec3};

use thorus_forge::scene::sphere::unit_sphere;
use thorus_forge::{
    Camera, FrameBuffer, SCENE_HEIGHT, SCENE_WIDTH, WebpEncoder, output_webp_path_from_args,
};
use thorus_forge::{DiffuseLight, draw_faces};

const CAMERA_POS: Vec3 = Vec3::new(0.1, 0.4, -1.0);

fn model_matrix_still() -> Mat4 {
    let scale = Mat3::from_diagonal(Vec3::splat(0.5));
    Mat4::from_mat3(scale)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = output_webp_path_from_args();

    let mut framebuffer = FrameBuffer::new(SCENE_WIDTH, SCENE_HEIGHT);
    let camera = Camera::for_viewport(SCENE_WIDTH, SCENE_HEIGHT).move_to(CAMERA_POS);
    let light = DiffuseLight::new(glam::Vec3::new(1.0, 1.0, -1.0).into(), 0.1);

    let mesh = unit_sphere(0).transform(model_matrix_still());
    draw_faces(&mut framebuffer, &camera, &mesh, &light);

    let mut encoder = WebpEncoder::new(SCENE_WIDTH, SCENE_HEIGHT)?;
    encoder.add_frame(&framebuffer)?;
    encoder.write(Path::new(&out_path))?;

    println!(
        "Wrote {} ({SCENE_WIDTH}×{SCENE_HEIGHT}, lossless)",
        out_path.to_string_lossy()
    );

    Ok(())
}
