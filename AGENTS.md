# Agent guidance

Instructions for AI assistants and contributors working on this repository.

## What this project is

Personal learning project: a **3D software rasterizer** in **Rust**, developed primarily on **macOS**. Work proceeds in two broad phases: **CPU rasterization** first, then optional **GPU (`wgpu`)** acceleration. Authoritative planning lives under:

- `doc/planning/project-spec.md` — goals, math, dependencies, coordinate conventions, raster strategy.
- `doc/planning/project-breakdown.md` — iterative milestones and expected artifacts (WebP stills/animations).

When editing **`doc/planning/project-breakdown.md`**, **do not change completed milestones** (items marked **`[x]`**) unless the **user explicitly asks** to revise that finished task. Prefer updating **open** milestones (`[ ]`) and the **Notes / deferred** section when the plan or shipped reality needs clarification.

When behavior or scope is unclear, **prefer the planning docs** over guessing.

## Stack and layout

- **Language:** Rust (see `Cargo.toml` for `edition` and package name).
- **Layout:** Single Cargo crate for now; introduce modules as needed (e.g. `raster`, export/WebP). Scene-wide defaults (**`SCENE_WIDTH`**, **`DEFAULT_OUT_PATH`**, …) stay as **`pub const`** in **`lib.rs`** until a dedicated config module earns its keep. Split into a workspace only if maintainability demands it.
- **Modules:** Prefer the post-2018 layout for directory modules: **`parent.rs` + `parent/child.rs`** (e.g. `geometry.rs` + `geometry/shape.rs`, `shapes.rs` + `shapes/cube.rs`). Avoid adding new **`mod.rs`** files unless there is a compelling reason. **Private submodules, flat public surface:** implementation files stay **`mod`**-private; re-export types and functions at the parent boundary (**`geometry::Shape`**, **`shapes::cube()`**, …).

## Conventions to preserve

- **World/camera intuition:** Unity-style **left-handed**, **+Y up**, **+Z forward** (see spec for clip/screen mapping details).
- **Geometry API:** **`geometry::Shape`** (**`Vec<Vec3>`** + **`Vec<Facet>`** behind **`vertices()`** / **`facets()`** **`&`** slice accessors) implements **`TriMesh`**; **`geometry::Facet`** and **`geometry::UnitVec3`** are re-exported alongside **`Shape`** (implementation lives in private **`geometry/{shape,facet,unit_vec3}.rs`**). Procedural **`shapes::cube()`**, **`shapes::dodecahedron()`**, and **`shapes::sphere(splits)`** return **`[-½, ½]³`‑boxed **`TriMesh`** shapes** where noted (Plato dodeca: same axis bounds as **`cube()`**, not Plato “edge = 1” circumradius sizing; **`sphere`** is unit-radius before world scale). **`visible_facets`** yields **`Triangle`** (corners **`[Vec3; 3]`**, facet **`UnitVec3`**) per front **`Facet`**; **`draw_facets`** uses one **`FillTriangle`** draw per **`Triangle`**. **Torus** adds geometry on this stack only; evolve **`Vertex`** only when a milestone needs new attributes.
- **Scope:** Restricted scenes in early phases; procedural meshes; long-term visual target includes a **torus**—follow milestone order in the breakdown doc.

## How to work in this repo

- Run **`cargo build`**, **`cargo test`**, **`cargo fmt`**, and **`cargo clippy`** after substantive changes when applicable.
- Prefer **small, focused changes** that match existing style and module boundaries.
- Do **not** expand scope (new dependencies, large refactors, unrelated features) without a clear ask or alignment with the planning docs.

## Commits

**AI assistants:** Treat this section as **mandatory**, not optional workflow polish.

- Use a **brief message** (about **one or two lines**) that captures the **essence** of what changed—avoid long bullet lists or full change logs in the subject body unless truly necessary.
- **Never run `git commit` (or `git add` + `git commit`) in the same turn as the proposal.** First output the **exact** final message (wording as it will appear on the commit), then **stop** and wait for the user’s reply.
- **Do not infer consent** from vague intent—“commit”, “let’s commit”, “save”, or “ship it” only mean the user wants a commit **someday**; they are **not** approval of **your** proposed message until they say so (or send edited wording to use verbatim).
- Only after the user **explicitly approves** that message—or sends **replacement wording** you then use—run **`git add`** / **`git commit`** with **that** message only.

## Cursor-specific notes

- Project rules may also live under `.cursor/rules/`; this file is the **human- and agent-readable** overview at the repository root.
- Optional local skills under `.cursor/skills/` are user-defined; use them when the task matches their description.
