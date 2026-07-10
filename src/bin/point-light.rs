//! Ground plane lit by two white **point lights** — lossless still WebP export.

use std::path::Path;

use glam::{Mat4, Vec3};

use thorus_forge::geometry::{Facet, Mesh, UnitVec3};
use thorus_forge::meshes::sphere;
use thorus_forge::{
    Camera, FrameBuffer, Light, Material, Rgb, SCENE_BACKGROUND, SCENE_HEIGHT, SCENE_WIDTH, Shape,
    WebpEncoder,
};

const OUT_PATH: &str = "point-light.webp";
const CAMERA_POS: Vec3 = Vec3::new(0.0, 0.25, -1.0);

/// Horizontal **`y = −0.9`** rectangle spanning **`x, z ∈ [−0.9, 0.9]`**.
fn ground_plane() -> Mesh {
    let vertices = vec![
        Vec3::new(-0.9, -0.7, -0.9),
        Vec3::new(0.9, -0.7, -0.9),
        Vec3::new(0.6, -0.7, 0.9),
        Vec3::new(-0.6, -0.7, 0.9),
    ];
    let normal = UnitVec3::Y;
    let facets = vec![
        Facet::with_facet_normal([0, 1, 2], normal),
        Facet::with_facet_normal([0, 2, 3], normal),
    ];
    Mesh::new(vertices, facets)
}

fn light_marker_material() -> Material {
    let yellow = Rgb::from_hex(0xFFD700);
    Material::new(yellow, Rgb::BLACK, Rgb::BLACK, None)
}

fn light_marker(position: Vec3) -> Shape {
    let mesh =
        sphere(2).transform(Mat4::from_translation(position) * Mat4::from_scale(Vec3::splat(0.04)));
    Shape::new(mesh, light_marker_material())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut framebuffer = FrameBuffer::new(SCENE_WIDTH, SCENE_HEIGHT);
    framebuffer.clear(SCENE_BACKGROUND);

    let camera = Camera::for_viewport(SCENE_WIDTH, SCENE_HEIGHT).move_to(CAMERA_POS);
    let light_positions = [Vec3::new(0.0, 0.1, 0.0)];
    let lights: Vec<Light> = light_positions
        .iter()
        .map(|&position| Light::point(position, 1.0))
        .collect();

    let material = Material::new(Rgb::BLACK, Rgb::from_hex(0x156289), Rgb::BLACK, None);

    Shape::new(ground_plane(), material).render(&mut framebuffer, &camera, &lights);
    for &position in &light_positions {
        light_marker(position).render(&mut framebuffer, &camera, &lights);
    }

    let mut encoder = WebpEncoder::new(SCENE_WIDTH, SCENE_HEIGHT)?;
    encoder.add_frame(&framebuffer)?;
    encoder.write(Path::new(OUT_PATH))?;
    println!(
        "Wrote {OUT_PATH} ({}×{}, lossless)",
        SCENE_WIDTH, SCENE_HEIGHT,
    );

    Ok(())
}
