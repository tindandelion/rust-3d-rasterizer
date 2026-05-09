# Agent guidance

Instructions for AI assistants and contributors working on this repository.

## What this project is

Personal learning project: a **3D software rasterizer** in **Rust**, developed primarily on **macOS**. Work proceeds in two broad phases: **CPU rasterization** first, then optional **GPU (`wgpu`)** acceleration. Authoritative planning lives under:

- `doc/planning/project-spec.md` — goals, math, dependencies, coordinate conventions, raster strategy.
- `doc/planning/project-breakdown.md` — iterative milestones and expected artifacts (WebP stills/animations).

When behavior or scope is unclear, **prefer the planning docs** over guessing.

## Stack and layout

- **Language:** Rust (see `Cargo.toml` for `edition` and package name).
- **Layout:** Single Cargo crate for now; introduce modules as needed (e.g. `raster`, `mesh`, export/WebP). Split into a workspace only if maintainability demands it.
- **Linear algebra:** Plan is to use **`glam`** once math code lands; verify projection/view conventions against the spec—do not assume every helper matches the chosen spaces.
- **Output:** Fixed **800×600** RGBA framebuffer; encode **lossless WebP** (`webp-animation` + native **libwebp** toolchain when that milestone ships).

## Conventions to preserve

- **World/camera intuition:** Unity-style **left-handed**, **+Y up**, **+Z forward** (see spec for clip/screen mapping details).
- **Geometry API:** Rasterizer should consume triangles as a stream (`[Vertex; 3]` or equivalent); evolve `Vertex` only when a milestone needs new attributes.
- **Scope:** Restricted scenes in early phases; procedural meshes; long-term visual target includes a **torus**—follow milestone order in the breakdown doc.

## How to work in this repo

- Run **`cargo build`**, **`cargo test`**, **`cargo fmt`**, and **`cargo clippy`** after substantive changes when applicable.
- Prefer **small, focused changes** that match existing style and module boundaries.
- Do **not** expand scope (new dependencies, large refactors, unrelated features) without a clear ask or alignment with the planning docs.

## Cursor-specific notes

- Project rules may also live under `.cursor/rules/`; this file is the **human- and agent-readable** overview at the repository root.
- Optional local skills under `.cursor/skills/` are user-defined; use them when the task matches their description.
