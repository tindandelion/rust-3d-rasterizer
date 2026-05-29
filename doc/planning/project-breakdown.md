# Project breakdown

This document describes how I plan to approach the project iteratively.

**Current code layout** (module paths, export bins, re-export policy) lives in **Notes / deferred** at the bottom. Completed **`[x]`** milestones below retain their original wording where it describes what was shipped at the time (**`Cube`**, **`scene/cube`**, …); use **Notes / deferred** and **`AGENTS.md`** when navigating the codebase today.

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
- **Outcome:** Orthographic **`still-cube`** and **`animated-cube`** show **fewer than twelve** segments whenever **back faces hide** hull edges (**animated** exposes silhouette changes over **`ANIMATED_CUBE_FRAME_COUNT`** at **20 ms**). Geometry and adjacency live in **`src/scene/cube.rs`**; at this milestone the export bins wireframed via **`draw_line`** only (**filled quads arrive in **`Cube: quad stream — filled faceted raster`**).

### [x] Cube: quad stream — filled faceted raster (**depth buffer deferred**)

- **Goal:** Make **`scene/cube`** authoritative as a **quad stream** (**`[Vertex; 4]`** per face, or indexed quads unfolded at the raster boundary). **Cull back-facing faces** at submit time (**same facet-facing rule** as hull-edge BF). Add a **2D raster helper** for **filled convex quads:** compute the **axis-aligned bounding box** of the four projected vertices (in pixel space), scan pixels inside the box, and **fill only** pixels that pass an **inside test** against the **4-vertex convex** polygon (e.g. consistent **half-plane** tests along edges—simple and sufficient for orthographic **cube** faces). Extend **`Vertex`** as needed (**position + per-face flat RGB**, or equivalent face-level color plumbing). **No lighting model yet** (**diffuse/lighting stays** in **`Cube: basic shading`**). **Optional sanity pass:** build **deduped edges** from surviving quads and **`draw_line`** them to confirm **topology + incidence** and parity with the prior **hull wireframe** look—nice for debugging, not a separate shipping milestone. **Interim evolution:** **`draw_facets`** consumes **`Cube::visible_facets`** (**`Triangle`** per front **`Facet`**) with **one filled-triangle pass** per **`Triangle`**. **`scene/cube`** still **seeds** **twelve **`Facet`**s** from **six** hull **quads**; **`Dodecahedron`** refactors toward **`[Vertex; 3]`** submit (**see open milestone**).
- **Shipped:** **`Cube::visible_facets`**, **`draw_facets`** (**`ScanlineFillTriangle`** per projected **`Triangle`**); **`still-cube`** / **`animated-cube`** call **`draw_facets`** (filled visual regression is covered by in-crate tests such as **`tests/draw_unit_cube.rs`** rather than a committed WebP golden).
- **Outcome:** **`still-cube`** / **`animated-cube`** **`WebP`** exports show a **rotating faceted filled cube** filled from **`Cube::visible_facets`** (**`Triangle`** per **`Facet`**), not handcrafted hull edges (**surface tint** is finalized in **`Cube: basic shading`**: uniform **`SHAPE_BASE_COLOR`** × **`DiffuseLight`**). **Defer per-pixel depth test** (**see milestone `Depth buffer`**, placed **before torus**)—a single **convex exterior** cube stays safe without **`z`**; general meshes still wait for depth.

### [x] Cube: basic shading

- **Goal:** Learn **faceted shading** math (per-face normals duplicated at vertices, or equivalent).
- **Shipped:** **`geometry::UnitVec3`** stored on **`CubeFace`** / **`Quad`**; directional **`DiffuseLight`** (Lambert-style **`max(0, n̂ · L)`** plus clamped **ambient** blend); **`draw_facets`** applies **`SHAPE_BASE_COLOR.scale(intensity)`** per visible quad; **`still-cube`** and **`animated-cube`** construct a fixed **`DiffuseLight`** alongside the existing orthographic / tumble paths.
- **Outcome:** Filled cube with **simple shading** (e.g. diffuse term) so faces read as distinct planes; **Lambert **`DiffuseLight`** → **`SHAPE_BASE_COLOR`** shading** carries over unchanged on **`Triangle`** **`TriMesh`** (**`[x]`** **`Dodecahedron: triangular mesh`**).

### [x] Dodecahedron: triangular mesh — filled faceted solid (**cube → triangles**)

- **Goal:** **Switch filled geometry to triangle corners** (**`[Vec3; 3]`** per **`Triangle`** in code); **raster:** filled triangles via **`draw_facets`** (**shipped **`ScanlineFillTriangle`**; **`HalfSpaceFillTriangle`** alternate). **Refactor **`scene/cube`** so each planar hull face yields **two** indexed **`Facet`** / **`Triangle`** records (**consistent winding** with **`draw_facets`** today); **drop** **`Quad`** from the **filled** submission path. **Ship** the **same **`draw_facets`** path** on a **regular dodecahedron** whose **pentagons force** triangles (early sketch: **centroid fan ~60** tris; **shipped:** **`three.js`** **`DodecahedronGeometry`** detail 0 wedges—**36** planar facets—fine for this milestone).

- **Shipped:** **`TriMesh`** + **`Triangle`** + **`draw_facets`** (**`src/lib.rs`**). **`Cube`:** **twelve **`Facet`**s** (**`(w,x,y)` **`(w,y,z)`** wedges per seeded hull quad**), **`visible_facets`** yields **`Triangle`**. **`scene::dodecahedron::Dodecahedron`:** twenty verts, wedge indices lifted from **`three.js`**. **`still-cube`** (**filled **`Cube`**), **`animated-scene`** (**filled **`Dodecahedron`** + Euler tumble)—**`src/bin/`**.

- **Outcome:** **One **`draw_facets`** filled-triangle path** (**`ScanlineFillTriangle`**) for shaded solids (**no** legacy dual quad/triangle filled raster); **triangle-mesh cube** **plus** faceted shaded **dodecahedron** in **`.webp`** / in-crate tests (**exports as appetite allows**).

### [x] Camera: arbitrary eye — target scene center, world +Y up

- **Goal:** Generalize **view** beyond today’s implicit **fixed** camera: place the eye at any **allowed** position in **world space**, build a **`look-at`** (or equivalent **view matrix**) so the camera **looks at** a **scene target** (**default:** **world origin** / **`Vec3::ZERO`**—adjust if a future “scene root” earns a different focal point). **Camera up** is **always aligned with world +Y** (no tilted horizon / banking); treat **singularities** where **forward ≈ ±Y** as out-of-scope unless you deliberately add an alternate fallback axis.
- **Shipped:** **`ortho_camera::Camera`** (**`src/ortho_camera.rs`**) — **`for_viewport`** / **`move_to`** (**fixed target **`Vec3::ZERO`**, **world +Y** up via Gram–Schmidt; **±Y** pole / **eye = target **`panic`**), **`transform`**, **`direction`** for **`TriMesh`** facing; **`still-cube`** offset eye, **`animated-scene`** **orbit +** tumble halves use **`move_to`**.
- **Outcome:** Existing **orthographic** exports can pick **orbit-style** viewpoints (same **projection** math as today; **only** the **view** / **composite **`view * projection`** plumbing changes where it currently assumes a baked pose). The **`Perspective projection (CPU)`** milestone **reuses** this **eye / scene-target / world-up** convention; it **only swaps** the **projection** matrix.

### [x] Sphere: triangular mesh — procedural tessellation

- **Goal:** On the **unified triangle stack** from **`Dodecahedron`**, generate a **procedural sphere** as **triangular facets**; tighten **indexed mesh** structure where it pays off (**shared vertices** vs triangle soup). **Focus** stays on **sphere math / tessellation**—**cube** **and** **raster baseline** **already** **`[Vertex; 3]`**.
- **Shipped:** **`shapes::sphere(splits)`** (**`src/shapes/sphere.rs`**) — octahedron seed, iterative edge-midpoint subdivision with a midpoint cache; indexed **`Shape`** / **`Facet`** storage; **`still-sphere`** bin (**`sphere(0)`**); **`animated-scene`** uses **`sphere(4)`** via **`draw_facets`**.
- **Outcome:** **Faceted** shaded sphere (low tessellation should still read “polyhedral”).

### [x] Sphere: Gouraud shading — interpolated diffuse

- **Goal:** Learn **smooth shading** via **per-vertex normals** and **interpolated lighting**. Attach **radial normals** at sphere vertices; evaluate existing **`DiffuseLight`** (Lambert + ambient) **at each corner**; extend the scanline rasterizer to **interpolate intensity** (or shaded RGB) across edges and horizontal spans. **Faceted** path for **cube / dodecahedron** stays unchanged (**per-face **`UnitVec3`**).
- **Shipped:** **`geometry::Facet`** stores **`vertex_normals`** (**`with_facet_normal`** duplicates the facet normal for **cube** / **dodecahedron**); **`Triangle`** carries **`[UnitVec3; 3]`**; **`shapes::sphere`** attaches **radial** vertex normals (subdivision preserves them); **`draw_facets`** evaluates **`DiffuseLight`** per corner and draws via **`ShadedFillTriangle`** (**intensity** lerp along edges and horizontal spans). **`ScanlineFillTriangle`** remains for flat/unshaded fills and tests.
- **Outcome:** Same **`shapes::sphere`** mesh reads as a **smooth diffuse** surface (**`still-sphere`**, **`animated-scene`**); low tessellation may still show **Mach bands** or missed tight highlights—that motivates **Phong** next.

### [ ] Sphere: Phong shading — interpolated normals + specular

- **Goal:** **Per-pixel** lighting: interpolate **vertex normals** in screen space, **renormalize** per fragment, then evaluate lighting. Extend the lighting model with **Blinn–Phong specular** (**half-vector**, one **shininess** exponent, modest **specular weight** on top of ambient + diffuse; **view direction** from **`Camera::direction`** is sufficient under orthographic projection). **Defer specular on Gouraud**—highlights computed only at vertices smear or vanish.
- **Outcome:** Same sphere mesh with **smoother diffuse** and **visible specular highlights**; lighting path **reused** on **filled torus** (**see **`Torus: filled + smooth shading`**).

### [ ] Depth buffer (**opaque occlusion**)

- **Goal:** Introduce **per-pixel depth** (orthogonal depth \(z\) in view space vs **normalized device \(z_{\text{NDC}}\)** vs **inverse depth** — pick one rule and stick to it; align later with **`wgpu`** convention checkpoint). **Filled geometry** uses the **unified triangle** path (**cube** already **`[Vertex; 3]`** after **`Dodecahedron`**) so each fragment carries **\(z_\text{opaque}\)** alongside RGB and **overwrites only if nearer** (**no transparency** assumption for now). Land this **once** **`cube` / dodecahedron / sphere** solid rendering is exercised so the filled **torus** can lean on **`z`** for **tube/hole self-overlap** without cramming depth into earlier cube milestones.
- **Outcome:** Reliable **overlap resolution** when **draw order stops being safe**—the **immediate consumer** is **filled torus** (and similar self-occluding opaque meshes). Optionally **stress on cube** (**two overlapping cubes**) if you want a simpler regression harness than the torus alone; **perspective quirks** (**see `Perspective projection`**, **after torus**) are a separate follow-on stress suite.

### [ ] Torus: wireframe — back-face–aware edges (A)

- **Goal:** Handle **torus** topology with the same line rasterizer; classify edges using adjacent **triangle** facing vs the camera (**Option A**)—cube wireframe rehearsal above, generalized to tessellated/quads-through-triangle soup.
- **Outcome:** **Wireframe torus** that draws **silhouette edges** plus edges on the visible (front-facing) side of the mesh, and **drops edges that belong only to back-facing triangles**. Accept that **front-facing self-occlusion** may still look “see-through” (lines visible through the tube/hole where a full hidden-line treatment would remove them). Still **restricted scenes** unless you opt into extra clipping.

### [ ] Torus: wireframe — hidden lines / occlusion (B, optional)

- **Goal:** Improve wireframe realism where **A** is insufficient: suppress segments that are **front-facing** but **occluded** by other parts of the **same** torus (**Option B**).
- **Outcome:** Cleaner wireframe from viewpoints where the hole/tube overlap used to show false edges—implementation choice left open (e.g. depth-buffer tests along edges, mesh/object-space hidden-line approaches). **Skip this milestone** if you prefer to move on once **A** looks good enough.

### [ ] Torus: filled + smooth shading

- **Goal:** Apply the same **triangle** filled pipeline as **cube/sphere** to the torus, reusing the **Phong** path (**interpolated normals + Blinn–Phong specular** from **`Sphere: Phong shading`**). **Orthographic camera** retained through this milestone—the **projection matrix** swaps in **`Perspective projection`**, **next**.
- **Outcome:** **Phong-shaded filled torus** under **orthographic projection** (**CPU rasterizer capstone mesh** before **`Perspective projection`** and **Phase 2**).

### [ ] Perspective projection (CPU)

- **Goal:** After the **orthographic torus** track is proven, swap the **camera / projection block** to **perspective**: **homogeneous coordinates**, **`w`** divide, and **explicit guardrails** for **near/far**, **\(w \le 0\)**, and **behind-camera** geometry (**`wgpu` parity** on depth range / NDC handedness stays a **checkpoint for Phase 2** unless you tighten it earlier). Keep the **`Camera: arbitrary eye`** convention (**eye position**, **scene-center target**, **world +Y up**) for **`look-at`**; perspective only replaces the **projection** half of the matrices. Re-validate **`z`** (**depth buffer**) under the new projection where **\(z_{\text{NDC}}\)** / interpolation quirks appear.
- **Outcome:** **`WebP`** still + animated demos that **replay** representative scenes (**cube** wireframe and/or fills, **`torus`** wireframe/fill—as much as appetite allows) **only changing** projection (**same tumble / timing policies** wherever you mirror prior exports). Bridges **Phase 1 CPU** orthography stack to **`wgpu`/NDC intuition** ahead of Phase 2.

### [ ] Phase 2 — GPU / wgpu (to elaborate)

- **Goal:** Port the proven pipeline concepts to **wgpu** on **macOS** (Metal backend).
- **Outcome (draft checklist):** swapchain/surface, buffers, pipeline state, vertex/fragment shaders, depth test, culling state aligned with a deliberate **wgpu convention checkpoint** (depth range, winding, NDC vs framebuffer).

---

## Notes / deferred

- **Current module layout:** **`geometry`** — **`Shape`**, **`Facet`**, **`UnitVec3`** (re-exported; private **`geometry/{shape,facet,unit_vec3}.rs`**). **`shapes`** — **`cube()`**, **`dodecahedron()`**, **`sphere(splits)`** (re-exported; private **`shapes/{cube,dodecahedron,sphere}.rs`**). **`TriMesh`**, **`Triangle`**, **`draw_facets`**, scene constants — **`src/lib.rs`**. Export bins — **`src/bin/`**.
- **Animated export:** **`animated-scene`** (**`src/bin/animated-scene.rs`**) is the **two-phase** **lossless animated WebP** binary (formerly **`animated-cube`**): **`sphere(4)`** mesh (**Gouraud**), **`0.75`** uniform world scale, **`360`** frames (**`180`** camera orbit **`+`** **`180`** Y squash) at **`ANIMATED_SCENE_FRAME_SPACING_MS`**. **`still-cube`** is the **`shapes::cube()`** orthographic still (**π/4 X/Y tilt**, **½** scale, **faceted**). **`still-sphere`** exports a **Gouraud** **`shapes::sphere(0)`** still (**½** scale).
- **Iteration order:** **`Camera: arbitrary eye`** **[x]** is **orthogonal** to **orthographic vs perspective**: **orbit-style **`look-at`** (eye anywhere sensible, target **scene center** defaulting to **world origin**, **world +Y up**) is **landed** in **`ortho_camera`**. **`Sphere: triangular mesh`** **[x]** (**`shapes::sphere`**). **`Sphere: Gouraud shading`** **[x]** (**`ShadedFillTriangle`**, radial vertex normals). **`Sphere: Phong shading`** and **`Depth buffer`** remain open (**Phong → depth → torus**). **`Perspective projection`** still follows **filled torus** and **inherits** that **`look-at`** policy (**projection** swaps to **`w`** divide afterward). **Overall rhythm:** **cube + dodecahedron + sphere (triangle **`TriMesh`**) → Gouraud → Phong → depth → torus** stays **orthographic** first (simpler **\(w\)**-free correctness). **`Depth buffer`** sits **before torus** for **tube/hole overlap** under **orthographic** **`z`**; revisit **\(z_{\text{NDC}}\)** / depth behavior once **perspective** lands (**see `Perspective projection`**).
- **Golden image regression tests:** add when eyeballing saturates — decode `.webp` to RGB and compare, or compare raw framebuffer bytes **before** encode (still vs animated).
- **Live window** (`winit` + framebuffer blit): optional after disk-export workflow is boring—pairs naturally with animation (**real-time** rotation instead of writing WebPs).
- **PNG / ffmpeg:** optional escape hatches for tooling compatibility or pixel-diff tooling that prefers PNG—**not** the default deliverable.
