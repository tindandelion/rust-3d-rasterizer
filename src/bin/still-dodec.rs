//! Lossless WebP still: **`Dodecahedron`** (**`[-0.5, 0.5]³`** like **`Cube::default`**), **`CUBE_ALBEDO`**, **`DiffuseLight`**; pose matches **`still-cube`** (**π/4 X/Y**, **½** world scale).

use std::path::Path;

use glam::{Mat3, Mat4, Vec3};

use thorus_forge::scene::dodecahedron::Dodecahedron;
use thorus_forge::{
    Camera, FrameBuffer, SCENE_HEIGHT, SCENE_WIDTH, WebpEncoder, output_webp_path_from_args,
};
use thorus_forge::{DiffuseLight, draw_faces};

const TILT: f32 = std::f32::consts::FRAC_PI_4;

/// **π/4** tilt on **X** then **Y**, **0.5** uniform scale (readable ortho snapshot, same **`still-cube`** framing).
fn model_matrix_still() -> Mat4 {
    let base = Mat3::from_rotation_x(TILT) * Mat3::from_rotation_y(TILT);
    let scale = Mat3::from_diagonal(Vec3::splat(0.5));
    Mat4::from_mat3(base * scale)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = output_webp_path_from_args();

    let mut framebuffer = FrameBuffer::new(SCENE_WIDTH, SCENE_HEIGHT);
    let camera = Camera::new(SCENE_WIDTH, SCENE_HEIGHT);
    let light = DiffuseLight::new(glam::Vec3::new(1.0, 1.0, -1.0).into(), 0.25);

    let mesh = Dodecahedron::default().transform(model_matrix_still());
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
