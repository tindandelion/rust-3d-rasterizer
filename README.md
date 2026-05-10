# 3D rasterizer in Rust

A personal learning project: a 3D software rasterizer implemented in Rust, developed mainly on macOS. The plan is to build the math and rasterization path on the CPU first, then add optional GPU rendering with wgpu (Metal on Mac). Scenes stay small and procedural—working up shapes like a cube, sphere, and eventually a torus—with lossless WebP stills and animations as the main artifacts.

## Current progress

![Current render output](doc/output/current.webp)

## Project plans

- [`doc/planning/project-spec.md`](doc/planning/project-spec.md) — goals, math, dependencies, coordinate conventions, raster strategy
- [`doc/planning/project-breakdown.md`](doc/planning/project-breakdown.md) — iterative milestones and expected artifacts
