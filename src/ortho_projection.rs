//! Orthographic **screen mapping** for this phase of the rasterizer (`src/ortho_projection.rs`, wired from
//! **`main.rs`** as **`mod ortho_projection`**).
//!
//! # Public API
//!
//! The **`pub fn`** surface is **`project`** only. Unit tests live in the **`tests`** submodule below.
//!
//! # Conventions (aligned with repo planning docs)
//!
//! - **Handedness / axes:** **Left-handed** scene, **+Y up**, **+Z forward** (Unity-style intuition).
//! - **Fixed camera:** **`eye = (0, 0, −1)`** in world space, looking at **`(0, 0, 0)`**, **`up = +Y`**.
//!   With translation-only view, **`view_xy = world_xy`** (the eye has no **x** / **y** offset), and
//!   **`view_z = world_z + 1`**. This module uses **`world_point.x`** and **`world_point.y`** as the
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
//! - **Depth:** **`world_point.z`** does **not** affect **`project`** — it is ignored for **xy** (see test
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

use glam::{UVec2, Vec3};

/// Map a **world-space** point to an **integer pixel** in a **`width × height`** framebuffer.
///
/// Uses **`world_point.x`** and **`world_point.y`** as **NDC-style coordinates in `[-1, 1]`** for
/// well-behaved in-bounds output; **`world_point.z`** is **ignored**.
///
/// **x** / **y** use the **same** world-to-pixel scale so a square in **`xy`** stays **square** on
/// non-square viewports (content centered; bars on the long side).
///
/// Internally this is **linear map → `f32::round` → `as u32`** on each axis (no separate float-pixel type
/// exposed). Does **not** clamp to **`0 … width - 1`** / **`0 … height - 1`** before the **`u32`** cast;
/// clip beforehand if you require in-bounds indices. If **`width == 0`** or **`height == 0`**, returns
/// **`(0, 0)`**.
pub fn project(world_point: Vec3, width: u32, height: u32) -> UVec2 {
    if width == 0 || height == 0 {
        return UVec2::ZERO;
    }

    let max_x = (width - 1) as f32;
    let max_y = (height - 1) as f32;
    let dim_minus_1 = max_x.min(max_y);
    let scale = dim_minus_1 / 2.0;
    let cx = max_x / 2.0;
    let cy = max_y / 2.0;

    let px = world_point.x * scale + cx;
    let py = -world_point.y * scale + cy;

    UVec2::new(px.round() as u32, py.round() as u32)
}

#[cfg(test)]
mod tests {
    //! Covers **[`super::project`]** on **`z = 0`** NDC corners/interior unless **`z`** independence is the point.

    use super::*;

    #[test]
    fn corners_map_to_expected_pixels() {
        let width = 101_u32;
        let height = 51_u32;

        assert_eq!(
            project(Vec3::new(-1.0, -1.0, 0.0), width, height),
            UVec2::new(25, height - 1),
        );
        assert_eq!(
            project(Vec3::new(1.0, 1.0, 0.0), width, height),
            UVec2::new(75, 0),
        );
        assert_eq!(
            project(Vec3::new(-1.0, 1.0, 0.0), width, height),
            UVec2::new(25, 0),
        );
        assert_eq!(
            project(Vec3::new(1.0, -1.0, 0.0), width, height),
            UVec2::new(75, height - 1),
        );

        assert_eq!(
            project(Vec3::new(0.0, 0.0, 0.0), width, height),
            UVec2::new(50, 25),
        );

        assert_eq!(
            project(Vec3::new(-0.5, 1.0, 0.0), width, height),
            UVec2::new(38, 0),
        );
    }

    #[test]
    fn square_viewport_corners_touch_frame_edges() {
        let n = 51_u32;
        let h = n - 1;

        assert_eq!(project(Vec3::new(-1.0, -1.0, 0.0), n, n), UVec2::new(0, h),);
        assert_eq!(project(Vec3::new(1.0, 1.0, 0.0), n, n), UVec2::new(h, 0),);
    }

    #[test]
    fn non_square_viewport_preserves_equal_axis_span_in_pixels() {
        let w = 800_u32;
        let h = 600_u32;
        let left = project(Vec3::new(-1.0, 0.0, 0.0), w, h).x;
        let right = project(Vec3::new(1.0, 0.0, 0.0), w, h).x;
        let top = project(Vec3::new(0.0, 1.0, 0.0), w, h).y;
        let bottom = project(Vec3::new(0.0, -1.0, 0.0), w, h).y;

        assert_eq!(right - left, bottom - top);
    }

    #[test]
    fn world_z_shift_does_not_change_screen_xy() {
        let w = 640_u32;
        let h = 480_u32;
        let a = Vec3::new(0.12, -0.34, 5.0);
        let b = Vec3::new(0.12, -0.34, -900.0);
        assert_eq!(project(a, w, h), project(b, w, h));
    }
}
