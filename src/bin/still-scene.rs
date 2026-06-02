use std::path::Path;

use glam::{Mat3, Mat4, Vec3};

use thorus_forge::shapes::sphere;
use thorus_forge::{Camera, FrameBuffer, SCENE_HEIGHT, SCENE_WIDTH, WebpEncoder};
use thorus_forge::{DiffuseLight, Material, draw_facets};

const CAMERA_POS: Vec3 = Vec3::new(0.1, 0.4, -1.0);
const OUT_PATH: &str = "still-scene.webp";

fn model_matrix_still() -> Mat4 {
    let scale = Mat3::from_diagonal(Vec3::splat(0.5));
    Mat4::from_mat3(scale)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut framebuffer = FrameBuffer::new(SCENE_WIDTH, SCENE_HEIGHT);
    let camera = Camera::for_viewport(SCENE_WIDTH, SCENE_HEIGHT).move_to(CAMERA_POS);
    let light = DiffuseLight::new(glam::Vec3::new(1.0, 1.0, -1.0).into(), Material::matte(0.2));

    let mesh = sphere(3).transform(model_matrix_still());
    draw_facets(&mut framebuffer, &camera, &mesh, &light);

    let mut encoder = WebpEncoder::new(SCENE_WIDTH, SCENE_HEIGHT)?;
    encoder.add_frame(&framebuffer)?;
    encoder.write(Path::new(OUT_PATH))?;

    println!(
        "Wrote {} ({SCENE_WIDTH}×{SCENE_HEIGHT}, lossless)",
        OUT_PATH
    );

    Ok(())
}
