# Project breakdown

This document describes how I plan to approach the project iteratively.

## Iterations

### [x] Base WebP (still)

- **Goal:** Learn to produce a valid **lossless WebP** from raw **RGB** pixel data (browser-displayable artifact).
- **Outcome:** An **800×600** **still `.webp`** with a **single white pixel** at the center (sanity-check stride, origin corner, and **`webp-animation`** / libwebp encode path).

### [ ] Drawing lines

- **Goal:** Implement the simplest practical line rasterization (**DDA-style** stepping).
- **Outcome:** An **800×600** **still `.webp`** showing a **crossed square** (two rectangles sharing edges—or another simple 2D line exercise you can eyeball for gaps/overdraw).

### [ ] Cube: orthographic projection

- **Goal:** Learn the math for **orthographic** projection and viewport mapping (**Unity-ish LH / +Y up / +Z forward** in world terms; clip/screen mapping stays pragmatic until the wgpu checkpoint).
- **Outcome:** A **single still** **800×600** **`.webp`**: **wireframe** cube, **all twelve edges**, **fixed camera**, **one chosen model orientation** (rotate the cube “just enough” in that frame so it reads as a cube in 3D). Triangle soup is fine. No time loop yet—this milestone is projection correctness, not animation plumbing.

### [ ] Cube: animation — rotating wireframe (orthographic)

- **Goal:** Introduce **multi-frame / time**: drive **model orientation** that changes every frame—rotation around **all three axes** (implementation choice: **per-axis Euler** with documented order + gimbal caveat, or **quaternion + axis-angle** if you prefer fewer traps).
- **Outcome:** One **lossless animated `.webp`** (**800×600**) showing the **orthographic wireframe cube** rotating smoothly over \(N\) frames (pick \(N\), frame timestamps in ms, and loop convention once and reuse). Encode with **`webp-animation`** using **`EncodingType::Lossless`**. This establishes your **animation scaffold** early; later milestones **reuse** it (swap projection, filled raster, shading—same loop).

### [ ] Cube: perspective projection

- **Goal:** Switch projection to **perspective** (homogeneous divide, guardrails for \(w\) / behind-camera junk).
- **Outcome:** **Still frame first:** same cube/orientation style as orthographic still milestone, but **perspective** → **still `.webp`**. Then **reuse the animation loop** from the previous step: **animated `.webp`** of **perspective wireframe** rotation (same axis/time policy as orthographic animation).

### [ ] Cube: filled raster + depth buffer

- **Goal:** Implement **filled triangle rasterization** (pick **half-space/barycentric** _or_ scanlines as your first serious implementation—don’t maintain both long-term) plus a **depth buffer**.
- **Outcome:** A **solid cube** using a **constant fragment color** (no lighting yet). Occlusion must look correct from your fixed viewpoint. Prefer at least one **short animated `.webp`** (rotating cube) so motion reveals depth-buffer bugs that a single still hides.

### [ ] Cube: back-face culling

- **Goal:** Learn **triangle-level back-face culling** using consistent **winding + face normal / view direction** tests.
- **Outcome:** Same filled constant-color cube, but **back-facing triangles are discarded before raster** (fewer fills; should match the depth-buffer result when the mesh is closed).

### [ ] Cube: basic shading

- **Goal:** Learn **faceted shading** math (per-face normals duplicated at vertices, or equivalent).
- **Outcome:** Filled cube with **simple shading** (e.g. diffuse term) so faces read as distinct planes.

### [ ] Sphere: more complex shape

- **Goal:** Generate a **procedural sphere**; move toward an **indexed mesh** where it pays off (shared vertices vs triangle soup).
- **Outcome:** **Faceted** shaded sphere (low tessellation should still read “polyhedral”).

### [ ] Sphere: smooth shading

- **Goal:** Learn **smooth shading** via **interpolated normals** (and renormalizing per fragment if you go that route).
- **Outcome:** Same sphere mesh with **smooth** shading.

### [ ] Torus: wireframe — back-face–aware edges (A)

- **Goal:** Handle **torus** topology with the same line rasterizer; classify edges using adjacent triangle facing relative to the camera (**Option A**).
- **Outcome:** **Wireframe torus** that draws **silhouette edges** plus edges on the visible (front-facing) side of the mesh, and **drops edges that belong only to back-facing triangles**. Accept that **front-facing self-occlusion** may still look “see-through” (lines visible through the tube/hole where a full hidden-line treatment would remove them). Still **restricted scenes** unless you opt into extra clipping.

### [ ] Torus: wireframe — hidden lines / occlusion (B, optional)

- **Goal:** Improve wireframe realism where **A** is insufficient: suppress segments that are **front-facing** but **occluded** by other parts of the **same** torus (**Option B**).
- **Outcome:** Cleaner wireframe from viewpoints where the hole/tube overlap used to show false edges—implementation choice left open (e.g. depth-buffer tests along edges, mesh/object-space hidden-line approaches). **Skip this milestone** if you prefer to move on once **A** looks good enough.

### [ ] Torus: filled + smooth shading

- **Goal:** Apply the same filled pipeline as the cube/sphere to the torus.
- **Outcome:** **Smooth-shaded filled torus** (your CPU rasterizer capstone before GPU).

### [ ] Phase 2 — GPU / wgpu (to elaborate)

- **Goal:** Port the proven pipeline concepts to **wgpu** on **macOS** (Metal backend).
- **Outcome (draft checklist):** swapchain/surface, buffers, pipeline state, vertex/fragment shaders, depth test, culling state aligned with a deliberate **wgpu convention checkpoint** (depth range, winding, NDC vs framebuffer).

---

## Notes / deferred

- **Golden image regression tests:** add when eyeballing saturates — decode `.webp` to RGB and compare, or compare raw framebuffer bytes **before** encode (still vs animated).
- **Live window** (`winit` + framebuffer blit): optional after disk-export workflow is boring—pairs naturally with animation (**real-time** rotation instead of writing WebPs).
- **PNG / ffmpeg:** optional escape hatches for tooling compatibility or pixel-diff tooling that prefers PNG—**not** the default deliverable.
