# Project breakdown

This document describes how I plan to approach the project iteratively.

## Iterations

### [x] Base WebP (still)

- **Goal:** Learn to produce a valid **lossless WebP** from raw **RGB** pixel data (browser-displayable artifact).
- **Outcome:** An **800×600** **still `.webp`** with a **single white pixel** at the center (sanity-check stride, origin corner, and **`webp-animation`** / libwebp encode path).

### [x] Drawing lines

- **Goal:** Implement the simplest practical line rasterization (**DDA-style** stepping).
- **Outcome:** An **800×600** **still `.webp`** built from many **spokes**: radial segments from the scene center to a circle, so gaps, endpoint inclusivity, and **`set_pixel`** skipping are easy to eyeball. (A **crossed square**—two rectangles sharing edges—remains a fine alternative scene if you want a grid-aligned regression image later.)

### [x] Cube: orthographic projection

- **Goal:** Learn the math for **orthographic** projection and viewport mapping (**Unity-ish LH / +Y up / +Z forward** in world terms; clip/screen mapping stays pragmatic until the wgpu checkpoint).
- **Outcome:** A **single still** **800×600** **`.webp`**: **wireframe** cube, **all twelve edges**, **fixed camera**, **one chosen model orientation** (rotate the cube “just enough” in that frame so it reads as a cube in 3D). Triangle soup is fine. No time loop yet—this milestone is projection correctness, not animation plumbing.

### [x] Cube: animation — rotating wireframe (orthographic)

- **Goal:** Introduce **multi-frame / time**: drive **model orientation** that changes every frame.
- **Shipped:** A **three-axis Euler-style** tumble: **world-fixed** **`R_z R_y R_x`** with **α = β = γ = t**, **t** sweeping **0 … τ** over **`ANIMATED_CUBE_FRAME_COUNT`** frames (**seamless loop**). The animated mesh is **only** **0.5** uniform scale on the unit cube (**no** π/4 **X/Y tilt** from the still—that tilt stays on **`still-cube`** for a readable single-frame ortho snapshot). The library **`wireframe`** module exposes **`draw_edges`** only; each **export binary** owns its **model matrix**.
- **Outcome:** One **lossless animated `.webp`** (**800×600**) showing the **orthographic wireframe cube** tumbling smoothly over **`ANIMATED_CUBE_FRAME_COUNT`** frames (**20 ms** spacing in code → 50 fps), encoded with **`webp-animation`** and **`EncodingType::Lossless`**. This **animation scaffold** is in place for later milestones (wireframe back-face filtering, projection swap, filled raster, shading—same loop).

### [x] Cube: wireframe — back-face–aware edges

- **Goal:** Learn **facet facing** vs the camera (**face normal · view**) on the **`scene/cube::Cube`** hull **while** the export path stays **orthographic** (projection swap is **deferred**: **see `Perspective projection`**). Treat each undirected hull edge as shared by **two faces** (convex cube: standard incidence). **Draw** an edge iff **not** both adjacent faces are **back-facing** (**Option A**–style silhouette / visible-hull wiring—the same classification idea later reused for torus wireframe).
- **Outcome:** Orthographic **`still-cube`** and **`animated-cube`** show **fewer than twelve** segments whenever **back faces hide** hull edges (**animated** exposes silhouette changes over **`ANIMATED_CUBE_FRAME_COUNT`** at **20 ms**). Geometry and adjacency live in **`src/scene/cube.rs`**; raster stays **`wireframe`** + **`draw_line`** only (**no filled triangle raster** yet—the authoritative **`[Vertex; 3]`** stream waits until **`Cube: triangles — wireframe`**).

### [ ] Cube: triangles — wireframe (triangle stream)

- **Goal:** Move the authoritative mesh to a **triangle stream** (**`[Vertex; 3]`** or indexed triangles from **`scene/cube`**) while **still rasterizing only lines**. **Cull back-facing triangles** first (**same facet-facing rule** as hull-edge BF); build **edges** only from surviving triangles (**dedupe** shared edges across adjacent front faces → still **fewer segments** outside degenerate viewpoints). Validates **topology + incidence** ahead of fills without adding a framebuffer depth pass yet.
- **Outcome:** **`still-cube`** / **`animated-cube`** match the **prior wireframe look** driven from **triangles** (not just the handcrafted **twelve hull edges**)—foundation for **`Vertex` evolution** (position + **solid face color**) in the fill milestone.

### [ ] Cube: filled raster — faceted colors (**depth buffer deferred**)

- **Goal:** Rasterize **filled triangles** (**half-space / barycentric** _or_ scanlines—stick with one algorithm long-term); carry **triangle-level back-face discard** forward from wireframe (**submit-time**, before fill). Shade each **cube face** with a **distinct flat RGB** (six sides → six deliberate colors)—**no lighting model yet** (**diffuse/lighting stays** in **`Cube: basic shading`**).
- **Outcome:** Animated or still **`WebP`** of a **rotating faceted rainbow-ish cube**. **Defer per-pixel depth test** (**see milestone `Depth buffer`**, placed **before torus**)—a clean **single exterior convex** cube avoids the classic “wrong triangle wins” fights that animated sort-free draws would expose on **general** meshes.

### [ ] Cube: basic shading

- **Goal:** Learn **faceted shading** math (per-face normals duplicated at vertices, or equivalent).
- **Outcome:** Filled cube with **simple shading** (e.g. diffuse term) so faces read as distinct planes.

### [ ] Sphere: more complex shape

- **Goal:** Generate a **procedural sphere**; move toward an **indexed mesh** where it pays off (shared vertices vs triangle soup).
- **Outcome:** **Faceted** shaded sphere (low tessellation should still read “polyhedral”).

### [ ] Sphere: smooth shading

- **Goal:** Learn **smooth shading** via **interpolated normals** (and renormalizing per fragment if you go that route).
- **Outcome:** Same sphere mesh with **smooth** shading.

### [ ] Depth buffer (**opaque occlusion**)

- **Goal:** Introduce **per-pixel depth** (orthogonal depth \(z\) in view space vs **normalized device \(z_{\text{NDC}}\)** vs **inverse depth** — pick one rule and stick to it; align later with **`wgpu`** convention checkpoint). Raster carries **\(z_\text{opaque}\)** alongside RGB and **overwrites only if nearer** (**no transparency** assumption for now). Land this **once** **`cube`/`sphere`** solid rendering is exercised so the filled **torus** can lean on **`z`** for **tube/hole self-overlap** without cramming depth into earlier cube milestones.
- **Outcome:** Reliable **overlap resolution** when **triangle draw order stops being safe**—the **immediate consumer** is **filled torus** (and similar self-occluding opaque meshes). Optionally **stress on cube** (**two overlapping cubes**) if you want a simpler regression harness than the torus alone; **perspective quirks** (**see `Perspective projection`**, **after torus**) are a separate follow-on stress suite.

### [ ] Torus: wireframe — back-face–aware edges (A)

- **Goal:** Handle **torus** topology with the same line rasterizer; classify edges using adjacent **triangle** facing vs the camera (**Option A**)—cube wireframe rehearsal above, generalized to tessellated/quads-through-triangle soup.
- **Outcome:** **Wireframe torus** that draws **silhouette edges** plus edges on the visible (front-facing) side of the mesh, and **drops edges that belong only to back-facing triangles**. Accept that **front-facing self-occlusion** may still look “see-through” (lines visible through the tube/hole where a full hidden-line treatment would remove them). Still **restricted scenes** unless you opt into extra clipping.

### [ ] Torus: wireframe — hidden lines / occlusion (B, optional)

- **Goal:** Improve wireframe realism where **A** is insufficient: suppress segments that are **front-facing** but **occluded** by other parts of the **same** torus (**Option B**).
- **Outcome:** Cleaner wireframe from viewpoints where the hole/tube overlap used to show false edges—implementation choice left open (e.g. depth-buffer tests along edges, mesh/object-space hidden-line approaches). **Skip this milestone** if you prefer to move on once **A** looks good enough.

### [ ] Torus: filled + smooth shading

- **Goal:** Apply the same filled pipeline as the cube/sphere to the torus (**orthographic camera** retained through this milestone—the **projection matrix** swaps in **`Perspective projection`**, **next**).
- **Outcome:** **Smooth-shaded filled torus** under **orthographic projection** (**CPU rasterizer capstone mesh** before **`Perspective projection`** and **Phase 2**).

### [ ] Perspective projection (CPU)

- **Goal:** After the **orthographic torus** track is proven, swap the **camera / projection block** to **perspective**: **homogeneous coordinates**, **`w`** divide, and **explicit guardrails** for **near/far**, **\(w \le 0\)**, and **behind-camera** geometry (**`wgpu` parity** on depth range / NDC handedness stays a **checkpoint for Phase 2** unless you tighten it earlier). Re-validate **`z`** (**depth buffer**) under the new projection where **\(z_{\text{NDC}}\)** / interpolation quirks appear.
- **Outcome:** **`WebP`** still + animated demos that **replay** representative scenes (**cube** wireframe and/or fills, **`torus`** wireframe/fill—as much as appetite allows) **only changing** projection (**same tumble / timing policies** wherever you mirror prior exports). Bridges **Phase 1 CPU** orthography stack to **`wgpu`/NDC intuition** ahead of Phase 2.

### [ ] Phase 2 — GPU / wgpu (to elaborate)

- **Goal:** Port the proven pipeline concepts to **wgpu** on **macOS** (Metal backend).
- **Outcome (draft checklist):** swapchain/surface, buffers, pipeline state, vertex/fragment shaders, depth test, culling state aligned with a deliberate **wgpu convention checkpoint** (depth range, winding, NDC vs framebuffer).

---

## Notes / deferred

- **Iteration order:** **`Perspective projection`** deliberately follows **filled torus** so **cube → sphere → depth → torus** stays entirely **orthographic** first (simpler **\(w\)**-free correctness). **`Triangles + wireframe`** introduces **`[Vertex; 3]`** while drawing **edges** (**submit-time BF**). **`Depth buffer`** sits **before torus** for **tube/hole overlap** under **orthographic** **`z`**; revisit **\(z_{\text{NDC}}\)** / depth behavior once **perspective** lands (**see `Perspective projection`**). **`Filled faceted cube`** (**six flat colors**) still skips **`z`** for a sane **single convex exterior** reading.
- **Golden image regression tests:** add when eyeballing saturates — decode `.webp` to RGB and compare, or compare raw framebuffer bytes **before** encode (still vs animated).
- **Live window** (`winit` + framebuffer blit): optional after disk-export workflow is boring—pairs naturally with animation (**real-time** rotation instead of writing WebPs).
- **PNG / ffmpeg:** optional escape hatches for tooling compatibility or pixel-diff tooling that prefers PNG—**not** the default deliverable.
