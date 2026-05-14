//! Pair-programmed, milestone‑by‑milestone orthographic derivation. We assume the drawable **fills
//! the whole framebuffer** (NDC **`xy`** maps to **`0 … width−1`**, **`0 … height−1`** — no inset viewport).
//!
//! Each `#[test]` locks in one behavior of [`rust_3d_rasterizer::projection::orth_projection`]
//! and companion helpers. Implement stubs in `src/projection.rs` until all tests pass.

use approx::assert_relative_eq;
use glam::{Mat4, UVec2, Vec2, Vec3, Vec4};
use rust_3d_rasterizer::projection::{
    clamp_float_pixel_xy, depth_u16_to_linear01, linear01_to_depth_u16, ndc_to_framebuffer_px,
    ndc_z_to_linear01, orth_projection, orth_projection_matrix, orth_world_to_clip_identity_basis,
    orth_world_to_framebuffer_pixel, ortho_point_to_framebuffer_pixel, quantize_pixel_xy,
};

/// **Case 1 — trivial ortho on the symmetric unit cube**
///
/// We fix the convention for this milestone: input coordinates are **already** expressed in the
/// same orthographic view volume we will eventually map to normalized space — here the symmetric
/// box \([-1, 1]\) on **x**, **y**, and **z**. No scale, no bias, no axis flip yet; the projection
/// is the **identity**. Later cases will remap arbitrary axis-aligned bounds into this volume.
#[test]
fn case_01_symmetric_unit_volume_is_identity() {
    let p = Vec3::new(0.25, -0.5, 0.75);
    let vmin = Vec3::splat(-1.0);
    let vmax = Vec3::splat(1.0);
    assert_eq!(orth_projection(p, vmin, vmax), p);
}

/// **Case 2 — symmetric box centered at the origin**
///
/// The view volume is axis-aligned and centered at **0**: on each axis \(a \in \{x,y,z\}\), valid
/// coordinates lie in **`[-half_a, half_a]`** where **`half_a > 0`**. Orthographic normalization
/// maps that interval linearly onto **`[-1, 1]`**. Express that as **`vmin = (-half)`**,
/// **`vmax = (+half)`** component-wise — the general **[`vmin`, `vmax`]** formula must reduce to this
/// behavior.
#[test]
fn case_02_scaled_symmetric_axes_map_to_negative_one_to_one() {
    let half_extents = Vec3::new(4.0, 10.0, 2.0);
    let vmin = -half_extents;
    let vmax = half_extents;
    let p = Vec3::new(2.0, -5.0, 1.0);
    assert_eq!(orth_projection(p, vmin, vmax), Vec3::new(0.5, -0.5, 0.5),);

    let half_extents = Vec3::splat(8.0);
    let vmin = -half_extents;
    let vmax = half_extents;
    let q = Vec3::new(-8.0, 8.0, 0.0);
    assert_eq!(orth_projection(q, vmin, vmax), Vec3::new(-1.0, 1.0, 0.0));
}

/// **Case 3 — arbitrary axis-aligned box (off-center volumes)**
///
/// The preimage is still an axis-aligned box, but **`vmin`/`vmax` need not be symmetric about 0**.
/// Per axis \(a\), map **`[vmin_a, vmax_a] → [-1, 1]`** with the unique affine map that sends the
/// endpoints to \(-1\) and \(+1\): **`(-1)` at `vmin`**, **`(+1)` at `vmax`**. Equivalently, translate
/// to the box center, then scale by half the axis length (`(max - min) / 2` in the denominator when
/// you write it as “subtract center, divide by half-extent”).
#[test]
fn case_03_off_center_bounds_map_endpoints_and_center() {
    // x ∈ [10, 20], y ∈ [2, 6], z ∈ [0, 8]
    let vmin = Vec3::new(10.0, 2.0, 0.0);
    let vmax = Vec3::new(20.0, 6.0, 8.0);

    assert_eq!(
        orth_projection(vmin, vmin, vmax),
        Vec3::new(-1.0, -1.0, -1.0),
    );
    assert_eq!(orth_projection(vmax, vmin, vmax), Vec3::splat(1.0),);

    let center = 0.5 * (vmin + vmax); // (15, 4, 4)
    assert_eq!(orth_projection(center, vmin, vmax), Vec3::ZERO,);

    // Interior point not at the midpoint on every axis simultaneously
    let p = Vec3::new(12.0, 6.0, 2.0); // xmax on y ⇒ ndc_y = +1
    assert_eq!(orth_projection(p, vmin, vmax), Vec3::new(-0.6, 1.0, -0.5),);
}

/// **Case 4 — same map as rows 1–3, but as a 4×4 (column-vector convention)**
///
/// Pack the axis-wise affine **`[-1, 1]`** normalization into **`Proj`** such that **`Proj * Vec4(v, 1)`**
/// reproduces **`orth_projection`**. Rows 1–3 already imply per-axis **`scale`** and **`translation`**
/// in clip space (`ndc`): solve for the diagonal **`2 / (max - min)`** and the **`-(max + min)/(max-min)`**
/// offset. Keep **`w'` = 1`** for this orthographic core (later you may multiply by a view matrix
/// on the left and still accumulate into the same **`Proj`** pattern).
#[test]
fn case_04_matrix_matches_point_projection_on_corners_and_interior() {
    let vmin = Vec3::new(10.0, 2.0, 0.0);
    let vmax = Vec3::new(20.0, 6.0, 8.0);

    let proj = orth_projection_matrix(vmin, vmax);

    for p in [
        Vec3::new(0.25, -0.5, 0.75), // deliberately outside the bbox; both paths must still agree
        Vec3::new(10.0, 2.0, 0.0),
        Vec3::new(20.0, 6.0, 8.0),
        0.5 * (vmin + vmax),
        Vec3::new(12.0, 6.0, 2.0),
    ] {
        let expected_xyz = orth_projection(p, vmin, vmax);
        let clip = proj * Vec4::from((p.x, p.y, p.z, 1.0));

        assert_relative_eq!(clip.truncate(), expected_xyz, epsilon = 1e-6,);
        assert_eq!(clip.w, 1.0, "homogeneous w must stay 1 for this orth core");
    }
}

/// **Case 5 — NDC to framebuffer (`xy`; top-left raster, Y flipped)**
///
/// After ortho, **`x,y`** live in **`[-1, 1]`**. Raster output uses a **bitmap-style** **`(0,0)` =
/// top-left** system with **`+y` down**, while **`ndc_y > 0`** means **above** center in view space —
/// hence one **invert on `y`**. Stretch **`[-1, 1]`** across **`width`** and **`height`** using **`(dimension - 1)`**
/// factors so **`±1`** **NDC hits the outer pixel columns/rows**.
#[test]
fn case_05_ndc_xy_maps_to_framebuffer_with_top_left_origin_and_y_flip() {
    let width = 101_u32;
    let height = 51_u32;

    assert_relative_eq!(
        ndc_to_framebuffer_px(Vec2::NEG_ONE, width, height),
        Vec2::new(0.0, (height - 1) as f32),
        epsilon = 1e-6,
    );
    assert_relative_eq!(
        ndc_to_framebuffer_px(Vec2::ONE, width, height),
        Vec2::new((width - 1) as f32, 0.0),
        epsilon = 1e-6,
    );
    assert_relative_eq!(
        ndc_to_framebuffer_px(Vec2::new(-1.0, 1.0), width, height),
        Vec2::ZERO,
        epsilon = 1e-6,
    );
    assert_relative_eq!(
        ndc_to_framebuffer_px(Vec2::new(1.0, -1.0), width, height),
        Vec2::new((width - 1) as f32, (height - 1) as f32),
        epsilon = 1e-6,
    );

    let center_px = Vec2::new((width - 1) as f32, (height - 1) as f32) * 0.5;
    assert_relative_eq!(
        ndc_to_framebuffer_px(Vec2::ZERO, width, height),
        center_px,
        epsilon = 1e-6,
    );

    let vmin = Vec3::splat(-1.0);
    let vmax = Vec3::splat(1.0);
    let p = Vec3::new(-0.5, 1.0, 0.0);
    assert_relative_eq!(
        ndc_to_framebuffer_px(orth_projection(p, vmin, vmax).truncate(), width, height),
        Vec2::new(25.0, 0.0),
        epsilon = 1e-5,
    );
}

/// **Case 7 — linear depth: NDC \(z\) in \([-1,1]\) → \([0,1]\)**  
///
/// **`x/y`** are already squared away for raster placement; **`z`** carries **“which depth inside
/// the ortho slab”**. Before any future **reverse‑Z**, Vulkan/D3D-style depth ranges, or non-linear
/// perspective depth, we adopt the **straight affine** map **`linear01 = (z_ndc + 1) / 2`**. Cheap
/// **z-sort / depth-buffer stubs** can consume **`f32`** in **`[0,1]`** without reinterpretation drama.
#[test]
fn case_07_ndc_z_linearly_encodes_between_zero_and_one() {
    assert_relative_eq!(ndc_z_to_linear01(-1.0), 0.0, epsilon = 1e-7);
    assert_relative_eq!(ndc_z_to_linear01(1.0), 1.0, epsilon = 1e-7);
    assert_relative_eq!(ndc_z_to_linear01(0.0), 0.5, epsilon = 1e-7);
    assert_relative_eq!(ndc_z_to_linear01(0.25), 0.625, epsilon = 1e-7);

    // Slab corners on **z**, identity on **x/y** (−1..1³); sample mid‑depth in view → center NDC → 0.5
    let vmin = Vec3::splat(-1.0);
    let vmax = Vec3::splat(1.0);
    assert_relative_eq!(
        ndc_z_to_linear01(orth_projection(Vec3::new(0.3, -0.2, 0.75), vmin, vmax).z),
        0.875,
        epsilon = 1e-6,
    );

    let vmin = Vec3::new(10.0, 2.0, 5.0);
    let vmax = Vec3::new(20.0, 6.0, 15.0); // depth span 10 ⇒ z = 15 is **far slab** ⇒ **ndc_z = +1**
    assert_relative_eq!(
        ndc_z_to_linear01(orth_projection(Vec3::new(12.0, 4.0, 15.0), vmin, vmax).z),
        1.0,
        epsilon = 1e-6,
    );
}

/// **Case 8 — float pixel coordinates → integer indices (`set_pixel` bridge)**
///
/// Raster APIs index pixels on an integer lattice. After [`ndc_to_framebuffer_px`],
/// **`quantize_pixel_xy`** applies the standard **`f32::round`** per axis so
/// “corner” samples that land cleanly on **`f32`** wholes stay stable while sub-pixel leftovers
/// pick a neighboring column/row.
#[test]
fn case_08_quantized_pixels_round_float_framebuffer_coords() {
    assert_eq!(
        quantize_pixel_xy(Vec2::new(0.0, 49.499_94)),
        UVec2::new(0, 49),
    );
    assert_eq!(quantize_pixel_xy(Vec2::new(10.2, 20.8)), UVec2::new(10, 21),);
    // **`599.499_985` is not representable in `f32`** — it becomes **exactly `599.5`**, so `round → 600`.
    assert_eq!(
        quantize_pixel_xy(Vec2::new(99.499_985, 599.499)),
        UVec2::new(99, 599)
    );

    // Half-away-from-zero midpoint for **positive** values
    assert_eq!(quantize_pixel_xy(Vec2::splat(10.5)), UVec2::splat(11));

    // Target column **199**: **\((ndc_x + 1) \cdot (W-1)/2 = 199\)** ⇒ **`ndc_x = -401/799`** (use **799** for **x**, not **599**).
    let vmin = Vec3::splat(-1.0);
    let vmax = Vec3::splat(1.0);
    let width = 800_u32;
    let height = 600_u32;
    let ndc_target_x = -401.0_f32 / 799.0_f32;
    let p = Vec3::new(ndc_target_x, 1.0, 0.0);
    let float_px = ndc_to_framebuffer_px(orth_projection(p, vmin, vmax).truncate(), width, height);

    assert_relative_eq!(
        float_px,
        Vec2::new(199.0, 0.0),
        epsilon = 1e-3,
        max_relative = 1e-5,
    );
    assert_eq!(quantize_pixel_xy(float_px), UVec2::new(199, 0));

    assert_eq!(
        quantize_pixel_xy(ndc_to_framebuffer_px(Vec2::NEG_ONE, 160, 120)),
        UVec2::new(0, 119),
    );
}

/// **Case 9 — clamp float pixels to the framebuffer (NDC overshoot guard)**
///
/// Out-of-volume points still produce **finite** `f32` rows/columns from the linear map, but those
/// may sit **below 0** or **past `width−1` / `height−1`**. Before [`quantize_pixel_xy`], squeeze into
/// the valid index range so rounding never sees **negative** floats (which would **wrap** as `u32`).
#[test]
fn case_09_clamp_float_pixels_to_surface_bounds() {
    let width = 400_u32;
    let height = 300_u32;

    assert_eq!(
        clamp_float_pixel_xy(Vec2::new(-12.5, 150.25), width, height),
        Vec2::new(0.0, 150.25),
    );
    assert_eq!(
        clamp_float_pixel_xy(Vec2::new(500.0, -1.0), width, height),
        Vec2::new(399.0, 0.0),
    );
    assert_eq!(
        clamp_float_pixel_xy(Vec2::new(200.0, 299.0), width, height),
        Vec2::new(200.0, 299.0),
    );

    let wide = ndc_to_framebuffer_px(Vec2::new(3.0, 0.0), width, height);
    assert_eq!(wide, Vec2::new(798.0, 149.5));
    let safe = clamp_float_pixel_xy(wide, width, height);
    assert_eq!(safe, Vec2::new(399.0, 149.5));
    assert_eq!(quantize_pixel_xy(safe), UVec2::new(399, 150));

    let low = clamp_float_pixel_xy(
        ndc_to_framebuffer_px(Vec2::new(0.0, -5.0), width, height),
        width,
        height,
    );
    assert_eq!(low, Vec2::new(199.5, 299.0));
    assert_eq!(quantize_pixel_xy(low), UVec2::new(200, 299));
}

/// **Case 10 — `linear01` depth → `u16` (dense fixed-point slab)**
///
/// GPU and software paths often narrow **`f32`** to **`u16`** for depth storage. After Case 7
/// (**`linear01`**), multiply by **`65535`**, apply **`f32::round`**, truncate to **`u16`**. Out-of-band
/// values **saturate** to **`[0, 1]`** before scaling.
#[test]
fn case_10_linear_depth_packs_into_u16() {
    assert_eq!(linear01_to_depth_u16(0.0), 0);
    assert_eq!(linear01_to_depth_u16(1.0), 65535);
    assert_eq!(linear01_to_depth_u16(0.5), 32768);

    assert_eq!(linear01_to_depth_u16(-2.5), 0);
    assert_eq!(linear01_to_depth_u16(2.5), 65535);

    let vmin = Vec3::splat(-1.0);
    let vmax = Vec3::splat(1.0);
    let ndc_z = orth_projection(Vec3::new(4.0, -1.5, -1.0), vmin, vmax).z;

    assert_eq!(linear01_to_depth_u16(ndc_z_to_linear01(ndc_z)), 0,);

    let vmin = Vec3::new(-4.0, 0.0, 24.0);
    let vmax = Vec3::new(8.0, 24.0, 40.0);
    assert_eq!(
        linear01_to_depth_u16(ndc_z_to_linear01(
            orth_projection(Vec3::new(0.0, 16.0, 32.0), vmin, vmax).z,
        )),
        32768,
    );
}

/// **Case 11 — ortho view position → discrete **`UVec2`** (full-buffer pipeline)**
///
/// One entry point that stitches every **xy** milestone: [`orth_projection`], [`ndc_to_framebuffer_px`],
/// [`clamp_float_pixel_xy`], [`quantize_pixel_xy`]. Keeps wireframe `set_pixel` call sites boring.
#[test]
fn case_11_ortho_point_composes_to_framebuffer_pixel() {
    let vmin = Vec3::splat(-1.0);
    let vmax = Vec3::splat(1.0);

    assert_eq!(
        ortho_point_to_framebuffer_pixel(Vec3::new(-401.0 / 799.0, 1.0, 0.0), vmin, vmax, 800, 600,),
        UVec2::new(199, 0),
    );

    assert_eq!(
        ortho_point_to_framebuffer_pixel(Vec3::ZERO, vmin, vmax, 101, 51),
        UVec2::new(50, 25),
    );

    assert_eq!(
        ortho_point_to_framebuffer_pixel(Vec3::new(-3.0, 0.0, 0.0), vmin, vmax, 400, 300),
        UVec2::new(0, 150),
    );
    assert_eq!(
        ortho_point_to_framebuffer_pixel(Vec3::new(3.0, 0.0, 0.0), vmin, vmax, 400, 300),
        UVec2::new(399, 150),
    );
}

/// **Case 13 — world → clip with a **translation-only** view (axes still world-aligned)**
///
/// Build **`View = Translate(−eye)`**, **`Proj = [`orth_projection_matrix`](orth_projection_matrix)`**,
/// assert **`Proj · View · (x,y,z,1)`** reproduces **`orth_projection(world − eye, …).extend(1)`** — the
/// usual bootstrap before adding **rotation** (look-at) in a later milestone.
#[test]
fn case_13_world_to_clip_matches_subtract_camera_then_ortho() {
    let vmin = Vec3::new(10.0, 2.0, 0.0);
    let vmax = Vec3::new(20.0, 6.0, 8.0);

    let eye = Vec3::new(5.0, -1.0, 12.0);
    let world = Vec3::new(17.0, 5.0, 14.0);
    let view = world - eye;

    let expected = orth_projection(view, vmin, vmax).extend(1.0);
    let clip = orth_world_to_clip_identity_basis(world, eye, vmin, vmax);

    assert_relative_eq!(
        clip.truncate(),
        expected.truncate(),
        epsilon = 1e-4,
        max_relative = 1e-5
    );
    assert_eq!(clip.w, 1.0);
    assert_eq!(expected.w, 1.0);

    let proj = orth_projection_matrix(vmin, vmax);
    let view_m = Mat4::from_translation(-eye);
    let manual_mat = proj * view_m * world.extend(1.0);
    assert_relative_eq!(clip, manual_mat, epsilon = 1e-4, max_relative = 1e-5);

    let unit_lo = Vec3::splat(-1.0);
    let unit_hi = Vec3::splat(1.0);
    let clip_unit = orth_world_to_clip_identity_basis(
        Vec3::new(100.5, -200.25, 50.0),
        Vec3::new(100.0, -200.0, 50.0),
        unit_lo,
        unit_hi,
    );
    assert_relative_eq!(
        clip_unit.truncate(),
        Vec3::new(0.5, -0.25, 0.0),
        epsilon = 1e-5,
    );
    assert_eq!(clip_unit.w, 1.0);
}

/// **Case 14 — world → framebuffer pixel (translation-only camera)**
///
/// Same as Case 11, but **`p_view = p_world − eye`**. One call site for “drop a world vertex on the
/// raster grid” before you add **model** matrices or **look-at** rotation.
#[test]
fn case_14_world_to_framebuffer_pixel_composes_view_subtract() {
    let unit_lo = Vec3::splat(-1.0);
    let unit_hi = Vec3::splat(1.0);
    let eye = Vec3::new(10.0, -5.0, 3.0);
    let view = Vec3::new(-401.0 / 799.0, 1.0, 0.0);
    let world = eye + view;

    assert_eq!(
        orth_world_to_framebuffer_pixel(world, eye, unit_lo, unit_hi, 800, 600),
        ortho_point_to_framebuffer_pixel(view, unit_lo, unit_hi, 800, 600),
    );
    assert_eq!(
        orth_world_to_framebuffer_pixel(world, eye, unit_lo, unit_hi, 800, 600),
        UVec2::new(199, 0),
    );

    let world_far = eye + Vec3::new(3.0, 0.0, 0.0);
    assert_eq!(
        orth_world_to_framebuffer_pixel(world_far, eye, unit_lo, unit_hi, 400, 300),
        UVec2::new(399, 150),
    );
}

/// **Case 16 — decode **`u16`** depth back toward **`linear01`**
///
/// **[`linear01_to_depth_u16`]** loses information; **`depth_u16_to_linear01`** is the deterministic
/// read path **`d / 65535`**. Perfect round-trip only when **`t`** reproduces exactly after **`round`**.
#[test]
fn case_16_depth_u16_reconstructs_linear01_fraction() {
    assert_relative_eq!(depth_u16_to_linear01(0), 0.0, epsilon = 1e-7);
    assert_relative_eq!(depth_u16_to_linear01(65535), 1.0, epsilon = 1e-7);
    assert_relative_eq!(
        depth_u16_to_linear01(32768),
        32768.0 / 65535.0,
        epsilon = 1e-7
    );

    let encoded = linear01_to_depth_u16(0.3_f32);
    assert_relative_eq!(
        depth_u16_to_linear01(encoded),
        0.3,
        epsilon = 1e-4,
        max_relative = 1e-4,
    );
}
