# Cube (orthographic wireframe) — implementation breakdown

Task-size guide for the milestone **Cube: orthographic projection** in `project-breakdown.md`: a single **800×600** lossless **still `.webp`** of a **wireframe cube** (all **12 edges**), fixed camera, one chosen model orientation, **no** animation yet.

---

## 1. Lock conventions before coding matrices

Document briefly (comments are enough for this stage):

- **World:** left-handed, **+Y up**, **+Z forward** (per project spec).
- **Camera:** eye position, look-at target, up vector → **view** matrix (inverse of camera pose).
- **Clip / NDC:** the numeric range after orthographic projection (e.g. **x**, **y** in a chosen normalized interval).
- **Framebuffer mapping:** how **NDC y** maps to **screen row y** (top-left vs bottom-left origin). The current framebuffer uses **row-major** **y** downward; if NDC **y** is “up,” apply an explicit **flip in the viewport** step.
- **Model:** a single orientation (e.g. one rotation). If using **Euler** angles, fix **rotation order** and note the **gimbal** caveat; **quaternion + axis-angle** is an alternative.

This reduces mirrored or upside-down surprises across later milestones.

---

## 2. Add `glam` and a thin math layer

Per `project-spec.md`, use **`glam`** for `Vec3` / `Mat4`. Add the dependency, then keep helpers small (e.g. `transform.rs`, `camera.rs`, or equivalent modules):

- **`model_matrix`** — rotation (and uniform scale 1 if needed).
- **`view_matrix`** — from eye, target, up.
- **`orthographic_matrix`** — `left` / `right` / `bottom` / `top` / `near` / `far` chosen so the **oriented** cube sits inside the box by construction (restricted scene; tune by hand).
- **`viewport`** — maps projected **x**, **y** to **u32** pixel coordinates in `0..800`, `0..600`.

Compose **`P * V * M`** (verify **`glam`** column-vector convention once). Use homogeneous **`Vec4`** end-to-end so **w** is explicit; orthographic **w** is often **1**, which keeps the path aligned with a later **perspective** divide.

---

## 3. Cube geometry: edge list

Deliverable is **12 edges**, not filled triangles.

- **8 vertices** of a unit cube in **model space** (e.g. centered at origin; document edge length).
- **12 undirected edges** as vertex index pairs `(i, j)`; draw each segment once.

An explicit edge list matches the milestone and stays independent of “triangle soup” plumbing used later.

---

## 4. Rendering loop (reuse existing line rasterization)

No new raster core:

1. Optionally clear the framebuffer to black.
2. For each edge `(a, b)`:
   - Transform vertices **a** and **b** with **`P * V * M`** → **clip/NDC** **x**, **y** (and **z** optional for future depth; not required for this still if there is no depth test).
   - Apply **viewport** → `Point(x0, y0)`, `Point(x1, y1)` in framebuffer space.
   - Call existing **`FrameBuffer::draw_line`** with the chosen wire color (e.g. white).

Out-of-bounds behavior stays **skip-only** (`set_pixel` guards); **no line clipping** yet (per spec).

---

## 5. Export path

Keep the current **`WebpEncoder`** workflow: **800×600** RGB buffer → **lossless** still `.webp`. Only the scene drawing function changes relative to earlier milestones.

---

## 6. Done criteria (sanity checks)

- **12** segments are drawn (or fix projection if one degenerates).
- **Aspect:** ortho frustum + **800×600** viewport do not visibly squash the cube.
- **Readability:** one model orientation so the cube clearly reads as **3D** (not a flat silhouette).

---

## Suggested module map (optional)

| Area | Responsibility |
|------|----------------|
| `Cargo.toml` | Add **`glam`** |
| Cube data | Vertex table + **12** edge index pairs (`mesh.rs` or local to the binary) |
| Math | View, ortho, viewport (`matrices.rs` / `camera.rs` or similar) |
| Raster / IO | Unchanged **`framebuffer.rs`**, **`webp_encoder.rs`**; thin **`main.rs`** orchestration |

---

## Order of work

1. Conventions + dependency + **viewport** behavior nailed down.  
2. **`glam`** + view / ortho / model + viewport helpers.  
3. Cube vertices + edges; transform endpoints; **`draw_line`**.  
4. Encode still WebP; eyeball against the milestone description.

Later milestones (animated orthographic wireframe, perspective, fills) **reuse** this scaffold—keep matrix and viewport code structured accordingly.
