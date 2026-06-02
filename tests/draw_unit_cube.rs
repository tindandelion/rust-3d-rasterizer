//! Integration: **`draw_facets`** on the default unit cube vs a hand-built golden framebuffer (**[`ShadedFillTriangle`]**).
//!
//! With **`Camera::direction` = +Z**, the strictly front-facing hull facet is the **−Z** cap (outward normal
//! **`NEG_Z`**). **`PhongLightModel`** toward **`NEG_Z`** with a high-ambient matte yields uniform corner intensity, so
//! [`thorus_forge::SHAPE_BASE_COLOR`] is unchanged after **`Rgb::scale`**. On this **`FB_WIDTH`×`FB_HEIGHT`** canvas,
//! **`scale = (min(w,h) − 1) / 2`** is an integer, so unit-cube **`±0.5`** corners land exactly on
//! **`FILLED_MIN…FILLED_LAST`** (**no** intermediate **`f32::round`**).

use glam::{UVec2, Vec3};
use thorus_forge::{
    Camera, FrameBuffer, Material, PhongLightModel, SHAPE_BASE_COLOR, draw_facets,
    framebuffer::{ShadedCorner, ShadedFillTriangle},
    shapes::cube,
};

const FB_WIDTH: u32 = 101;
const FB_HEIGHT: u32 = 101;
const CAMERA_POS: Vec3 = Vec3::new(0.0, 0.0, -1.0);

#[test]
fn draw_unit_cube() {
    let mut fb = FrameBuffer::new(FB_WIDTH, FB_HEIGHT);
    let camera = Camera::for_viewport(FB_WIDTH, FB_HEIGHT).move_to(CAMERA_POS);
    let light = PhongLightModel::new(-camera.direction(), Material::matte(0.9));
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
    // Front **−Z** facet: duplicated normals → uniform **`calc_intensity` = 1.0**.
    let shaded = |indices: [usize; 3]| {
        indices.map(|i| ShadedCorner {
            pos: corners[i],
            intensity: 1.0,
        })
    };
    ShadedFillTriangle::new(shaded([0, 1, 2]), SHAPE_BASE_COLOR).draw(&mut fb);
    ShadedFillTriangle::new(shaded([0, 2, 3]), SHAPE_BASE_COLOR).draw(&mut fb);
    fb
}
