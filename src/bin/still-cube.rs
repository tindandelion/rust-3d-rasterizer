//! Lossless WebP still: **filled** **cube** — **faceted** **`DiffuseLight`** (**per-face normals** duplicated at corners), **`SHAPE_BASE_COLOR`**, back-face culled; edge length **0.5** in world space.

use std::path::Path;

use glam::{Mat3, Mat4, Vec3};

use thorus_forge::shapes::cube;
use thorus_forge::{
    Camera, FrameBuffer, SCENE_HEIGHT, SCENE_WIDTH, WebpEncoder, output_webp_path_from_args,
};
use thorus_forge::{DiffuseLight, draw_facets};

const TILT: f32 = std::f32::consts::FRAC_PI_4;
const CAMERA_POS: Vec3 = Vec3::new(0.1, 0.4, -1.0);

fn model_matrix_still() -> Mat4 {
    let base = Mat3::from_rotation_y(TILT);
    let scale = Mat3::from_diagonal(Vec3::splat(0.5));
    Mat4::from_mat3(base * scale)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = output_webp_path_from_args();

    let mut framebuffer = FrameBuffer::new(SCENE_WIDTH, SCENE_HEIGHT);
    let camera = Camera::for_viewport(SCENE_WIDTH, SCENE_HEIGHT).move_to(CAMERA_POS);
    let light = DiffuseLight::new(glam::Vec3::new(1.0, 1.0, -1.0).into(), 0.25);

    let mesh = cube().transform(model_matrix_still());
    draw_facets(&mut framebuffer, &camera, &mesh, &light);

    let mut encoder = WebpEncoder::new(SCENE_WIDTH, SCENE_HEIGHT)?;
    encoder.add_frame(&framebuffer)?;
    encoder.write(Path::new(&out_path))?;

    println!(
        "Wrote {} ({SCENE_WIDTH}×{SCENE_HEIGHT}, lossless)",
        out_path.to_string_lossy()
    );

    Ok(())
}
