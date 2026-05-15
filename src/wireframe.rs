//! Shared wireframe raster helper (**`draw_edges`**) for cube export binaries.

use crate::scene::cube::{Cube, Edge};
use crate::{Camera, FrameBuffer, Rgb};

/// Rasterize every [`Cube::edges`] segment through **`camera`** (**DDA** in **[`FrameBuffer::draw_line`]**).
pub fn draw_edges(fb: &mut FrameBuffer, camera: &Camera, cube: &Cube, color: Rgb) {
    for Edge(a, b) in cube.edges() {
        fb.draw_line(camera.transform(a), camera.transform(b), color);
    }
}
