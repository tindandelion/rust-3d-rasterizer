//! Orthographic projection helpers.
//!
//! Derived step by step with `tests/ortho_projection.rs`. **Convention:** raster mapping uses the **full**
//! framebuffer **`width × height`** (Cases 5+); inset viewports were dropped from this module’s API.

use glam::{Mat4, UVec2, Vec2, Vec3, Vec4};

/// Maps a single point through the current orthographic projection (see tests for the exact
/// contract at each milestone).
///
/// `vmin` / `vmax` are **inclusive** corners of an axis-aligned box: on each axis \(a\),
/// coordinates in **`[vmin_a, vmax_a]`** map linearly to **`[-1, 1]`**. Require **`vmin_a < vmax_a`**
/// for every axis.
pub fn orth_projection(point: Vec3, vmin: Vec3, vmax: Vec3) -> Vec3 {
    let center = 0.5 * (vmin + vmax);
    let half = 0.5 * (vmax - vmin);
    (point - center) / half
}

/// **Column-major** orthographic matrix `Proj` matching [`orth_projection`]: for a view‑space
/// point `p`, the product **`Proj * Vec4(p, 1)`** must agree with **`orth_projection(p, vmin, vmax).extend(1)`**
/// on all four components (**`w` stays 1`**). Use the usual axis-wise map that sends **`vmin → -1`**
/// and **`vmax → +1`** (same preimage box as [`orth_projection`]).
///
/// Stored in **`glam` column-major layout** (`Mat4::from_cols` takes column vectors in order).
pub fn orth_projection_matrix(vmin: Vec3, vmax: Vec3) -> Mat4 {
    let center = 0.5 * (vmin + vmax);
    let scale_vector = 2.0 / (vmax - vmin);

    let translation = Mat4::from_translation(-center);
    let scale = Mat4::from_scale(scale_vector);
    scale * translation
}

/// Maps **normalized device coordinates** **`x,y ∈ [-1, 1]`** (output of [`orth_projection`] /
/// homogeneous clip **`xy`** after dividing out **`w`** when **`w ≡ 1`**) onto **floating pixel
/// coordinates** for a framebuffer of **`width × height`** pixels.
///
/// Conventions (**deliberately explicit** — match these in phase‑1 raster → WebP):
/// - Raster origin **`(0, 0)`** is the **top-left** pixel; **`+x`** is **right**; **`+y`** is **down**.
/// - NDC **`+y`** is **up**. That implies an axis **flip**: **`ndc_y = +1`** is the **top** row (**`py → 0`**).
/// - The segment **`[-1, 1]`** along each axis spans **corner pixel centers**, i.e. we scale by **`(dimension - 1)`**
///   so **`ndc = ±1`** lands on **`0`** or **`width - 1` / `height - 1`** (inclusive raster range).
///
/// **`z`** / depth is out of scope here (wireframe milestones may ignore it entirely at first).
pub fn ndc_to_framebuffer_px(ndc_xy: Vec2, width: u32, height: u32) -> Vec2 {
    let ndc_x = (ndc_xy.x + 1.0) * (width - 1) as f32 / 2.0;
    let ndc_y = (-ndc_xy.y + 1.0) * (height - 1) as f32 / 2.0;

    Vec2::new(ndc_x, ndc_y)
}

/// Linearly remaps **`ndc_z ∈ [-1, 1]`** (the third component from [`orth_projection`] / clip **before**
/// any platform-specific depth tricks) into **`[0, 1]`**, with **`-1 → 0`**, **`+1 → 1`**, **`0 → 0.5`**.
///
/// This is the **simplest** depth encoding for the learning path: a straight affine lift of the same
/// ortho interval you already use for **`x` and `y`**. Real APIs often use different **`z` ranges**
/// (e.g. **`[0,1]`** only, reverse‑Z, or non‑linear perspective depth); call those out later instead
/// of changing this contract silently.
pub fn ndc_z_to_linear01(ndc_z: f32) -> f32 {
    (ndc_z + 1.0) / 2.0
}

/// Rounds **floating framebuffer coordinates** (outputs of [`ndc_to_framebuffer_px`]) to **unsigned
/// integer pixel indices**.
///
/// Each component uses **`f32::round`**: ties **half-way from zero** (**`±10.5` → `±11`** as `f32`;
/// **`10.5` → `11`** unsigned). Typical raster inputs from these helpers are **`≥ 0`**; callers that
/// may feed negatives should clamp separately (not covered here).
pub fn quantize_pixel_xy(px: Vec2) -> UVec2 {
    UVec2::new(px.x.round() as u32, px.y.round() as u32)
}

/// Clamps **`px`** interpreted as **`ndc_to_framebuffer_px`‑style** coordinates (**origin top-left**, **`+y` down**)
/// onto the inclusive pixel span **`0 … width − 1`** and **`0 … height − 1`**. Useful when primitives
/// leave the **`[-1,1]²`** NDC square but you still want **nearest in-bounds pixels** (`skip`-style
/// raster per project notes) rather than bogus casts.
///
/// Call **before [`quantize_pixel_xy`]** if **`px`** might be negative or past the outer column/row.
pub fn clamp_float_pixel_xy(px: Vec2, width: u32, height: u32) -> Vec2 {
    Vec2::new(
        px.x.clamp(0.0, (width - 1) as f32),
        px.y.clamp(0.0, (height - 1) as f32),
    )
}

/// Packs **`t ∈ [0, 1]`** (typically [`ndc_z_to_linear01`]) into **`16`‑bit normalized depth**.
///
/// **Saturates** **`t`** to **`[0, 1]`** first. Scale is **`65535`** so near/far corners still use the
/// full **`u16` range**: **`t = 1 → 65535`**, **`t = 0 → 0`**. Rounding follows **`f32::round`** on the
/// product **`t × 65535`** ( **`0.5` → round half away from zero** → midpoint lands on **`32768`** ).
pub fn linear01_to_depth_u16(t: f32) -> u16 {
    (t.clamp(0.0, 1.0) * 65535.0).round() as u16
}

/// Decodes **`u16`** depths from **[`linear01_to_depth_u16`]**: **`encoded as f32 / 65535.0`** in **`[0, 1]`**.
/// Exact round-trip **`linear01`** → **`u16`** → **`f32`** only when the forward **`round`** lands on values
/// whose quotient reproduces **`t`** at test precision — otherwise expect **\(O(1/65535)\)** drift.
pub fn depth_u16_to_linear01(encoded: u16) -> f32 {
    encoded as f32 / 65535.0
}

/// Builds a **left-handed `World → View`** matrix (**`glam::Mat4::look_at_lh`**). Pass **`eye`**, **`camera_target`**, and
/// **`camera_up_world`** (**`Vec3::Y`** typically). View space: **`+X`** right, **`+Y`** up, **`+Z`** forward along
/// **`normalize(camera_target − eye)`** — matches **`glam`** and the project’s LH / **`+Y`** up / **`+Z`** forward notes.
pub fn view_matrix_look_at_lh(eye: Vec3, camera_target: Vec3) -> Mat4 {
    let _ = (eye, camera_target);
    Mat4::IDENTITY
}

/// **`Proj · View · p_world`** with **`View`** from **[`view_matrix_look_at_lh`]** (full rigid camera),
/// **`Proj`** from **[`orth_projection_matrix`**]. Still **`w = 1`** in the orthographic core.
pub fn orth_world_to_clip_look_at_lh(
    world_point: Vec3,
    eye: Vec3,
    camera_target: Vec3,
    vmin: Vec3,
    vmax: Vec3,
) -> Vec4 {
    let _ = (world_point, eye, camera_target, vmin, vmax);
    Vec4::ZERO
}

/// **`orth_projection` → framebuffer `UVec2`** for a **full-surface** drawable (`800×600`-style).
///
/// Composes **[`orth_projection`] (xy) → [`ndc_to_framebuffer_px`] → [`clamp_float_pixel_xy`] → [`quantize_pixel_xy`]**
/// in that order. Matches the recommended “safe raster” path once NDC overshoot is possible.
pub fn ortho_point_to_framebuffer_pixel(
    point_view: Vec3,
    vmin: Vec3,
    vmax: Vec3,
    width: u32,
    height: u32,
) -> UVec2 {
    let ndc = orth_projection(point_view, vmin, vmax);
    let fb_px = ndc_to_framebuffer_px(ndc.truncate(), width, height);
    let clamped_px = clamp_float_pixel_xy(fb_px, width, height);
    quantize_pixel_xy(clamped_px)
}

/// **`Proj · View · p_world`** when the **only** view change is camera **position** (**no rotation**):
/// **`View = Translate(−eye)`** so **view-space = world-space − `camera_position_world`**. Axes stay
/// aligned with the world **`+X` / `+Y` / `+Z`** frame (first step before a full “look-at” later).
///
/// Returns **`Vec4` (`x,y,z,w`)** with **`w = 1`**; **`.truncate()`** matches [`orth_projection`] on
/// **view-space** coordinates **(`world_point - camera_position_world`, `vmin`, `vmax`)**.
pub fn orth_world_to_clip_identity_basis(
    world_point: Vec3,
    camera_position_world: Vec3,
    vmin: Vec3,
    vmax: Vec3,
) -> Vec4 {
    let view = Mat4::from_translation(-camera_position_world);
    let proj = orth_projection_matrix(vmin, vmax);
    proj * view * world_point.extend(1.0)
}

/// World-space point → framebuffer **`UVec2`** with **identity-basis view** (**translation-only**, Case 13):
/// **`orth_projection` → `xy` → [`ndc_to_framebuffer_px`] → [`clamp_float_pixel_xy`] → [`quantize_pixel_xy`]**, after
/// moving **`world_point − eye`** through **clip**.
pub fn orth_world_to_framebuffer_pixel(
    world_point: Vec3,
    camera_position_world: Vec3,
    vmin: Vec3,
    vmax: Vec3,
    width: u32,
    height: u32,
) -> UVec2 {
    let ndc = orth_world_to_clip_identity_basis(world_point, camera_position_world, vmin, vmax);
    let fb_px = ndc_to_framebuffer_px(ndc.truncate().truncate(), width, height);
    let clamped_px = clamp_float_pixel_xy(fb_px, width, height);
    quantize_pixel_xy(clamped_px)
}
