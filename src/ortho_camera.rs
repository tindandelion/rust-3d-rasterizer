//! Orthographic **screen mapping** for this phase of the rasterizer (`src/ortho_camera.rs`).
//! Exposed from the library crate (`thorus_forge::ortho_camera`); the **`still-cube`** bin uses **`Camera`**.
//!
//! # Public API
//!
//! **`[`Camera`]`** holds viewport dimensions at construction; **`[`Camera::transform`]** maps **`Vec3`** to
//! **`UVec2`** pixels. Unit tests live in the **`tests`** submodule below.
//!
//! # Conventions (aligned with repo planning docs)
//!
//! - **Handedness / axes:** **Left-handed** scene, **+Y up**, **+Z forward** (Unity-style intuition).
//! - **Fixed camera:** **`eye = (0, 0, −1)`** in world space, looking at **`(0, 0, 0)`**, **`up = +Y`**.
//!   With translation-only view, **`view_xy = world_xy`** (the eye has no **x** / **y** offset), and
//!   **`view_z = world_z + 1`**. **`Camera::transform`** uses **`world_point.x`** and **`world_point.y`** as the
//!   values that feed the raster mapping — equivalent to **NDC `xy` in `[-1, 1]`** after the fixed camera
//!   for geometry lying in the **`z = 0`** plane used in these tests.
//! - **Projection:** **`Proj`** is **identity** (no extra ortho scale / no **`vmin` / `vmax`** frustum
//!   parameters). Content is assumed to already sit in the **unit** range in **x** and **y** when you
//!   care about in-bounds pixels.
//!
//! # Framebuffer layout
//!
//! - **Full surface:** **`width × height`** only — there is **no** inset viewport or sub-rectangle API.
//! - **Pixel grid:** **`(0, 0)`** is **top-left**, **+x** right, **+y** **down** (bitmap style).
//! - **NDC vs bitmap Y:** In view/NDC, **+y** is **up**; framebuffer **+y** is **down**, so **y** uses
//!   **`px_y = −world_y * scale + c_y`** (same flip as **`(−world_y + 1)`** when **scale** matches the
//!   vertical span).
//! - **Aspect ratio:** **x** and **y** share one scale **`(min(width, height) − 1) / 2`**, centered in the
//!   bitmap (**letterboxing** / **pillarboxing** when **`width ≠ height`**). A world **`[-1, 1]²`** square
//!   maps to a **square** patch of pixels; corners **`(±1, ±1)`** touch the **shorter** image side’s edges
//!   and sit inset on the longer side.
//!
//! # What we are *not* doing (yet)
//!
//! - **Depth:** **`world_point.z`** does **not** affect **`Camera::transform`** — it is ignored for **`xy`** (see test
//!   **`world_z_shift_does_not_change_screen_xy`**). No **z-buffer**, **`linear01`**, or **`u16`** depth
//!   packing here.
//! - **Border clamp:** Out-of-range **`xy`** is **not** clamped to the image. Values outside **`[-1, 1]`**
//!   still produce **float** intermediates, then **`f32::round`** and **`as u32`**, which **does not** saturate
//!   and may **wrap** for negative floats. **Line / edge** raster code should clip in float space so endpoints
//!   are not snapped to the border by a clamp inside this helper.
//! - **General ortho boxes:** There is no **`[vmin, vmax]`** slab map or **`orth_projection_matrix`** in
//!   this stripped-down path.
//! - **Moving / rotating camera:** No **`look_at`**, no **`Mat4`** view matrix in this module — a
//!   different **`eye`** or orientation would require extending or replacing this API.

use glam::{Mat4, UVec2, Vec3};

use crate::geometry::Normal3;

/// Fixed orthographic **`Vec3`** → framebuffer pixel mapping for one **`width × height`** raster target.
///
/// Holds a precomputed **`Mat4`** (**NDC `xy`** + **`z`** through scale part → pixel space before **`round`**).
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    transform_matrix: Mat4,
}

impl Camera {
    /// Builds a camera for a **`width × height`** framebuffer.
    ///
    /// # Panics
    ///
    /// **`width`** and **`height`** must both be **non-zero**. Otherwise **`panic!`**s via **`assert!`**.
    pub fn new(viewport_width: u32, viewport_height: u32) -> Self {
        assert!(viewport_width > 0 && viewport_height > 0);

        Self {
            transform_matrix: ndc_viewport_matrix(viewport_width, viewport_height),
        }
    }

    /// Unit vector in world space pointing **into** the scene (**view / forward**).
    ///
    /// Matches this module’s fixed pose (**`+Z`** forward, left-handed — see module docs). Consumed by
    /// [`TriMesh::visible_facets`](crate::TriMesh) / [`Facet::is_front_facing`](crate::scene::facet::Facet::is_front_facing).
    pub fn direction(&self) -> Normal3 {
        Normal3::Z
    }

    /// **`world_point.x`** / **`y`** are NDC-style **`[-1, 1]`** for in-bounds framing; **`z`** does not affect **`xy`**.
    pub fn transform(&self, world_point: Vec3) -> UVec2 {
        let p = self.transform_matrix * world_point.extend(1.0);
        UVec2::new(p.x.round() as u32, p.y.round() as u32)
    }
}

/// Builds **NDC `xy`** → homogeneous pixel coordinates (**before** `round` / `as u32`).
///
/// **Requires:** **`width > 0`** and **`height > 0`**.
fn ndc_viewport_matrix(width: u32, height: u32) -> Mat4 {
    let max_x = (width - 1) as f32;
    let max_y = (height - 1) as f32;
    let scale_dimension = max_x.min(max_y);
    let scale = scale_dimension / 2.0;
    let cx = max_x / 2.0;
    let cy = max_y / 2.0;

    // Column vectors: `(T * S) * v` scales first, then translates so
    // `x' = scale * x + cx`, `y' = -scale * y + cy` (bitmap **+y** down).
    Mat4::from_translation(Vec3::new(cx, cy, 0.0)) * Mat4::from_scale(Vec3::new(scale, -scale, 1.0))
}

#[cfg(test)]
mod tests {
    //! Covers **`Camera::transform`** on **`z = 0`** NDC corners/interior unless **`z`** independence is the point.

    use super::*;

    #[test]
    fn direction_is_pos_z_forward() {
        let camera = Camera::new(800, 600);
        assert_eq!(camera.direction(), Normal3::Z);
    }

    #[test]
    fn world_center_maps_to_viewport_center() {
        let camera = Camera::new(101, 51);
        assert_eq!(
            camera.transform(Vec3::new(0.0, 0.0, 0.0)),
            UVec2::new(50, 25),
        );
    }

    #[test]
    fn corners_map_to_expected_pixels() {
        let camera = Camera::new(101, 51);

        assert_eq!(
            camera.transform(Vec3::new(-1.0, -1.0, 0.0)),
            UVec2::new(25, 50),
        );
        assert_eq!(
            camera.transform(Vec3::new(1.0, 1.0, 0.0)),
            UVec2::new(75, 0),
        );
        assert_eq!(
            camera.transform(Vec3::new(-1.0, 1.0, 0.0)),
            UVec2::new(25, 0),
        );
        assert_eq!(
            camera.transform(Vec3::new(1.0, -1.0, 0.0)),
            UVec2::new(75, 50),
        );
    }

    #[test]
    fn non_square_viewport_preserves_equal_axis_span_in_pixels() {
        let camera = Camera::new(crate::SCENE_WIDTH, crate::SCENE_HEIGHT);

        let left = camera.transform(Vec3::new(-1.0, 0.0, 0.0)).x;
        let right = camera.transform(Vec3::new(1.0, 0.0, 0.0)).x;
        let top = camera.transform(Vec3::new(0.0, 1.0, 0.0)).y;
        let bottom = camera.transform(Vec3::new(0.0, -1.0, 0.0)).y;

        assert_eq!(right - left, bottom - top);
    }

    #[test]
    fn world_z_shift_does_not_change_screen_xy() {
        let camera = Camera::new(640, 480);
        let a = Vec3::new(0.12, -0.34, 5.0);
        let b = Vec3::new(0.12, -0.34, -900.0);
        assert_eq!(camera.transform(a), camera.transform(b));
    }
}
