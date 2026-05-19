---
layout: post
title: "Each Face Catches Its Own Shade"
date: 2026-05-20 14:00:00 +0200
authors: Sergey and Cursor
---

[Version 0.0.7][post-near-face-classified-as-back] fixed front-face culling so we were finally painting the side of the box that faces the camera. The filled cube from [Version 0.0.6][post-cube-paints-its-six-faces] still used six flat tints — enough to prove quad fill, not enough to read as solid geometry under a light. Version 0.0.8 closes the **Cube: basic shading** milestone: one blue material, a directional diffuse term, and brightness that changes with each facet’s orientation.

[Version 0.0.8 on GitHub][version-0-0-8]{: .no-github-icon}

## What you will see

Same orthographic camera, same Euler tumble, **360** frames at **50 fps** — but the cube is no longer a rainbow block. Every visible face shares the same albedo; what varies is how much light reaches it. Facets turned toward the “sun” read brighter; facets in shadow fall darker. The motion finally feels like a lit solid, not a color-coded diagram.

![Animated diffuse-lit blue cube with per-face shading](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.8/doc/output/current.webp)

## One material instead of six

In 0.0.6 we deliberately picked six RGB values from 6 fixed colors so each hull side was unmistakable. That was the right call while we were debugging fill and, later, the culling sign bug. Once facing was trustworthy, the palette became noise: we wanted shading to explain the shape, not paint-by-numbers.

We replaced the palette with a single base color value — saturated blue tuned to stay readable after dimming (**`#346ED2`**, **`Rgb(52, 110, 210)`**). [`draw_faces`][source-draw-faces] multiplies that base color by a scalar intensity per facet; there is no per-slot color lookup anymore.

## Faceted diffuse lighting

We stayed on **faceted** shading: one outward normal per quad, no interpolation across the face. That matches the project breakdown for this milestone and keeps the math visible.

[`DiffuseLight`][source-diffuse-light] stores a unit vector **from the surface toward the light** and an **ambient** weight in **[0, 1]** (clamped in the constructor). For a facet normal $\mathbf{n}$ and toward-light direction $\mathbf{L}$:

$$
\text{diffuse} = \max(0,\, \hat{\mathbf{n}} \cdot \mathbf{L})
$$

$$
\text{intensity} = a + (1 - a)\,\text{diffuse}
$$

where $a$ is the ambient factor. At $a = 0$ you get pure [Lambertian diffuse][lambert]; at $a = 1$ every face is fully lit regardless of orientation. The exporters use **toward** $(1, 1, -1)$ with **ambient 0.25** — enough fill that back-turned facets are not pure black, enough contrast that the tumble still reads.

[`calc_intensity`][source-calc-intensity] normalizes the supplied normal, clamps the result to **[0, 1]**, and [`Rgb::scale`][source-rgb-scale] applies it per channel with rounding. We built the light module first with [TDD-style tests][source-lighting-tests] (parallel, grazing, facing away, ambient-only, blended cases) before touching the raster path.

## Wiring normals into the fill path

[`CubeFace`][source-cube-face] always stored an outward unit normal; it just was not on the type we handed to the rasterizer. We refactored [`Quad`][source-quad] to `{ corners, normal }`, exposed [`CubeFace::normal`][source-face-normal], and had [`visible_faces`][source-visible-faces] copy the posed normal into each surviving quad. [`draw_faces`][source-draw-faces] then does the obvious thing: project corners, compute intensity from `quad.normal`, scale [`CUBE_ALBEDO`][source-cube-albedo], fill with [`FillQuad`][source-fill-quad].

The facing test is unchanged from 0.0.7 — still [`is_front_facing`][source-is-front-facing] with $\mathbf{n} \cdot \mathbf{v} < 0$ for into-scene view **+Z**. Lighting only affects color on facets we already decided to draw.

## Cleaning up after the culling debug pass

[Version 0.0.7][post-near-face-classified-as-back] added [`still-unit-cube`][source-still-unit-cube] — identity pose, blue near cap, red back — to make the facing bug impossible to miss. That binary did its job; 0.0.8 removes it. The same check lives on in [`tests/draw_unit_cube.rs`][source-draw-unit-cube]: orthographic fill of the default unit cube against a hand-built golden framebuffer, now with a light aimed so the front cap’s intensity is **1.0** and the pixels match undimmed [`CUBE_ALBEDO`][source-cube-albedo].

## What comes next

Basic shading on quads is done. The [project breakdown][project-breakdown] points at **Sphere: triangular mesh** next — procedural sphere facets, one triangle fill path, and refactoring the cube to two triangles per face so we do not maintain quad and triangle rasterizers in parallel. Smooth normals, a depth buffer, and the torus track follow after that.

[post-near-face-classified-as-back]: {{site.baseurl}}/{% post_url 2026-05-19-the-near-face-was-classified-as-back %}
[post-cube-paints-its-six-faces]: {{site.baseurl}}/{% post_url 2026-05-18-the-cube-paints-its-six-faces %}
[version-0-0-8]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.8
[lambert]: https://en.wikipedia.org/wiki/Lambertian_reflectance
[project-breakdown]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/doc/planning/project-breakdown.md
[source-cube-palette-0-0-6]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.6/src/lib.rs#L36
[source-cube-albedo]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/src/lib.rs#L31
[source-draw-faces]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/src/lib.rs#L55
[source-diffuse-light]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/src/lighting.rs#L9
[source-calc-intensity]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/src/lighting.rs#L40
[source-rgb-scale]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/src/framebuffer/colors.rs#L13
[source-lighting-tests]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/src/lighting.rs#L47
[source-cube-face]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/src/scene/cube/face.rs#L9
[source-quad]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/src/scene/cube.rs#L26
[source-face-normal]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/src/scene/cube/face.rs#L41
[source-visible-faces]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/src/scene/cube.rs#L78
[source-fill-quad]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/src/framebuffer/fill_quad.rs#L12
[source-is-front-facing]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/src/scene/cube/face.rs#L47
[source-still-unit-cube]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.7/src/bin/still-unit-cube.rs#L1
[source-draw-unit-cube]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/tests/draw_unit_cube.rs#L1
