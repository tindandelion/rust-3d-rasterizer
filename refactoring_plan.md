# Refactoring plan

Forward-looking refactors aligned with **`doc/planning/project-breakdown.md`**.

Completed foundational work—the library + **`src/bin`** layout, **`scene::cube`** (**`Cube`**, **`Edge`**), crate-root **`SCENE_WIDTH`**, **`SCENE_HEIGHT`**, and **`DEFAULT_OUT_PATH`**—is left to git history and **`AGENTS.md`**. It is intentionally not listed below.

## Current layout

| Piece | Role |
| --- | --- |
| **`src/lib.rs`** | **`SCENE_*`**, **`DEFAULT_OUT_PATH`**, `mod …`, **`pub use`** facade (`FrameBuffer`, `Rgb`, `Camera`, `WebpEncoder`) |
| **`framebuffer.rs`** | RGB storage, **`set_pixel`**, DDA **`draw_line`** |
| **`ortho_camera.rs`** | **`Camera`**, NDC-ish → framebuffer **`UVec2`** |
| **`scene.rs`**, **`scene/cube.rs`** | **`Cube`**, **`Edge`**, **`UNIT_VERTS`**, **`edges()`** (topology + transform stored on **`Cube`**) |
| **`webp_encoder.rs`** | Lossless **`WebpEncoder`** around **`webp-animation`** |
| **`src/bin/still-cube.rs`** | Builds **`cube_transform()`**, local wireframe raster loop (**`Camera::transform`** + **`draw_line`**), CLI + WebP encode |
| **`tests/`**, **`tests/common/mod.rs`** | Spawn **`still-cube`**, decode WebPs, **`cube`** golden compare |

Observations worth keeping in mind:

- **`still-cube`** is the natural home for demo-specific **`Mat4`** (scale + tilt) until a shared “demo scene” API exists.
- The local **`draw_wireframe`** in **`still-cube`** is the pattern other wireframe demos will repeat until item **1** lands.
- **`tests/common`** hard-codes **`INTEGRATION_TEST_BIN = "still-cube"`**—if more bins gain golden tests, centralize bin names alongside crate-root **`SCENE_*`** or lift into a thin test harness.

## Candidate refactors (rough priority)

### 1. Generic wireframe helper

Lift the “project both endpoints → **`draw_line`**” loop into **`raster`** (**new**) or **`framebuffer`**, e.g. **`draw_edges`** / **`draw_world_wireframe`** over **`Iterator<Item = Edge>`** or edges + verts + a projector closure.

**Why:** **`Cube::edges()`** already fits; spokes, crossed-square stills, and torus wires should reuse the same path (**`still-cube`** stays declarative).

### 2. Explicit pipeline stages

Name or lightly structure **model/world transforms → `Camera::transform` → raster** (even before full **`Mat4`** view/proj split) so animation and perspective milestones slot in without rewriting **`still-cube`** ad hoc.

**Why:** Matches **`project-spec`** mental model; reduces risk when Euler/quaternion orientation or perspective divide appears.

### 3. **`FrameBuffer` accessors & test layout**

Add **`width()` / `height()`** if exporters or loaders need introspection—optional. Optionally move framebuffer **`cfg(test)`** helpers (**`get_pixel`**, **`to_ascii_art`**) behind a **`test_support`** submodule if they grow.

### 4. **`WebpEncoder::write`** error typing

Replace **`Box<dyn std::error::Error>`** with a small **`enum`** (encode vs **`io::Error`**) instead of collapsing failure sources.

### 5. **`ortho_camera`** naming

Defer renaming to something like **`camera`** (with orthographic vs perspective beside each other) until the perspective milestone is real—avoid churn for its own sake.

## Deferred intentionally

- Heavy plugin-style **`trait`** graphs spanning every milestone.
- Cargo **workspace split** unless maintainability demands it (**`AGENTS.md`**).

## References

- **`AGENTS.md`** — conventions, committing, scope.
- **`doc/planning/project-breakdown.md`** — milestone order (animation, perspective, filled raster, …).
- **`doc/planning/project-spec.md`** — coordinates, framebuffer contract.
