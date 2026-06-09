//! **`meshes::sphere(4)`** still: **Phong** **`BlinnLightModel`** (**`Material::shiny(0.15, 100.0)`**), **`0.7`** uniform scale.

use std::path::Path;

use glam::{Mat4, Vec3};

use thorus_forge::BlinnLightModel;
use thorus_forge::Material;
use thorus_forge::Rgb;
use thorus_forge::geometry::{Facet, Mesh, UnitVec3};
use thorus_forge::meshes::sphere;
use thorus_forge::{Camera, FrameBuffer, SCENE_HEIGHT, SCENE_WIDTH, Shape, WebpEncoder};

const CAMERA_POS: Vec3 = Vec3::new(0.0, 0.5, -1.0);
const LIGHT_DIRECTION: Vec3 = Vec3::new(-10.0, 10.0, -10.0);
const OUT_PATH: &str = "still-scene.webp";
const SPHERE_COLOR: Rgb = Rgb(52, 110, 210);
const PLANE_COLOR: Rgb = Rgb(185, 72, 58);

fn model_matrix_still() -> Mat4 {
    Mat4::from_scale(Vec3::new(0.7, 0.7, 0.7))
}

/// **`y = 0`**, **`[-1, 1]²`** in **XZ**. Two triangles, outward **`+Y`** — same winding as the cube top face.
fn horizontal_plane() -> Mesh {
    #[rustfmt::skip]
    let vertices = vec![
        Vec3::new(-1.0, 0.0, -1.0),
        Vec3::new(-1.0, 0.0,  1.0),
        Vec3::new( 1.0, 0.0,  1.0),
        Vec3::new( 1.0, 0.0, -1.0),
    ];
    let facets = vec![
        Facet::with_facet_normal([0, 1, 2], UnitVec3::Y),
        Facet::with_facet_normal([0, 2, 3], UnitVec3::Y),
    ];
    Mesh::new(vertices, facets)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut framebuffer = FrameBuffer::new(SCENE_WIDTH, SCENE_HEIGHT);
    let camera = Camera::for_viewport(SCENE_WIDTH, SCENE_HEIGHT).move_to(CAMERA_POS);
    let light = BlinnLightModel::new(LIGHT_DIRECTION.into(), Material::shiny(0.15, 100.0));

    let sphere = Shape::new(sphere(4).transform(model_matrix_still()), SPHERE_COLOR);
    let plane = Shape::new(horizontal_plane(), PLANE_COLOR);

    sphere.render(&mut framebuffer, &camera, &light);
    plane.render(&mut framebuffer, &camera, &light);

    let mut encoder = WebpEncoder::new(SCENE_WIDTH, SCENE_HEIGHT)?;
    encoder.add_frame(&framebuffer)?;
    encoder.write(Path::new(OUT_PATH))?;

    println!(
        "Wrote {} ({SCENE_WIDTH}×{SCENE_HEIGHT}, lossless)",
        OUT_PATH
    );

    Ok(())
}
