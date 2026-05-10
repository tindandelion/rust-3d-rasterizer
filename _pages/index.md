---
layout: home
title: Welcome
permalink: /
list_title: Project diary
---

A personal learning project: a **3D software rasterizer** in **Rust**, developed mainly on **macOS**. The plan is to build the math and rasterization path on the **CPU** first, then add optional **GPU** rendering with **wgpu** (Metal on Mac). Scenes stay small and procedural—working up shapes like a cube, sphere, and eventually a torus—with lossless **WebP** stills and animations as the main artifacts.

![Current render output](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/main/doc/output/current.webp)

### Project plans

* [Project specification][spec] — goals, math, dependencies, coordinate conventions, raster strategy  
* [Milestone breakdown][breakdown] — iterative milestones and expected artifacts  

Both documents live in the [`main` branch][repo] of the repository under `doc/planning/`.

### Project blog

Like my [BitTorrent client project][bt-blog], I am documenting progress here as a **project diary**: milestones, detours, and notes from building the renderer.

[spec]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/doc/planning/project-spec.md
[breakdown]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/doc/planning/project-breakdown.md
[repo]: https://github.com/tindandelion/rust-3d-rasterizer
[bt-blog]: https://www.tindandelion.com/rust-bittorrent-client/
