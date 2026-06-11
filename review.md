# Code quality review

Thermo-nuclear audit of the codebase as of 2025-06-11. Scope: full tree (~2,966 lines in `src/`, no file over 1k). Tests and clippy pass clean.

**Overall:** The core stack is coherent — `Mesh` → `visible_triangles` → `Shape::render` → `PhongShadedTriangle` → `write_pixel`. Module boundaries (`geometry/`, `meshes/`, `framebuffer/`, `ortho_camera/`, `lighting/`) are mostly right. The main problems are **documentation lying about deleted code**, **animation-time allocation churn**, and **scene setup scattered across bins** rather than deep raster spaghetti.

---

## Blockers / high-conviction issues

### 1. `AGENTS.md` and `project-spec.md` describe code that no longer exists

`AGENTS.md` still claims:

> **`GouraudShadedTriangle`** and **`Line`** remain for raster tests.

Neither exists anywhere under `src/`. Same for **`draw_facets`**, **`ScanlineFillTriangle`**, **`shapes::sphere`**, **`wireframe`**, **`still-cube`**, etc. in `project-spec.md`. The breakdown doc's **Notes / deferred** section is accurate; **`AGENTS.md` and `project-spec.md` are not**.

This is architectural drift in the docs layer. A new contributor (or agent) will search for `GouraudShadedTriangle`, find nothing, and either reintroduce dead abstractions or misread the design intent.

**Remedy:** Update `AGENTS.md` and `project-spec.md` to match the actual pipeline: `Shape::render` → `PhongShadedTriangle` only. Drop references to removed raster paths unless you deliberately restore them.

---

### 2. Animation hot path allocates a full mesh every frame

In `src/bin/animated-scene.rs`:

```rust
let torus = Shape::new(base_mesh.transform(model_matrix_tumble(t)), TORUS_COLOR);
torus.render(&mut framebuffer, &camera, &light);
```

`Mesh::transform` allocates new `vertices` and `facets` vectors on every iteration — 360 times for a torus with ~3,072 vertices and ~6,144 facets. For rigid motion (rotation + uniform scale), **facet indices are invariant**; only vertex positions and normals change.

This works, but it makes the animation loop carry avoidable allocation pressure and obscures the real per-frame work (project + raster).

**Code-judo move:** Add something like `Mesh::transform_in_place(&mut self, m: Mat4)` (or a reusable `PosedMesh` buffer) that updates vertices/normals without reallocating the facet list. The animation loop becomes: pose once into a scratch mesh, render, repeat. Same behavior, dramatically simpler cost model.

---

### 3. `Shape` in `lib.rs` couples the crate root to raster internals

`lib.rs` imports `PhongCorner` and `PhongShadedTriangle` directly and implements `Shape::render` with per-triangle raster orchestration inline. The rest of the crate keeps a clean "private submodules, flat public surface" policy; this is the leak.

**Remedy:** Move `Shape` to `src/scene.rs` (or `src/shape.rs`). Keep `lib.rs` as constants + re-exports. Optionally, fold the per-triangle loop into a single `PhongShadedTriangle::draw_mesh(...)` helper so `Shape::render` is three lines and raster details stay inside `framebuffer/`.

Not urgent at ~60 lines, but it will get worse when perspective lands (clip, w-divide, per-fragment eye vector).

---

### 4. Export bins duplicate scene contract with a silent light mismatch

Both bins share camera, torus tessellation, material, and color — but **light direction differs**:

| | `still-scene` | `animated-scene` |
|---|---|---|
| Light | `(-10, 10, -10)` | `(1, 0.5, -1)` |
| Camera | `(0, 0.5, -1)` | same |
| Torus | `48×32` | same |

Docs say they share the same Phong setup. They don't. That is either an undocumented intentional choice or accidental drift — either way it makes the still/animated pair unreliable as regression companions.

**Remedy:** Extract shared demo constants (camera, light, torus params, material) into one place — e.g. `src/scene/demo.rs` or `lib.rs` next to `SCENE_WIDTH` — and import from both bins. Per `AGENTS.md`, output paths stay in `src/bin/`.

---

## Missed simplification opportunities

### 5. `NormalInterpolator` is a thin wrapper that may not earn its keep

In `src/framebuffer/phong_shaded_triangle.rs`, `NormalInterpolator` wraps `Interpolator<Vec3>` only to convert `UnitVec3` at the endpoints and re-normalize on `get`. It adds a type and two conversion sites for "lerp Vec3, then normalize." Readable, but if you are looking for judo: either extend `Interpolator` with a `map`/post-process hook, or inline `Interpolator<Vec3>` + `.into()` at the two call sites and delete the wrapper.

The bigger issue is the **panic contract** on opposite normals (`UnitVec3::from` on a zero vector). That is documented in tests but still a runtime footgun on grazing/degenerate geometry. When perspective adds clipping, this becomes more likely.

---

### 6. `BlinnLightModel::calc_intensity` + closure at every draw site

`Shape::render` passes `|normal| light_model.calc_intensity(normal, toward_eye)` into every triangle draw. The eye direction is constant for orthographic rendering.

**Simpler model:** `light_model.intensity_at(normal, toward_eye)` called directly inside `PhongShadedTriangle::draw`, or a small `ShadingContext { light, toward_eye }` passed once per shape. The `impl Fn(UnitVec3) -> f32` callback made sense when Gouraud and flat fills shared a raster core; with only Phong left, it is indirection without a second caller.

---

### 7. `scan_lines` edge selection uses a non-obvious predicate

In `PhongShadedTriangle::scan_lines`:

```rust
let current_edge = if y + 1 > midpoint.y {
    &bc_edge
} else {
    &ab_edge
};
```

`y + 1 > midpoint.y` is easy to misread and hard to prove correct. Classic flat-top/bottom splits usually name the cases explicitly (`y <= midpoint.y` vs `y > midpoint.y`) or split the triangle into two sub-triangles up front.

Tests are excellent (degenerate cases, vertex order invariance, occlusion), so behavior is likely correct — but the control flow is the kind of "magic branch" that becomes spaghetti when you add clipping or sub-pixel precision. Worth a comment or a named helper (`active_short_edge(y, midpoint_y)`) before perspective touches this path.

---

## Boundary / type-contract concerns

### 8. `UnitVec3: From<Vec3>` panics — acknowledged TODO, still a design smell

`src/geometry/unit_vec3.rs` has an open TODO to move to `TryFrom<Vec3>`. Panicking constructors propagate through `Facet::with_vertex_normals` (sum of normals → `into()`), normal interpolation, and mesh builders. For a learning rasterizer this is acceptable; for maintainability, **`TryFrom` at construction boundaries** and infallible `UnitVec3` internally would make failure modes explicit instead of scattered `should_panic` tests.

---

### 9. `Camera::transform` — documented u32 wrap, no clip stage

```rust
FbPixel::new(p.x.round() as u32, p.y.round() as u32, p.z)
```

Module docs honestly describe negative-float → `as u32` wrap. Fine for current demo bounds; **not fine as a permanent boundary** once perspective and larger scenes arrive. The clip stage should live in `Camera` (or a `ProjectedVertex` type with `Option<FbPixel>`), not in every raster caller. Flag this before the perspective milestone, not after.

---

## File size / decomposition

No file crosses 1k lines. Largest is `phong_shaded_triangle.rs` at **554 lines**, but ~**350 are tests**. Production raster code is ~130 lines — healthy.

If this file grows (clip, perspective-correct z, optional Gouraud regression path), **split tests into `phong_shaded_triangle/tests.rs`** or a `#[cfg(test)] mod` file before production code and tests share one 800+ line file.

`framebuffer.rs` embeds ~90 lines of ASCII-art test helpers — fine for now; extract when a second raster primitive needs the same helpers.

---

## What is working well (do not break it)

- **`geometry/` + `meshes/` split** — clean indexed mesh model, `visible_triangles` as the single culling gate.
- **`Interpolator<T>`** — generic, direct, well-tested; good reuse across depth, x, and normals.
- **`ortho_camera.rs`** — exhaustive module docs, panic contracts for degenerate poses, tests for default and arbitrary eye.
- **`Facet` / `NormalTransform`** — inverse-transpose handled once in `Mesh::transform`, not per facet ad hoc.
- **Integration test style** — `draw_unit_cube` uses semantic golden rectangles instead of brittle full-framebyte compares.
- **No premature abstraction** — no trait-heavy render graph, no half-space vs scanline dual pipeline in code (only in stale docs).

---

## Approval bar

| Criterion | Status |
|---|---|
| No structural regression in code | **Pass** — architecture is sound |
| No obvious code-judo missed | **Fail** — animation mesh alloc, scene constant duplication |
| No unjustified file-size explosion | **Pass** |
| No spaghetti branching growth | **Pass** (one opaque branch in scanlines) |
| No hacky/magic abstractions | **Mostly pass** — `NormalInterpolator`, `Fn` callback are mild |
| Clean type boundaries | **Partial** — `UnitVec3` panics, `Camera` u32 cast |
| Canonical layer ownership | **Partial** — `Shape` in `lib.rs`, docs describe deleted layers |
| Docs match code | **Fail** — `AGENTS.md`, `project-spec.md` |

**Verdict: Would not rubber-stamp.** The implementation quality is good for a ~3k-line learning rasterizer, but documentation drift and the animation allocation pattern are real maintainability debts. Fix those before starting **Perspective projection** — that milestone will touch camera, depth, lighting, and raster simultaneously; you want a truthful doc surface and a pose/render split that does not allocate every frame.

---

## Recommended priority order

1. **Sync `AGENTS.md` and `project-spec.md`** with current modules (quick, high leverage).
2. **Unify export-bin scene constants** and fix the light-direction mismatch.
3. **Add in-place mesh posing** for the animation loop.
4. **Extract `Shape` to `scene.rs`** (or add `draw_mesh` on the raster side).
5. **Before perspective:** clip/project boundary on `Camera`, revisit `UnitVec3`/`TryFrom`, simplify Phong shading API (drop callback if no second consumer).
