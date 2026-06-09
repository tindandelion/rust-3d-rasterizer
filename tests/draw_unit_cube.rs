//! Integration: **`Shape::render`** on the default unit cube vs a hand-built golden framebuffer.
//!
//! The golden fills the projected **−Z** cap with a local **`fill_rect`** helper (duplicated facet normals → uniform
//! intensity, so the shaded square matches a flat fill). With **`Camera::direction` = +Z**, the strictly
//! front-facing hull facet is the **−Z** cap (outward normal **`NEG_Z`**). **`BlinnLightModel`** toward **`NEG_Z`**
//! with a high-ambient matte yields uniform intensity, so
//! the shape **`color`** is unchanged after **`Rgb::scale`**. On this **`FB_WIDTH`×`FB_HEIGHT`** canvas,
//! **`scale = (min(w,h) − 1) / 2`** is an integer, so unit-cube **`±0.5`** corners land exactly on
//! **`FILLED_MIN…FILLED_LAST`** (**no** intermediate **`f32::round`**).

use glam::{Mat4, UVec2, Vec3};
use thorus_forge::{BlinnLightModel, Camera, FrameBuffer, Material, Rgb, Shape, meshes::cube};

const FB_WIDTH: u32 = 101;
const FB_HEIGHT: u32 = 101;
const CAMERA_POS: Vec3 = Vec3::new(0.0, 0.0, -1.0);

#[test]
fn draw_single_unit_cube_produces_rectangle() {
    let mut fb = FrameBuffer::new(FB_WIDTH, FB_HEIGHT);
    let camera = Camera::for_viewport(FB_WIDTH, FB_HEIGHT).move_to(CAMERA_POS);
    let light = BlinnLightModel::new(-camera.direction(), Material::matte(0.9));

    let shape = positioned_cube(0.0, Rgb::BLUE);
    shape.render(&mut fb, &camera, &light);

    let expected = framebuffer_with_rectangle(UVec2::new(25, 25), UVec2::new(75, 75), Rgb::BLUE);
    assert_eq!(fb, expected);
}

#[test]
#[ignore]
fn draw_occluded_cubes_hides_far_cube() {
    let mut fb = FrameBuffer::new(FB_WIDTH, FB_HEIGHT);
    let camera = Camera::for_viewport(FB_WIDTH, FB_HEIGHT).move_to(CAMERA_POS);
    let light = BlinnLightModel::new(-camera.direction(), Material::matte(0.9));

    let near_shape = positioned_cube(0.0, Rgb::BLUE);
    let far_shape = positioned_cube(2.0, Rgb::RED);

    near_shape.render(&mut fb, &camera, &light);
    far_shape.render(&mut fb, &camera, &light);

    let expected = framebuffer_with_rectangle(UVec2::new(25, 25), UVec2::new(75, 75), Rgb::BLUE);
    assert_eq!(fb, expected);
}

fn positioned_cube(z_position: f32, color: Rgb) -> Shape {
    Shape::new(
        cube().transform(Mat4::from_translation(Vec3::new(0.0, 0.0, z_position))),
        color,
    )
}

fn framebuffer_with_rectangle(top_left: UVec2, bottom_right: UVec2, color: Rgb) -> FrameBuffer {
    let mut fb = FrameBuffer::new(FB_WIDTH, FB_HEIGHT);
    for y in top_left.y..=bottom_right.y {
        for x in top_left.x..=bottom_right.x {
            (&mut fb).set_pixel(x, y, color);
        }
    }
    fb
}
