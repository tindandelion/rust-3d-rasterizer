# Refactoring plan

This document captures proposed refactorings given the current codebase layout (`main.rs`, `framebuffer`, `ortho_camera`, `webp_encoder`, integration tests that run the package binary). Order is **suggested priority**; adjust to taste.

## High leverage, low ceremony

### 1. Library + thin binary

Add `src/lib.rs` that exposes `framebuffer`, `ortho_camera`, and `webp_encoder` (public or `pub(crate)` as appropriate). Keep `main.rs` as argument handling, wiring, and `main` only.

**Why:** Enables unit and integration tests that call the raster path **without** spawning the binary via `std::process::Command`, and shares types with future examples or benchmarks.

### 2. Move scene data and drawing out of `main.rs`

Relocate cube geometry (`CUBE_VERTS`, `CUBE_EDGES`) and `draw_cube_wireframe` into a dedicated module (e.g. `scene/cube.rs` or `mesh/cube.rs`). `main` should build framebuffer + camera, invoke a small “render this frame” entry point, then encode.

**Why:** Upcoming milestones (animation loop, more meshes) should not grow `main.rs` into a grab bag.

### 3. Single place for scene dimensions and defaults

Centralize canvas size (e.g. 800×600), default output path, and any other shared render defaults in one module (e.g. `config` or `constants`).

**Why:** Reduces drift between `main`, tests, and golden snapshots when those values must stay aligned.

## Medium leverage (as the pipeline grows)

### 4. Explicit pipeline stages (model → camera → raster)

Name or extract steps: apply model transform to vertices, then `Camera::transform`, then line rasterization—even if some steps stay as small pure functions before a full matrix stack.

**Why:** Clarifies where orthographic vs perspective and animation orientation will plug in; matches mental model in `doc/planning/project-spec.md`.

### 5. Generic wireframe helper

Provide a function that draws an edge list against arbitrary vertex positions (e.g. `draw_edges(framebuffer, camera, &verts, &edges, color)`), instead of cube-specialized logic only.

**Why:** Reuse for spokes, crossed-square regression scenes, and later torus wireframe edges.

### 6. Framebuffer API and test helpers

If needed, add `width()` / `height()` for callers. Consider moving test-only helpers (`get_pixel`, `to_ascii_art`) into a `#[cfg(test)]` submodule or `test_support` for clarity while keeping the production `FrameBuffer` API minimal.

**Why:** Keeps tests readable as assertion surface grows.

## Smaller polish (optional)

### 7. `WebpEncoder::write` error types

Replace `Box<dyn std::error::Error>` with a small enum or concrete error chain (encode vs I/O) so callers do not lose the failure source.

**Why:** Better ergonomics without necessarily adding dependencies.

### 8. Module naming vs perspective milestone

Defer renaming `ortho_camera` until a second projection exists, or rename in the same change set that introduces perspective (e.g. `camera` + ortho/perspective types).

**Why:** Avoids rename churn unless it pays off immediately.

## Deferred / avoid for now

- **Heavy trait-based “plugin” architecture** for every future milestone—the crate is still small; two concrete projection paths may stay clearer until the second is real.
- **Cargo workspace split**—only if multiple crates become necessary; see `AGENTS.md`.

## Suggested sequencing

1. Library split + extract cube/scene from `main.rs` (best maintainability per line changed, aligns with animation and shared encode loop).
2. Shared dimensions/defaults + generic wireframe helper as new scenes appear.
3. Error typing and framebuffer API tweaks when pain appears.

## References

- `AGENTS.md` — scope, conventions, when to split the workspace.
- `doc/planning/project-breakdown.md` — milestone order (animation, perspective, filled raster, etc.).
