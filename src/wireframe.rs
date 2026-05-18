//! Shared wireframe raster helper (**`draw_edges`**) for cube export binaries.

use crate::scene::cube::{Cube, Edge};
use crate::{Camera, FrameBuffer, Line, Rgb};

/// Rasterize [`Cube::visible_edges`] through **`camera`** (**DDA** in **[`Line::draw`]**).
///
/// View direction comes from [`Camera::direction`] (same axis used for back-face classification).
pub fn draw_edges(fb: &mut FrameBuffer, camera: &Camera, cube: &Cube, color: Rgb) {
    let forward = camera.direction();
    for Edge(a, b) in cube.visible_edges(forward) {
        Line::new(camera.transform(a), camera.transform(b), color).draw(fb);
    }
}
