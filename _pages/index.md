---
layout: home
title: Welcome
permalink: /
list_title: Project diary
---

This is yet another learning project of mine, and a follow-up to my [BitTorrent client in Rust][bt-blog]. This time, I'm building a **3D software rasterizer in Rust**, developed mainly on **macOS**.

![Current render output](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/main/doc/output/current.webp)

### Motivations

There are two things I want to learn with this project, running side by side.

The first is **3D rasterization itself**. I've always been curious about how 3D graphics actually work under the hood — the math, the coordinate systems, the algorithms that turn a scene description into pixels on a screen. The plan is to build the math and rasterization path on the **CPU** first, then add optional **GPU** rendering with **wgpu** (Metal on Mac) as a stretch goal. Scenes stay small and procedural — working up shapes like a cube, a sphere, and eventually a torus — with lossless **WebP** stills and animations as the main artifacts.

The second motivation, equally important, is to deliberately practice **AI-assisted ("agentic") coding**. Tools like Cursor and similar agents have changed the day-to-day of writing software, and I want to develop a real, hands-on intuition for working with them — not as a passive autocomplete user, but as someone delegating meaningful work and reviewing the result.

### Working with AI agents

A few things I want to figure out along the way:

* **Use AI as much as possible, but consciously.** Where does it shine? Where does it struggle? Where am I tempted to lean on it instead of actually understanding something?
* **AI as a learning partner**, not just a code generator. Asking it to explain trade-offs, derive the math, propose alternatives, and critique my reasoning — and then verifying what it tells me against a primary source.
* **Keeping the code clean** in spite of AI's tendency to produce *slop*: redundant abstractions, over-engineered helpers, dead code, premature generality, and tests that look plausible but assert nothing useful. I want to find prompting patterns, review habits, and project conventions that push back on this.

I'll be writing about what works and what doesn't in the diary below.

### Project scope

I'll consider the project meaningfully accomplished when I can:

* Render a small set of procedural 3D shapes — at minimum a cube, a sphere, and a torus — with a working camera and basic lighting
* Produce smooth animations of those scenes, captured as lossless WebP
* Run the whole pipeline on the CPU, with the optional `wgpu`-based GPU path as a stretch goal
* Look at the codebase six months from now and still be happy to work in it

### Useful links

* [Project specification][spec] — goals, math, dependencies, coordinate conventions, raster strategy
* [Milestone breakdown][breakdown] — iterative milestones and expected artifacts
* [Source code on GitHub][repo] — the Rust implementation lives on the `main` branch
* [BitTorrent client in Rust][bt-blog] — my previous learning project, in the same diary format

[spec]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/doc/planning/project-spec.md
[breakdown]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/doc/planning/project-breakdown.md
[repo]: https://github.com/tindandelion/rust-3d-rasterizer
[bt-blog]: https://www.tindandelion.com/rust-bittorrent-client/
