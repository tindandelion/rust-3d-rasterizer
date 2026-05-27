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

- **Rust** for both phases.
- **Single Cargo crate** initially (modules like `raster`, `mesh`, `webp_io`, …). Split into a workspace only when pain appears (e.g. separate binaries for export vs live window).

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
- **Long-term visual target:** fixed viewpoint, single shape — a **torus**.
- **Mesh progression:**
  1. **Cube (interim)** — **quad faces** only for the **first** filled-cube milestone (**six** convex quads; **4-vertex** faces). Wireframe can stay edge-based. This is a **deliberate shortcut**: **one** simple **bbox + inner test** raster path before general **triangle** fill.
  2. **Dodecahedron + cube triangles** — **refactor **`scene/cube`** to **`[Vertex; 3]`** submit (**two triangles per face**—raster **`FillTriangle`** already in use). Showcase the **triangle stack** on a **regular dodecahedron** (**twelve** pentagonal faces → **triangles**, e.g. **five** tris per face via centroid fan)—a **better demo** shape **and** it **forces** triangles (pentagons are not convex quads in 3D facet form).
  3. **Sphere** — on the unified stack, **procedural tessellation**, **indexed** structure where worthwhile.
  4. **Torus** — capstone CPU mesh complexity before/at GPU transition (**triangle** soup or indexed tris).
- **Generation:** **procedural** meshes (no asset pipeline required early).

---

## Shading progression

- **Cube:** faceted first.
- **Dodecahedron:** faceted (**pentagonal** faces shaded as planar facets; triangulation is a **submission** detail).
- **Sphere:** faceted first as well (low tessellation reads as a polyhedron).
- **Smooth shading:** implemented as an explicit **sub-step after** faceted shading works on the relevant shapes (do not mix concerns early).

Phase 1 milestone ambition (from earlier discussion): interpolated vertex attributes (**level 3**) before treating phase 1 as complete; **perspective-correct texturing (**level 4**)** remains optional stretch (“phase 4” feeling).

---

## Geometry ↔ rasterizer boundary

- **Phase 1 order:** **`[Vertex; 4]` quad stream** persists for **`scene/cube`** geometry until **`Dodecahedron`**, **while **`draw_facets`** already rasterizes with **two **`FillTriangle`** fills** per projected quad. **`Dodecahedron`** **refactors `scene/cube`** to **`[Vertex; 3]`** submit (**one** triangle raster type **for solids** onward). **`Sphere`** adds **another mesh** onto that stack. Internal storage may stay indexed; **unfold** at the raster boundary.
- **`Vertex` evolution:** start with **position only**; add normals, colors, UVs, etc., **only when a milestone requires them**.

---

## Rasterization strategy (phase 1)

### Order of milestones

1. **Wireframe** before filled surfaces.
2. **Lines:** simplest practical approach — **DDA-style** stepping (float increments acceptable).
3. **Out-of-bounds:** **skip-only** (`set_pixel` guarded); no full line clipping initially.
4. **Filled hull facets (`cube`, interim geometry — shipped):** after wireframe is understood, each **visible **`Quad`**** is rasterized via **two** **half-plane-tested **`FillTriangle`** passes** (bbox scan per triangle). **`Dodecahedron`** collapses **`scene/cube`** to **`[Vertex; 3]`** submit (**same raster**)—**no** second fill primitive thereafter.
5. **Filled triangles (steady state — `Dodecahedron`):** **commit** **one** tri algorithm (**half-space / barycentric** _or_ scanlines); **same milestone** switches **`scene/cube`** to **`[Vertex; 3]`** and introduces the **triangle** shaded **dodecahedron** (**pentagon facets triangulated**).

6. **Sphere (next mesh):** **procedural** sphere facets on the **existing** **`[Vertex; 3]`** stack; optionally refine **indexed** representation.

7. Early fills may use **flat colors per face**; fancier per-primitive/debug coloring was explicitly **not** required early.

**Breakdown alignment:** Hull **edges + lines + back-face classification** are already exercised (`doc/planning/project-breakdown.md`). The shipped **cube** milestones implemented **quad-stream** fills (**depth deferred**). **`Dodecahedron`** is the **triangle cutover** (**cube refactor + raster + dodecahedron demo**); **`Sphere`** grows **sphere geometry only**.

### Parallel raster approaches

- **Half-space / scanline** filled rasterization: **both remain mentally allowed**, but treat as **sequential experiments** — avoid maintaining two full pipelines forever. Prefer **one path to export parity** (RGB framebuffer → WebP), optionally second raster implementation behind a trait later.

### Threading

- **Single-threaded** CPU raster until correctness is solid (parallelism deferred).

---

## Projection & camera

- **Orthographic first** for the first stable **cube wireframe**.
- Switch to **perspective immediately after orthographic cube wireframe is stable** (before leaning on sphere/torus complexity for projection debugging).

---

## Output & debugging

- **Framebuffer:** fixed **`800×600`**, **RGB** only (three `u8` channels per pixel); **no alpha**.
- **WebP-first (lossless), browser-friendly:** **still `.webp`** milestones until animation scaffolding lands, then **animated `.webp`** for motion (single file per clip).
- **Golden image regression tests:** intentionally **deferred**; rely on **eyes-only** review **for a while**. When added, compare decoded RGB or raw framebuffer bytes **before** encode—animated tests are heavier than stills.
- **Live window** (`winit` + `pixels` / `softbuffer` or similar): **possible later**; same framebuffer, different presentation.

---

## Platform (phase 2)

- **Mac-only** explicit requirement for now; cross-platform can wait until the renderer core is boring.

---

## Deferred checkpoints (do not lose track)

1. **wgpu convention alignment pass** — depth range, front-face winding, NDC handedness, relationship to framebuffer row order vs Unity/world intuition.
2. **Golden image / pixel-diff tests** — adopt when eyeballing saturates (WebP decode path or pre-encode buffer compare).
3. **Filled-triangle algorithm choice** — commit when **`Dodecahedron`** lands (half-space/barycentric vs scanlines); **shipped interim cube quads** keep **bbox + inner test** until that **same** **`Dodecahedron`** milestone **retires** them.
4. **Cross-platform** — revisit when/if portability becomes a goal.

---

## Original overview reference

Source intent from early notes: a **simple 3D rendering engine** using **rasterization** as the core technique, structured as a **learning project**.
