# Detailed plan: learning 3D rasterization (CPU → GPU)

Personal learning project: a simple **3D renderer** using **rasterization**, implemented in **Rust end-to-end**, developed **primarily on macOS**.

---

## Phasing

| Phase | Focus |
|-------|--------|
| **1** | Math and algorithms on the CPU (software rasterization path). |
| **2** | Hardware acceleration using **`wgpu`** (Metal backend on Mac). |

Optional stretch beyond the original two phases is allowed (e.g. deeper CPU topics or richer GPU work) if motivation persists.

---

## Language & project layout

- **Rust** for both phases. Package name **`thorus-forge`** (see **`Cargo.toml`**).
- **Single Cargo crate** (modules: **`framebuffer`**, **`geometry`**, **`lighting`**, **`meshes`**, **`ortho_camera`**, **`webp_encoder`**; scene **`Shape`** and scene constants in **`lib.rs`**; export bins in **`src/bin/`**). Split into a workspace only when pain appears (e.g. separate binaries for export vs live window).

---

## Math & dependencies

- Use **`glam`** for vectors, matrices, and multiplication/storage.
- Implement projection/view logic with conventions **you document explicitly**; do not assume every `glam` helper matches your chosen spaces without verification.

### Baseline crates (phase 1 export path)

- **`glam`** — linear algebra.
- **`webp-animation`** — **lossless WebP** output: **still** (single-frame) first, then **animated** (same crate, `EncodingType::Lossless`). Depends on **libwebp** via sys crates (native toolchain required).
- Optional: **`image`** with WebP decode feature **only if** you want convenience decoding in tests or tooling—not required for encoding if you compare pre-encode buffers.

---

## Coordinate conventions

- **Unity-style world/camera intuition:** left-handed, **+Y up**, **+Z forward** (camera/object relationships aligned with Unity thinking).
- **Clip space / framebuffer mapping:** **simplest pragmatic mapping first** on the CPU rasterizer. A deliberate **“wgpu alignment checkpoint”** is deferred until phase 2 prep (depth range, winding, NDC vs row-major Y, etc.).
- Keep **wgpu/Vulkan-style conventions** in mind for later so phase 2 is mostly “same ideas, different executor,” but **do not front-load** full parity in phase 1.

---

## Scene scope & content progression

- **Restricted scenes:** no general frustum clipping homework in phase 1; geometry stays inside the volume by construction.
- **Long-term visual target:** fixed viewpoint, single shape — a **torus** (**shipped** in export bins).
- **Mesh progression (current code):**
  1. **Cube** — **`meshes::cube()`**: eight verts, twelve **`Facet`** wedges (two per hull quad), faceted normals.
  2. **Dodecahedron** — **`meshes::dodecahedron()`**: regular dodecahedron (**`three.js`** detail 0 tri list).
  3. **Sphere** — **`meshes::sphere(splits)`**: octahedron seed + edge-midpoint subdivision; **radial vertex normals** for smooth shading.
  4. **Torus** — **`meshes::torus(ring_segments, tube_segments)`**: indexed mesh with parametric smooth normals; **`still-scene`** and **`animated-scene`** export bins render **`torus(48, 32)`**.
- **Generation:** **procedural** meshes (no asset pipeline required early).
- **Historical note:** earlier milestones exercised **wireframe**, **quad fill**, **`draw_facets`**, and **Gouraud** shading; those code paths were retired. Completed milestone wording in **`project-breakdown.md`** preserves that history.

---

## Shading progression

- **Cube / dodecahedron:** faceted (**`Facet::with_facet_normal`** duplicates the facet normal at each corner).
- **Sphere / torus:** smooth vertex normals (**`Facet::with_vertex_normals`**); **Phong** raster (**`PhongShadedTriangle`**: interpolate normals per pixel, renormalize, shade).
- **Lighting (current):** **`BlinnLightModel`** with **`Material::matte`** / **`Material::shiny`** (Blinn–Phong specular). **`Shape::render`** passes constant **`toward_eye = −Camera::direction()`** under orthographic projection (revisit for perspective).
- **Historical note:** **Gouraud** intensity interpolation was an intermediate milestone; only **Phong** remains in code.

Phase 1 milestone ambition (from earlier discussion): interpolated vertex attributes (**level 3**) before treating phase 1 as complete; **perspective-correct texturing (**level 4**)** remains optional stretch (“phase 4” feeling).

---

## Geometry ↔ rasterizer boundary

- **Current pipeline:** **`geometry::Mesh`** stores indexed **`Facet`**s + **`Vec3`** positions. **`Mesh::visible_triangles(view_direction)`** culls back faces and yields world-space **`Triangle`** records (corners + per-vertex **`UnitVec3`** normals). Scene-level **`Shape`** (**`mesh` + `color`**, **`src/lib.rs`**) projects each **`Triangle`** via **`Camera::transform`** → **`FbPixel`** (pixel **`xy`** + view-space **`depth`**) and draws with **`PhongShadedTriangle`**. **`meshes::{cube,dodecahedron,sphere,torus}`** are procedural builders on that stack.
- **Depth:** **`FrameBuffer::write_pixel`** keeps the nearer fragment (**smaller view-space **`z`**).
- **No separate `Vertex` type:** positions live in **`Mesh`**; normals live on **`Facet`**; surface tint is **`Shape::color`** scaled by lighting intensity.

---

## Rasterization strategy (phase 1)

### Current filled path

- **`PhongShadedTriangle`** (**`src/framebuffer/phong_shaded_triangle.rs`**): y-sorted scanlines, edge walking with **`Interpolator`**, per-pixel normal interpolation (**`NormalInterpolator`**), depth interpolation, and **`FrameBuffer::write_pixel`** (depth test + RGB write).
- **`Shape::render`**: one **`PhongShadedTriangle::draw`** per visible **`Triangle`**; lighting via **`BlinnLightModel::calc_intensity`** (closure passed into **`draw`**).
- **Out-of-bounds:** **`write_pixel`** ignores coordinates outside the framebuffer; **`Camera::transform`** does not clip — negative projected **`xy`** may wrap when cast to **`u32`** (documented in **`ortho_camera`** module docs).

### Historical milestones (retired code paths)

Earlier phase-1 work exercised **wireframe**, **DDA lines**, **quad fill**, **`draw_facets`**, **`ScanlineFillTriangle`**, **`GouraudShadedTriangle`**, and a **half-space** alternate. None of those remain in **`src/`**; see **`project-breakdown.md`** completed items for the iteration history.

### Threading

- **Single-threaded** CPU raster until correctness is solid (parallelism deferred).

---

## Projection & camera

- **Orthographic (shipped):** **`ortho_camera::Camera`** — **`for_viewport`** / **`move_to`**, look-at toward **`Vec3::ZERO`**, world **+Y** up, **`transform`** → **`FbPixel`** with view-space **`depth`**. Export bins and integration tests use this path.
- **Perspective (next open milestone):** homogeneous **`w`**, divide, near/far guardrails, perspective-correct depth interpolation, per-fragment **`toward_eye`**. See **`Perspective projection (CPU)`** in **`project-breakdown.md`**.

---

## Output & debugging

- **Framebuffer:** **`SCENE_WIDTH` × `SCENE_HEIGHT`** (**800×600**), **RGB** only (three `u8` channels per pixel); **no alpha**; per-pixel depth buffer.
- **WebP-first (lossless), browser-friendly:** **`still-scene`** (single-frame **`still-scene.webp`**) and **`animated-scene`** (**`ANIMATED_SCENE_FRAME_COUNT`** frames, **`ANIMATED_SCENE_FRAME_SPACING_MS`** spacing, default **`scene.webp`**, argv-overridable path).
- **Tests:** in-crate unit tests (ASCII-art raster regressions, camera, lighting, meshes); integration **`tests/draw_unit_cube.rs`** (cube occlusion via **`Shape::render`**); **`tests/animated_scene_writes_frames.rs`** (spawn **`animated-scene`** binary).
- **Golden image regression tests:** intentionally **deferred** for WebP pixel diffs; when added, compare decoded RGB or raw framebuffer bytes **before** encode—animated tests are heavier than stills.
- **Live window** (`winit` + framebuffer blit): **possible later**; same framebuffer, different presentation.

---

## Platform (phase 2)

- **Mac-only** explicit requirement for now; cross-platform can wait until the renderer core is boring.

---

## Deferred checkpoints (do not lose track)

1. **Perspective projection (CPU)** — **`w`** divide, clip guardrails, depth model migration, per-fragment eye vector for specular.
2. **wgpu convention alignment pass** — depth range, front-face winding, NDC handedness, relationship to framebuffer row order vs Unity/world intuition.
3. **Golden image / pixel-diff tests** — adopt when eyeballing saturates (WebP decode path or pre-encode buffer compare).
4. **Cross-platform** — revisit when/if portability becomes a goal.

---

## Original overview reference

Source intent from early notes: a **simple 3D rendering engine** using **rasterization** as the core technique, structured as a **learning project**.
