# Cube orthographic wireframe — implementation report

This document summarizes what was implemented for the **Cube: orthographic projection** milestone and the technical decisions behind it.

---

## Deliverable

The `still-cube` binary renders a **white wireframe cube** on a **black** **800×600** RGB framebuffer and exports a **lossless** single-frame **`.webp`**, same CLI contract as before (optional output path argument; default `scene.webp`).

---

## Dependencies

- **`glam` (0.32.x)** — `Mat4`, `Vec3`, and built-in **`Mat4::look_at_lh`**, **`Mat4::orthographic_lh`**, plus **`Mat4::from_rotation_y`**. Matches the project spec’s baseline for linear algebra.

---

## New / changed modules

| Piece | Role |
|--------|------|
| **`src/vertex.rs`** | Minimal **`Vertex { position: Vec3 }`** — the raster/export boundary type; more attributes arrive only when later milestones need them. |
| **`src/cube.rs`** | **Indexed mesh:** **8** positions **`VERTICES`** on **[-1, 1]³**; **`TRIANGLE_INDICES`** — **12** triangles (two per face). **`triangles()`** yields **`[Vertex; 3]`** per triangle (triangle soup with copied positions). **`wireframe_edge_indices()`** builds unique undirected edges from all triangle edges, then **keeps only axis-aligned pairs in model space** (the **12** hull edges), dropping the **six face diagonals** from triangulation (**18** unique edges before filter). |
| **`src/transform.rs`** | **`OrthoScene`**: **MVP** = **`P · V · M`**, maps model positions to **pixel-space floats** via clip/NDC and a **viewport** (documented in-file). |
| **`src/main.rs`** | **`assert_eq!(cube::triangles().count(), …)`** keeps the triangle stream and **`Vertex`** on the execution path (and satisfies strict **`clippy -D warnings`**). **`draw_wireframe_cube`** iterates **`wireframe_edge_indices()`** and **`draw_line_f`**. |
| **`src/framebuffer.rs`** | Public line raster entrypoint **`draw_line_f`** with **fractional** endpoints and **skip-only** clipping via **`set_pixel`**. |

---

## Geometry vs wireframe (alignment with the project spec)

- **Filled pipeline later** should consume **`triangles()`** as a stream of **`[Vertex; 3]`** (or equivalent), matching the spec’s triangle-first boundary.
- **Wireframe** does **not** maintain a hand-written **12-edge** list. It **derives** segments from the same **`TRIANGLE_INDICES`**: after collecting **unique** index pairs, a **model-space** test keeps endpoints that differ in **exactly one** coordinate (valid for this **axis-aligned unit** cube in model space). That removes triangulation diagonals; **scaled**, **skewed**, or **non-convex** meshes would need a different rule.

---

## Conventions and matrix choices

- **World / view:** Left-handed, **+Y up**, **+Z forward**, via **`Mat4::look_at_lh`**.
- **Projection:** **`Mat4::orthographic_lh`** — **NDC x/y** roughly **[-1, 1]**, depth **[0, 1]** (D3D / WebGPU style) for early alignment with a future **wgpu** checkpoint.
- **Viewport:** **NDC x** → horizontal pixels with **(ndc_x + 1) / 2**; **flip NDC y** so **+y** up matches **row 0 = top** of the framebuffer.

---

## Camera, frustum, and model pose

- **Camera:** **`look_at_lh`** from **`eye = (2.7, 2.1, −4.2)`** toward the origin, **up = (0, 1, 0)**.
- **Orthographic frustum:** Symmetric **`±2.6`** on **x** and **y**, **`near = 0.5`**, **`far = 25.0`**.

- **Model matrix:** **`R_y(20°)`** only — **`Mat4::from_rotation_y(20.0_f32.to_radians())`**. Single-axis rotation keeps the milestone readable and leaves composite Euler order out of this step.

---

## Rasterization notes

- **Projection** returns **`(f64, f64)`** in pixel space so segment sampling stays stable when endpoints map outside the image.
- **`draw_line_f`** uses **`steps = ⌈max(|Δx|, |Δy|)⌉`**.

---

## Testing

- **Framebuffer** ascii-art tests (via **`draw_line_f`**).
- **`cube` module:** triangle count, **12** hull edges after filtering, **18** unique edges before filtering (twelve hull + six face diagonals).
- Integration test: run the binary and decode the output WebP with **`webp-animation`**.

---

## Follow-ups (not done here)

- Mark the **Cube: orthographic projection** checkbox in **`project-breakdown.md`** when you are satisfied with the visual.
- Optional: **golden-image** regression once the reference frame is frozen.
- Next milestone: **animated orthographic wireframe** — reuse **`OrthoScene`**, animate **M**, **`webp-animation`** timestamps.
- When **non-uniform** scaling or **arbitrary** meshes appear, replace the **axis-aligned edge** filter with topology-aware edge classification or an explicit edge buffer if needed.
