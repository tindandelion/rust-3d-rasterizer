---
layout: post
title: "A Cube Takes Shape"
date: 2026-05-15 06:00:00 +0200
authors: Sergey and Cursor
---

[Version 0.0.1][post-one-white-pixel] proved the export path. [Version 0.0.2][post-lines-without-guesswork] gave us dependable line segments. Version 0.0.3 is the milestone we had been aiming at since then: a **wireframe cube** in **orthographic** projection, exported as the same **800×600** lossless [WebP][webp] still.

[Version 0.0.3 on GitHub][version-0-0-3]{: .no-github-icon}

## What you will see

The radial spoke pattern is gone. In its place is a tilted cube: twelve white edges on black, readable as three dimensions at a glance.

![Current render output](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.3/doc/output/current.webp)

That image is the whole point of this release. Not a demo pattern for line quality anymore, but geometry we care about projected onto the framebuffer.

## From lines to a scene

The rendering loop is intentionally small. We keep the existing [`draw_line`][source-draw-line] path from 0.0.2, but [`main`][source-main] now owns cube data and orchestration:

- **eight** corner vertices in model space (`CUBE_VERTS`, edge length **0.5** centered at the origin),
- **twelve** undirected edge pairs (`CUBE_EDGES`),
- for each edge, transform both endpoints through [`Camera::transform`][source-camera-transform], then rasterize.

[`draw_cube_wireframe`][source-draw-cube] applies a fixed model rotation before projection, then walks the edge list. No triangle fill, no depth buffer yet — only segments.

We also added [glam][glam] for `Vec3`, `Mat3`, `Mat4`, and `UVec2`. The custom `Point(u32, u32)` wrapper went away; pixel endpoints are now [`glam::UVec2`][glam-uvec2] end to end, which keeps the math types consistent as the scene grows.

## Orthographic mapping and aspect ratio

New module [`ortho_camera`][source-ortho-camera] holds a [`Camera`][source-camera] built once per framebuffer size. Its job is **world `xy` → pixel coordinates**, with conventions documented in the module header: left-handed scene, **+Y** up, **+Z** forward, bitmap origin top-left with **+y** down.

The subtle bug we hit early was **aspect distortion**. Mapping **[-1, 1]²** independently onto an **800×600** grid stretches a world-square into a **4:3** rectangle on screen. The fix is a single scale from the **shorter** framebuffer edge, centered on the bitmap:

```text
scale = min(width − 1, height − 1) / 2
px    = world.x * scale + (width − 1) / 2
py    = −world.y * scale + (height − 1) / 2
```

On a non-square viewport, that letterboxes (or pillarboxes) the content so equal world spans along **x** and **y** still occupy equal pixel spans. A unit square stays square in the image.

Under the hood this is a precomputed [`Mat4`][glam-mat4] viewport matrix (`ndc_viewport_matrix`), applied as homogeneous multiply then `round` to integers — same mapping, clearer composition order for when we add more matrix stages later.

**Depth is not part of this milestone.** [`Camera::transform`][source-camera-transform] uses **x** and **y** only; **z** does not change the pixel. That is deliberate for now and covered by a unit test (`world_z_shift_does_not_change_screen_xy`).

## Why the cube must be tilted

Without a model rotation, an axis-aligned cube projected with **xy-only** mapping collapses visually: the front face fills the square, and edges that differ only in **z** project to the same **(x, y)** — they never appear.

We tried an axis-aligned pass first to sanity-check aspect ratio (one face-on square — expected). The shipped pose uses **π/4** rotation about **Y**, then **X** (`Mat3::from_rotation_x * Mat3::from_rotation_y`), documented inline in [`draw_cube_wireframe`][source-draw-cube]. That is enough tilt for depth-only edges to show up on screen while keeping the math easy to reason about.

A full **look-at** view matrix and general orthographic frustum are on the roadmap; this release keeps the camera path narrow so we could ship the milestone and learn from the picture.

## How we got here (and what we removed)

Between 0.0.2 and 0.0.3 we spent real time on **orthographic projection** the hard way: integration tests, NDC conventions, viewport mapping, even depth packing experiments — then peeled layers back when the scope for *this* milestone became clear.

At one point there was a richer stack: separate mesh modules, triangle soup, wireframe edges derived from triangulation, `look_at_lh`, orthographic frustum parameters, fractional `draw_line_f`, and a golden snapshot with JSON metadata. Sergey kept the tests, planning notes, and reference WebP, but **deleted the implementation** to rebuild it step by step with guidance rather than inheriting a large diff.

Version 0.0.3 is that slimmer rebuild: cube vertices and edges live in `main`, projection lives in `ortho_camera`, and the line rasterizer stayed familiar. The diary lesson matches the project goal: use agents heavily, but still own the shape of the code you will maintain six months from now.

## Regression: golden WebP snapshot

Line tests still use ASCII art helpers. For the full scene we added an integration test that decodes the rendered WebP and compares **RGBA** bytes to a committed snapshot at [`snapshots/cube/scene.webp`][source-snapshot].

Regenerate after intentional visual changes:

```bash
cargo run --quiet -- snapshots/cube/scene.webp
```

That gives us the same two-level feedback as before: local geometry correctness in unit tests, end-to-end artifact validity (and now pixel-exact scene stability) at integration scope.

## What this version unlocks

We now have the orthographic wireframe backbone the [milestone breakdown][project-breakdown] described: fixed camera framing, twelve edges, one chosen orientation, still image only.

Not in 0.0.3 yet: time, perspective divide, filled triangles, depth buffer, culling, or lighting.

The natural next step is **animated orthographic wireframe** — reuse this cube and camera scaffold, drive model orientation per frame, and emit a lossless animated WebP with the same encoder path. After that comes perspective, then filled raster with depth.

[version-0-0-3]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.3
[post-one-white-pixel]: {{site.baseurl}}/{% post_url 2026-05-10-one-white-pixel %}
[post-lines-without-guesswork]: {{site.baseurl}}/{% post_url 2026-05-11-lines-without-guesswork %}
[webp]: https://developers.google.com/speed/webp
[glam]: https://docs.rs/glam/latest/glam/
[glam-uvec2]: https://docs.rs/glam/latest/glam/struct.UVec2.html
[glam-mat4]: https://docs.rs/glam/latest/glam/f32/struct.Mat4.html
[project-breakdown]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/doc/planning/project-breakdown.md
[source-draw-line]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.3/src/framebuffer.rs#L51
[source-main]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.3/src/main.rs#L53
[source-draw-cube]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.3/src/main.rs#L72
[source-ortho-camera]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.3/src/ortho_camera.rs#L1
[source-camera]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.3/src/ortho_camera.rs#L44
[source-camera-transform]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.3/src/ortho_camera.rs#L63
[source-snapshot]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.3/snapshots/cube/scene.webp
