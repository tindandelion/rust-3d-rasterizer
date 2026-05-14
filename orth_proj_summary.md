# Orthographic projection milestone — summary

Pair-programmed, TDD-driven derivation anchored in **`tests/ortho_projection.rs`** and **`src/projection.rs`**. Coordinates match the repo planning docs: Unity-style intuition **left-handed**, **+Y up**, **+Z forward** where relevant; **`glam`** for vectors and **`Mat4`**.

## Roles

We are working in a pair programming session. Your role is a senior developer who teaches me how to derive the implementation for the orthographic projection from the ground up. We do it in a case-by-case manner, from the simplest cases to the more elaborate ones. You will **never** write the implementation: that's my job as a student. You can write and modify the tests, but I will update the implementation.

## Global assumptions baked into the tests

- **Full framebuffer**: NDC **`xy`** maps over the entire **`width × height`** surface (pixel indices **`0 … width−1`**, **`0 … height−1`**). Older **inset viewport** cases were removed from the suite and API on purpose.
- **Column vectors**: homogeneous transforms use **`matrix * Vec4`** in **`glam`**’s usual layout.
- **Pixel grid**: framebuffer origin **`(0, 0)`** is **top-left**, **+x** right, **+y** **down** (bitmap style). **NDC +y** is **up**, so framebuffer **y** uses an explicit flip when mapping **`[-1, 1]`** to rows.

---

## Cases still covered by integration tests

| Case   | Idea                                                                                                              | Primary API exercised                   |
| ------ | ----------------------------------------------------------------------------------------------------------------- | --------------------------------------- |
| **1**  | Symmetric **`[-1,1]³`** volume → **`orth_projection`** is identity.                                               | **`orth_projection`**                   |
| **2**  | Symmetric slabs **`vmin = −h`**, **`vmax = +h`** per axis → NDC **`x,y,z`**.                                      | **`orth_projection`**                   |
| **3**  | Arbitrary axis-aligned **`[vmin,vmax]`** corners → **`−1/+1`** on endpoints.                                      | **`orth_projection`**                   |
| **4**  | Same map as **`orth_projection`** in a **`Proj`** **`Mat4`**: **`Proj * Vec4(p,1)`** vs scalar path, **`w = 1`**. | **`orth_projection_matrix`**            |
| **5**  | **NDC `xy`** → float pixel coords; **y** flipped for top-left raster.                                             | **`ndc_to_framebuffer_px`**             |
| **7**  | **NDC z** **`[-1,1]` → linear depth **`[0,1]`\*\* (`(z+1)/2`).                                                    | **`ndc_z_to_linear01`**                 |
| **8**  | Float pixels → **`UVec2`** via **`f32::round`**; **`f32`** literal gotchas noted in comments.                     | **`quantize_pixel_xy`**                 |
| **9**  | Clamp float pixels to valid index range before rounding (NDC overshoot).                                          | **`clamp_float_pixel_xy`**              |
| **10** | Pack **`linear01`** into **`u16`** (`65535` scale, saturate, round).                                              | **`linear01_to_depth_u16`**             |
| **11** | View-space point → **full** pixel pipeline: ortho → NDC xy → FB float → clamp → quantize.                         | **`ortho_point_to_framebuffer_pixel`**  |
| **13** | **World → clip** with **translation-only view**: **`View = Translate(−eye)`**, then **`Proj`**.                   | **`orth_world_to_clip_identity_basis`** |
| **14** | **World → `UVec2`** with same camera (translate-only): equivalent to Case 11 on **`world − eye`**.                | **`orth_world_to_framebuffer_pixel`**   |
| **16** | Decode **`u16`** depth back to **`linear01`** (`/ 65535`), approximate round-trip with Case 10.                   | **`depth_u16_to_linear01`**             |

**Removed / skipped numbering:** cases **6**, **12**, **15** (viewport sub-rectangles) were deleted when the project standardized on a full-frame drawable. **Case 17** (look-at view + **`Proj`**) was removed for now; it can be reintroduced as tests-only when you resume that thread.

---

## Public helpers in `src/projection.rs` (current)

Rough pipeline order:

1. **View volume → NDC (per axis `[-1,1]`)**  
   **`orth_projection`**, **`orth_projection_matrix`**
2. **NDC → raster**  
   **`ndc_to_framebuffer_px`**, **`clamp_float_pixel_xy`**, **`quantize_pixel_xy`**
3. **Depth sidecar**  
   **`ndc_z_to_linear01`**, **`linear01_to_depth_u16`**, **`depth_u16_to_linear01`**
4. **Compositions**  
   **`ortho_point_to_framebuffer_pixel`**, **`orth_world_to_clip_identity_basis`**, **`orth_world_to_framebuffer_pixel`**

**Work in progress:** **`view_matrix_look_at_lh`** and **`orth_world_to_clip_look_at_lh`** remain in **`projection.rs`** as placeholders (nothing in **`ortho_projection.rs`** calls them after Case **17** was dropped). When you add tests again, compose them like Case **13** with **`glam::Mat4::look_at_lh(eye, target, up)`** (and align the function signatures with **`up`** plus the rustdoc on those items).

---

## How to extend

- **`cargo test --test ortho_projection`** — single integration harness for everything above.
- Next natural additions: restoration of **look-at** tests only (you implement), then **`world + look_at → framebuffer`** mirroring Case **14**, or **model matrix** (**`Proj * View * Model`**) when vertices are authored in model space.
