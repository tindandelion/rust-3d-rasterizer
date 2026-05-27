//! Orthographic **screen mapping** for this phase of the rasterizer (`src/ortho_camera.rs`).
//! Exposed from the library crate (`thorus_forge::ortho_camera`); export binaries use **`Camera`**.
//!
//! # Public API
//!
//! - [`Camera::for_viewport`](Camera::for_viewport) — default **eye** **`(0, 0, −1)`**, **scene target** **`(0, 0, 0)`**, **world +Y** up.
//! - [`Camera::move_to`](Camera::move_to) — same **target** and **up** policy for a new **eye** (orbit-style **look-at**).
//! - [`Camera::transform`](Camera::transform) — **world `Vec3` → pixel `UVec2`** through **`viewport × view`** (see below).
//!
//! Unit tests live in the **`tests`** submodule below (including **`degenerate_eye_positions`** for **panic** contracts).
//!
//! # Conventions (aligned with repo planning docs)
//!
//! - **Handedness / axes:** **Left-handed** scene, **+Y up**, **+Z forward** (Unity-style intuition).
//! - **View (look-at):** **Scene target** is fixed at **`Vec3::ZERO`**. **Forward** (into the scene) is
//!   **`normalize(ZERO − eye)`**. **Camera up** is **world +Y** projected onto the plane ⊥ forward
//!   (standard Gram–Schmidt). **`Mat4` view** maps world → camera; combined with viewport as
//!   **`transform_matrix = viewport × view`**.
//! - **Default pose (`for_viewport` only):** **Eye** **`(0, 0, −1)`** yields **identity** rotation, so
//!   **view `xy` matches world `xy`**, and test **`world_z_shift_does_not_change_screen_xy`** still applies
//!   (only **world `z`** changes between two points). That shortcut is **not** true after **`move_to`** when
//!   the view is rotated: **orthographic** invariance is “same offset **along `Camera::direction()`**,” not
//!   “same **world `z`**.”
//! - **Panics:** **`move_to` / `for_viewport`** assert **non-degenerate** placement: **eye ≠ scene target**
//!   and **eye** not on the **±world-Y** line through the target (**Y pole** / parallel **up**).
//! - **Projection:** No separate **ortho slab** matrix — **content** is assumed to sit in a useful **xy** span
//!   (historically **NDC-like `[-1, 1]`** for many demo meshes on the default pose). No **`near` / `far`**
//!   clip box here.
//!
//! # Framebuffer layout
//!
//! - **Full surface:** **`width × height`** only — there is **no** inset viewport or sub-rectangle API.
//! - **Pixel grid:** **`(0, 0)`** is **top-left**, **+x** right, **+y** **down** (bitmap style).
//! - **NDC vs bitmap Y:** In view/NDC, **+y** is **up**; framebuffer **+y** is **down**, so **y** uses
//!   **`px_y = −view_y * scale + c_y`** after **`view`** (same flip as **`(−world_y + 1)`** on the **default**
//!   camera when **scale** matches the vertical span).
//! - **Aspect ratio:** **x** and **y** share one scale **`(min(width, height) − 1) / 2`**, centered in the
//!   bitmap (**letterboxing** / **pillarboxing** when **`width ≠ height`**). A world **`[-1, 1]²`** square
//!   maps to a **square** patch of pixels; corners **`(±1, ±1)`** touch the **shorter** image side’s edges
//!   and sit inset on the longer side.
//!
//! # What we are *not* doing (yet)
//!
//! - **Depth:** **`Camera::transform`** returns **only `xy`** pixels; there is **no z-buffer** or packed depth.
//!   **World `z`** can still affect **`xy`** whenever **`view`** rotates (**not** the default pose).
//! - **Border clamp:** Out-of-range **`xy`** is **not** clamped to the image. Values outside **`[-1, 1]`**
//!   still produce **float** intermediates, then **`f32::round`** and **`as u32`**, which **does not** saturate
//!   and may **wrap** for negative floats. **Line / edge** raster code should clip in float space so endpoints
//!   are not snapped to the border by a clamp inside this helper.
//! - **General ortho boxes:** There is no **`[vmin, vmax]`** slab map or full **`orthographic_lh`** frustum.
//! - **Configurable target / poles:** Scene target stays **`Vec3::ZERO`**; **±Y poles** **`panic`** (no fallback
//!   **up** vector yet).

use glam::{Mat3, Mat4, UVec2, Vec3};

use crate::geometry::UnitVec3;

/// Orthographic **world `Vec3` → framebuffer pixel** mapping for one **`width × height`** raster target.
///
/// Precomputes **`viewport × view`** (**look-at** origin, **world +Y** up when valid — see module docs).
/// **`transform`** uses the first two components after the **`Mat4`** multiply (before **`round`**).
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    direction: UnitVec3,
    viewport_transform: Mat4,
    transform_matrix: Mat4,
}

impl Camera {
    pub fn for_viewport(viewport_width: u32, viewport_height: u32) -> Self {
        assert!(viewport_width > 0 && viewport_height > 0);
        let viewport_transform = ndc_viewport_matrix(viewport_width, viewport_height);
        Self::with_viewport_transform(Vec3::new(0.0, 0.0, -1.0), viewport_transform)
    }

    pub fn move_to(self, position: Vec3) -> Self {
        Self::with_viewport_transform(position, self.viewport_transform)
    }

    /// Unit vector in world space pointing **into** the scene: **`normalize(scene_target − eye)`**
    /// with **`scene_target = Vec3::ZERO`** (same convention as the internal **look-at**).
    ///
    /// Consumed by [`TriMesh::visible_facets`](crate::TriMesh) /
    /// [`Facet::is_front_facing`](crate::geometry::Facet::is_front_facing).
    pub fn direction(&self) -> UnitVec3 {
        self.direction
    }

    /// **World** point through **`viewport × view`**; **rounded `xy`** → pixel. For the **default**
    /// **`for_viewport`** pose, **in-bounds demo `xy`** are **NDC-like `[-1, 1]`**; changing **only**
    /// **`world_point.z`** does **not** change **`xy`** (see **`world_z_shift_does_not_change_screen_xy`**).
    /// After **`move_to`**, **world `z`** may change **`xy`** when **view** is rotated.
    pub fn transform(&self, world_point: Vec3) -> UVec2 {
        let p = self.transform_matrix * world_point.extend(1.0);
        UVec2::new(p.x.round() as u32, p.y.round() as u32)
    }

    fn with_viewport_transform(position: Vec3, viewport_transform: Mat4) -> Self {
        let (direction, world_camera_transform) = world_to_camera(position);

        Self {
            direction: direction.into(),
            viewport_transform,
            transform_matrix: viewport_transform * world_camera_transform,
        }
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

fn world_to_camera(camera_position: Vec3) -> (Vec3, Mat4) {
    // Squared length must stay well above zero so `.normalize()` on the world-up projection is stable.
    const MIN_UP_LEN2: f32 = 1e-12;

    let look_at = Vec3::ZERO - camera_position;
    assert!(
        look_at.length_squared() > 0.0,
        "camera position must not coincide with the scene target (origin)"
    );

    let new_z = look_at.normalize();
    let up_ortho = Vec3::Y - Vec3::Y.dot(new_z) * new_z;
    assert!(
        up_ortho.length_squared() > MIN_UP_LEN2,
        "camera must not lie on the ±world-Y line through the scene target (degenerate world-up / Y pole)"
    );
    let new_y = up_ortho.normalize();
    let new_x = new_y.cross(new_z).normalize();

    let rotation = Mat4::from_mat3(Mat3::from_cols(new_x, new_y, new_z));
    let translation = Mat4::from_translation(camera_position);

    let world_to_camera = (translation * rotation).inverse();

    (new_z, world_to_camera)
}

#[cfg(test)]
mod tests {

    mod camera_at_default_pos {
        use super::super::*;

        #[test]
        fn direction_is_pos_z_forward() {
            let camera = Camera::for_viewport(800, 600);
            assert_eq!(camera.direction(), UnitVec3::Z);
        }

        #[test]
        fn world_center_maps_to_viewport_center() {
            let camera = Camera::for_viewport(101, 51);
            assert_eq!(
                camera.transform(Vec3::new(0.0, 0.0, 0.0)),
                UVec2::new(50, 25),
            );
        }

        #[test]
        fn corners_map_to_expected_pixels() {
            let camera = Camera::for_viewport(101, 51);

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
            let camera = Camera::for_viewport(100, 50);

            let left = camera.transform(Vec3::new(-1.0, 0.0, 0.0)).x;
            let right = camera.transform(Vec3::new(1.0, 0.0, 0.0)).x;
            let top = camera.transform(Vec3::new(0.0, 1.0, 0.0)).y;
            let bottom = camera.transform(Vec3::new(0.0, -1.0, 0.0)).y;

            assert_eq!(right - left, bottom - top);
        }

        #[test]
        fn world_z_shift_does_not_change_screen_xy() {
            let camera = Camera::for_viewport(640, 480);

            let a = Vec3::new(0.12, -0.34, 5.0);
            let b = Vec3::new(0.12, -0.34, -900.0);
            assert_eq!(camera.transform(a), camera.transform(b));
        }
    }

    mod camera_world_transform {
        use std::f32::consts;

        use super::super::*;
        use approx::assert_relative_eq;

        #[test]
        fn camera_direction() {
            let camera_pos: Vec3 = Vec3::new(0.0, 1.0, -1.0);
            let camera = Camera::for_viewport(101, 101).move_to(camera_pos);

            assert_relative_eq!(
                Vec3::new(0.0, -consts::SQRT_2 / 2.0, consts::SQRT_2 / 2.0),
                camera.direction()
            );
        }

        #[test]
        fn camera_position_becomes_new_origin() {
            let camera_pos: Vec3 = Vec3::new(0.0, -1.0, -1.0);
            let camera = Camera::for_viewport(101, 101).move_to(camera_pos);

            let camera_pt = camera.transform(camera_pos);
            assert_eq!(UVec2::new(50, 50), camera_pt);
        }

        #[test]
        fn rotate_camera_to_look_at_world_center() {
            let camera_pos: Vec3 = Vec3::new(1.0, 1.0, -1.0);
            let camera = Camera::for_viewport(101, 101).move_to(camera_pos);

            let camera_pt = camera.transform(Vec3::ZERO);
            assert_eq!(UVec2::new(50, 50), camera_pt);
        }

        #[test]
        fn camera_moves_along_z_axis() {
            let camera_pos = Vec3::new(0.0, 0.0, -1.0);
            let camera = Camera::for_viewport(101, 101).move_to(camera_pos);

            let camera_pt = camera.transform(Vec3::new(1.0, 1.0, 0.0));
            assert_eq!(UVec2::new(100, 0), camera_pt);
        }

        #[test]
        fn camera_at_arbitrary_position() {
            let camera_pos = Vec3::new(1.0, 1.0, -1.0);
            let camera = Camera::for_viewport(101, 101).move_to(camera_pos);

            let camera_pt = camera.transform(Vec3::new(1.0, 1.0, 0.0));
            assert_eq!(UVec2::new(85, 30), camera_pt);
        }
    }

    mod degenerate_eye_positions {
        use super::super::*;

        #[test]
        #[should_panic(expected = "scene target")]
        fn move_to_coincident_eye_and_target_panics() {
            let _ = Camera::for_viewport(101, 101).move_to(Vec3::ZERO);
        }

        #[test]
        #[should_panic(expected = "Y pole")]
        fn move_to_on_y_axis_through_target_panics() {
            let _ = Camera::for_viewport(101, 101).move_to(Vec3::new(0.0, 2.0, 0.0));
        }
    }
}
