# Detailed plan: learning 3D rasterization (CPU → GPU)

Personal learning project: a simple **3D renderer** using **rasterization**, implemented in **Rust end-to-end**, developed **primarily on macOS**.

---

## Phasing

| Phase | Focus |
|-------|--------|
| **1** | Math and algorithms on the CPU (software rasterization path). **Shipped:** orthographic camera, indexed meshes, Phong shading, depth buffer, export bins. |
| **2** | **Rendering pipeline** on CPU — decoupled **materials**, **lights**, and **scene clear color**; smooth Phong retained; current **`meshes::torus`** API unchanged. Export-first (**WebP** / Kitty); live **`winit`** viewer deferred. |
| **3** | Hardware acceleration using **`wgpu`** (Metal backend on Mac). **Perspective** lands in the GPU pipeline; **no** CPU-perspective prerequisite. |

Optional stretch beyond these phases is allowed (e.g. CPU perspective in Phase 2, richer GPU work, live viewer) if motivation persists.

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
- **`crossterm`** — terminal window size, raw mode, alternate screen (**`still-scene`** Kitty presentation).
- **`base64`** — Kitty graphics protocol payload encoding (**`still-scene`**).
- Optional: **`image`** with WebP decode feature **only if** you want convenience decoding in tests or tooling—not required for encoding if you compare pre-encode buffers.

---

## Coordinate conventions

- **Unity-style world/camera intuition:** left-handed, **+Y up**, **+Z forward** (camera/object relationships aligned with Unity thinking).
- **Clip space / framebuffer mapping:** **simplest pragmatic mapping first** on the CPU rasterizer. A deliberate **“wgpu alignment checkpoint”** is deferred until **Phase 3** prep (depth range, winding, NDC vs row-major Y, etc.).
- Keep **wgpu/Vulkan-style conventions** in mind for later so **Phase 3** is mostly “same ideas, different executor,” but **do not front-load** full parity in phase 1.

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
- **Lighting (current):** **`DirectionalLight`** (**`direction`** only for now) with explicit **`Material`** (**`diffuse`**, **`emissive`**, **`specular`**, **`shininess`**). **`Shape::render`** passes constant **`toward_eye = −Camera::direction()`** under orthographic projection.
- **Lighting (Phase 2 target):** decouple **lights** from **materials** in two steps. **First milestone:** per-shape **`Material`** with geometry-browser colors — diffuse **`0x156289`**, emissive **`0x072534`**, specular **`0x111111`**, **`shininess` 30** — on **`Shape`**; **`DirectionalLight`** (**`direction`** + **`intensity`**, white only); **`Shape::render(fb, camera, &light)`** with single-light Phong; **`FrameBuffer::clear(Rgb(68, 68, 68))`**. **Second milestone:** **`Shape::render`** takes **`&[DirectionalLight]`** and sums contributions; export bins add the browser’s three directionals (**`intensity` 3**). **Optional stretch (after perspective):** **positional** lights (point lights with per-fragment **`toward_light`**). **Emissive** added once per fragment throughout. **`Scene`** type deferred until multi-light bin wiring earns it.
- **Historical note:** **Gouraud** intensity interpolation was an intermediate milestone; only **Phong** remains in code.

Phase 1 milestone ambition (from earlier discussion): interpolated vertex attributes (**level 3**) before treating phase 1 as complete. **Phase 2** adds explicit material/light colors; **perspective-correct texturing (**level 4**)** remains optional stretch.

---

## Geometry ↔ rasterizer boundary

- **Current pipeline:** **`geometry::Mesh`** stores indexed **`Facet`**s + **`Vec3`** positions. **`Mesh::visible_triangles(view_direction)`** culls back faces and yields world-space **`Triangle`** records (corners + per-vertex **`UnitVec3`** normals). Scene-level **`Shape`** (**`mesh` + `color`**, **`src/lib.rs`**) projects each **`Triangle`** via **`Camera::transform`** → **`FbPixel`** (pixel **`xy`** + view-space **`depth`**) and draws with **`PhongShadedTriangle`**. **`meshes::{cube,dodecahedron,sphere,torus}`** are procedural builders on that stack.
- **Depth:** **`FrameBuffer::write_pixel`** keeps the nearer fragment (**smaller view-space **`z`**).
- **Phase 2 gaps (vs [Three.js TorusGeometry browser](https://threejs.org/docs/scenes/geometry-browser.html#TorusGeometry) reference):** export bins still use approximate colors (diffuse/specular **`Rgb(44, 94, 179)`**, emissive **`Rgb(8, 17, 32)`**, **`shininess` 100** vs reference diffuse **`0x156289`**, emissive **`0x072534`**, specular **`0x111111`**, **`shininess` 30**); **`DirectionalLight`** lacks **`intensity`**; black clear only (reference **`0x444444`**); single light (reference: three white directionals at **`intensity` 3**). **Out of Phase 2 scope (by choice):** wireframe overlay, flat shading, extended torus API (**`radius`**, **`tube`**, **`arc`**), live **`winit`** viewer / GUI.
- **No separate `Vertex` type:** positions live in **`Mesh`**; normals live on **`Facet`**; surface tint is **`Shape::color`** scaled by lighting intensity (Phase 2 moves full **`Material`** onto **`Shape`**).

---

## Rasterization strategy (phase 1)

### Current filled path

- **`PhongShadedTriangle`** (**`src/framebuffer/phong_shaded_triangle.rs`**): y-sorted scanlines, edge walking with **`Interpolator`**, per-pixel normal interpolation (**`NormalInterpolator`**), depth interpolation, and **`FrameBuffer::write_pixel`** (depth test + RGB write).
- **`Shape::render`**: one **`PhongShadedTriangle::draw`** per visible **`Triangle`**; lighting via **`Material::shade`** with **`DirectionalLight`** (closure passed into **`draw`**).
- **Out-of-bounds:** **`write_pixel`** ignores coordinates outside the framebuffer; **`Camera::transform`** does not clip — negative projected **`xy`** may wrap when cast to **`u32`** (documented in **`ortho_camera`** module docs).

### Historical milestones (retired code paths)

Earlier phase-1 work exercised **wireframe**, **DDA lines**, **quad fill**, **`draw_facets`**, **`ScanlineFillTriangle`**, **`GouraudShadedTriangle`**, and a **half-space** alternate. None of those remain in **`src/`**; see **`project-breakdown.md`** completed items for the iteration history.

### Threading

- **Single-threaded** CPU raster until correctness is solid (parallelism deferred).

---

## Projection & camera

- **Orthographic (shipped):** **`ortho_camera::Camera`** — **`for_viewport`** / **`move_to`**, look-at toward **`Vec3::ZERO`**, world **+Y** up, **`transform`** → **`FbPixel`** with view-space **`depth`**. Export bins and integration tests use this path through **Phase 2**.
- **Perspective (optional Phase 2 stretch):** homogeneous **`w`**, divide, near/far guardrails, perspective-correct depth interpolation, per-fragment **`toward_eye`**. **Not** a Phase 3 prerequisite — if skipped on CPU, perspective lands in **`wgpu`** (**Phase 3**). See **`Perspective projection (CPU)`** in **`project-breakdown.md`**.

---

## Output & debugging

- **Framebuffer:** **RGB** only (three `u8` channels per pixel); **no alpha**; per-pixel depth buffer. **`clear`** currently zeroes to black — **Phase 2** adds explicit **`clear(Rgb)`** for scene background. Default export size **`SCENE_WIDTH` × `SCENE_HEIGHT`** (**800×600**) — used by **`animated-scene`** and library constants in **`lib.rs`**. **`still-scene`** sizes the framebuffer to **terminal pixel dimensions** (see below).
- **WebP-first (lossless), browser-friendly:** **`animated-scene`** — **`ANIMATED_SCENE_FRAME_COUNT`** frames at **`ANIMATED_SCENE_FRAME_SPACING_MS`** spacing (default **`scene.webp`**, argv-overridable path). **`still-scene`** — writes **`still-scene.webp`** on a background thread at the same resolution as the terminal render (lossless single frame).
- **Terminal display (`still-scene`):** after rasterizing, blit the framebuffer to a **Kitty-compatible** terminal via the **graphics protocol** (**24-bit RGB**, centered, alternate screen + raw mode); dismiss on any keypress. Implementation: **`src/bin/still-scene/kitty_terminal.rs`** (**`crossterm`** + **`base64`**). Requires a terminal that reports pixel dimensions and supports Kitty graphics (e.g. **Kitty**, **Ghostty**, **iTerm2** with graphics enabled).
- **Tests:** in-crate unit tests (ASCII-art raster regressions, camera, lighting, meshes); integration **`tests/draw_unit_cube.rs`** (cube occlusion via **`Shape::render`**); **`tests/animated_scene_writes_frames.rs`** (spawn **`animated-scene`** binary).
- **Golden image regression tests:** intentionally **deferred** for WebP pixel diffs; when added, compare decoded RGB or raw framebuffer bytes **before** encode—animated tests are heavier than stills.
- **Live window** (`winit` + framebuffer blit): **deferred past Phase 2** — export bins remain the primary presentation path; a real-time viewer pairs naturally with **`animated-scene`** timing when appetite returns.

---

## Platform (phase 3)

- **Mac-only** explicit requirement for **`wgpu`** phase; cross-platform can wait until the renderer core is boring.

---

## Deferred checkpoints (do not lose track)

1. **Phase 2 — material (single light)** — explicit **`Material`** on **`Shape`** (diffuse **`0x156289`**, emissive **`0x072534`**, specular **`0x111111`**, **`shininess` 30**); **`DirectionalLight`** **`intensity`** (white only); **`FrameBuffer::clear(Rgb(68, 68, 68))`**; export bins with **one** light.
2. **Phase 2 — multi-light** — **`Shape::render(&[DirectionalLight])`** summation; three white directionals at **`intensity` 3** (geometry-browser positions).
3. **Phase 2 stretch — perspective projection (CPU)** — optional; only if appetite after materials/lights.
4. **Phase 2 stretch — positional lights** — optional; point lights with per-fragment **`toward_light`**; after perspective stretch (or after multi-light if perspective skipped).
5. **Phase 3 — wgpu** — swapchain/surface, buffers, pipeline state, shaders, depth test, culling; **wgpu convention alignment** (depth range, winding, NDC vs framebuffer); **perspective in GPU** if not done on CPU.
6. **Golden image / pixel-diff tests** — adopt when eyeballing saturates (WebP decode path or pre-encode buffer compare).
7. **Cross-platform** — revisit when/if portability becomes a goal.
8. **Live `winit` viewer** — interactive geometry-browser-style presentation; deferred past Phase 2.

---

## Original overview reference

Source intent from early notes: a **simple 3D rendering engine** using **rasterization** as the core technique, structured as a **learning project**.
