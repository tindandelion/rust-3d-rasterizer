//! Integration: orthographic fill of the default unit cube against a hand-built golden framebuffer.
//!
//! With **`Camera::direction` = +Z**, the strictly front-facing hull facet is the **−Z** cap (outward normal
//! **`NEG_Z`**). **`DiffuseLight`** toward **`NEG_Z`** yields full **`calc_intensity`**, so
//! [`thorus_forge::SHAPE_BASE_COLOR`] is unchanged after **`Rgb::scale`**. On this **`FB_WIDTH`×`FB_HEIGHT`** canvas,
//! **`scale = (min(w,h) − 1) / 2`** is an integer, so unit-cube **`±0.5`** corners land exactly on
//! **`FILLED_MIN…FILLED_LAST`** (**no** intermediate **`f32::round`**).

use glam::{UVec2, Vec3};
use thorus_forge::{
    Camera, DiffuseLight, FillTriangle, FrameBuffer, SHAPE_BASE_COLOR, draw_facets, shapes::cube,
};

const FB_WIDTH: u32 = 101;
const FB_HEIGHT: u32 = 101;
const CAMERA_POS: Vec3 = Vec3::new(0.0, 0.0, -1.0);

#[test]
fn draw_unit_cube() {
    let mut fb = FrameBuffer::new(FB_WIDTH, FB_HEIGHT);
    let camera = Camera::for_viewport(FB_WIDTH, FB_HEIGHT).move_to(CAMERA_POS);
    let light = DiffuseLight::new(-camera.direction(), 0.1);
    let mesh = cube();

    draw_facets(&mut fb, &camera, &mesh, &light);

    let expected = expected_framebuffer_unit_cube_camera_front();
    assert_eq!(fb, expected);
}

fn expected_framebuffer_unit_cube_camera_front() -> FrameBuffer {
    let mut fb = FrameBuffer::new(FB_WIDTH, FB_HEIGHT);
    let corners = [
        UVec2::new(25, 25),
        UVec2::new(25, 75),
        UVec2::new(75, 75),
        UVec2::new(75, 25),
    ];
    FillTriangle::new([corners[0], corners[1], corners[2]], SHAPE_BASE_COLOR).draw(&mut fb);
    FillTriangle::new([corners[0], corners[2], corners[3]], SHAPE_BASE_COLOR).draw(&mut fb);
    fb
}
