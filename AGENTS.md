# Agent guidance

Instructions for AI assistants and contributors working on this repository.

## What this project is

Personal learning project: a **3D software rasterizer** in **Rust**, developed primarily on **macOS**. Work proceeds in three phases: **Phase 1** CPU rasterization (**shipped** — orthographic, Phong, depth, export bins), **Phase 2** rendering pipeline on CPU (**materials, lights, colors**; export-first), **Phase 3** **`wgpu`** (Metal on Mac). Authoritative planning lives under:

- `doc/planning/project-spec.md` — goals, math, dependencies, coordinate conventions, raster strategy.
- `doc/planning/project-breakdown.md` — phased milestones and expected artifacts (WebP stills/animations); export bins use **`default_material()`** and **`default_lights()`** (one directional + two point lights — see breakdown **Phase 2 reference palette**).

When editing **`doc/planning/project-breakdown.md`**, **do not change completed milestones** (items marked **`[x]`**) unless the **user explicitly asks** to revise that finished task. Prefer updating **open** milestones (`[ ]`) and the **Notes / deferred** section when the plan or shipped reality needs clarification.

When behavior or scope is unclear, **prefer the planning docs** over guessing.

## Stack and layout

- **Language:** Rust (see `Cargo.toml` for `edition` and package name).
- **Layout:** Single Cargo crate for now; introduce modules as needed (e.g. `raster`, export/WebP). Scene-wide constants (**`SCENE_WIDTH`**, **`ANIMATED_SCENE_FRAME_COUNT`**, …) stay as **`pub const`** in **`lib.rs`** until a dedicated config module earns its keep; per-bin output paths and bin-local helpers live in **`src/bin/`** (directory bins when warranted — e.g. **`still-scene/`** with **`kitty_terminal.rs`**). Split into a workspace only if maintainability demands it.
- **Modules:** Prefer the post-2018 layout for directory modules: **`parent.rs` + `parent/child.rs`** (e.g. `geometry.rs` + `geometry/mesh.rs`, `meshes.rs` + `meshes/cube.rs`). Avoid adding new **`mod.rs`** files unless there is a compelling reason. **Private submodules, flat public surface:** implementation files stay **`mod`**-private; re-export types and functions at the parent boundary (**`geometry::Mesh`**, **`meshes::cube()`**, …).
- **Member ordering:** Within a module, **`impl`**, or type body, list items by visibility — **`pub`** first, then **`pub(crate)`** / **`pub(super)`**, then private members (constants, functions, fields, trait impls, and nested **`mod`** blocks).

## Conventions to preserve

- **World/camera intuition:** Unity-style **left-handed**, **+Y up**, **+Z forward** (see spec for clip/screen mapping details).
- **Scope:** Restricted scenes in early phases; procedural meshes; **torus** is the current export-bin mesh — follow open milestones in the breakdown doc (**Phase 2 remaining:** optional stretch CPU perspective, lighting parity; **Phase 3:** **`wgpu`**).

## How to work in this repo

- Run **`cargo build`**, **`cargo test -q`**, **`cargo fmt`**, and **`cargo clippy`** after substantive changes when applicable.
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
