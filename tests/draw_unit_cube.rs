//! Integration: orthographic fill of the default unit cube against a hand-built golden framebuffer.
//!
//! With **`Camera::direction` = +Z**, the strictly front-facing hull facet is the **−Z** cap (outward normal
//! **`NEG_Z`** → [`thorus_forge::CUBE_FACE_PALETTE`]\[0\] blue). On this **`FB_WIDTH`×`FB_HEIGHT`** canvas,
//! **`scale = (min(w,h) − 1) / 2`** is an integer, so unit-cube **`±0.5`** corners land exactly on
//! **`FILLED_MIN…FILLED_LAST`** (**no** intermediate **`f32::round`**).

use glam::UVec2;
use thorus_forge::{
    CUBE_FACE_PALETTE, Camera, FillQuad, FrameBuffer, draw_faces, scene::cube::Cube,
};

const FB_WIDTH: u32 = 101;
const FB_HEIGHT: u32 = 101;

#[test]
fn draw_unit_cube() {
    let mut fb = FrameBuffer::new(FB_WIDTH, FB_HEIGHT);
    let camera = Camera::new(FB_WIDTH, FB_HEIGHT);
    let cube = Cube::default();

    draw_faces(&mut fb, &camera, &cube);

    let expected = expected_framebuffer_unit_cube_camera_front();
    assert_eq!(fb, expected);
}

fn expected_framebuffer_unit_cube_camera_front() -> FrameBuffer {
    let mut fb = FrameBuffer::new(FB_WIDTH, FB_HEIGHT);
    let color = CUBE_FACE_PALETTE[0];
    let corners = [
        UVec2::new(25, 25),
        UVec2::new(25, 75),
        UVec2::new(75, 75),
        UVec2::new(75, 25),
    ];
    FillQuad::new(corners, color).draw(&mut fb);
    fb
}
