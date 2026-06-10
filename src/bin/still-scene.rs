//! **`meshes::sphere(4)`** still: **Phong** **`BlinnLightModel`** (**`Material::shiny(0.15, 100.0)`**), **`0.7`** uniform scale.

use std::path::Path;

use glam::{Mat4, Vec3};

use thorus_forge::BlinnLightModel;
use thorus_forge::Material;
use thorus_forge::Rgb;
use thorus_forge::meshes::sphere;
use thorus_forge::{Camera, FrameBuffer, SCENE_HEIGHT, SCENE_WIDTH, Shape, WebpEncoder};

const CAMERA_POS: Vec3 = Vec3::new(0.0, 0.5, -1.0);
const LIGHT_DIRECTION: Vec3 = Vec3::new(-10.0, 10.0, -10.0);
const OUT_PATH: &str = "still-scene.webp";
const SPHERE_COLOR: Rgb = Rgb(52, 110, 210);
const PLANE_COLOR: Rgb = Rgb(185, 72, 58);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut framebuffer = FrameBuffer::new(SCENE_WIDTH, SCENE_HEIGHT);
    let camera = Camera::for_viewport(SCENE_WIDTH, SCENE_HEIGHT).move_to(CAMERA_POS);
    let light = BlinnLightModel::new(LIGHT_DIRECTION.into(), Material::shiny(0.15, 100.0));

    let dome = Shape::new(
        sphere(4).transform(Mat4::from_scale(Vec3::new(0.5, 0.5, 0.5))),
        SPHERE_COLOR,
    );
    let body = Shape::new(
        sphere(4).transform(Mat4::from_scale(Vec3::new(0.9, 0.2, 0.9))),
        PLANE_COLOR,
    );

    dome.render(&mut framebuffer, &camera, &light);
    body.render(&mut framebuffer, &camera, &light);

    let mut encoder = WebpEncoder::new(SCENE_WIDTH, SCENE_HEIGHT)?;
    encoder.add_frame(&framebuffer)?;
    encoder.write(Path::new(OUT_PATH))?;

    println!(
        "Wrote {} ({SCENE_WIDTH}×{SCENE_HEIGHT}, lossless)",
        OUT_PATH
    );

    Ok(())
}
