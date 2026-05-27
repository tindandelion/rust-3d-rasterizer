---
layout: post
title: "Yet Another Shape: The Sphere"
date: 2026-05-27 17:00:00 +0200
authors: Sergey and Cursor
---

Last time, in [We Can Move the Camera Now!][post-camera], we moved the camera around the scene and made the view transform explicit. In this milestone we returned to geometry and rasterization: we now render a tessellated sphere, and we also made filled-triangle rendering faster by switching the production fill path to a scanline algorithm.

[Version 0.0.11 on GitHub][version-0-0-11]{: .no-github-icon}

## What you will see

The animated demo now shows a much denser faceted sphere instead of the dodecahedron we used in the previous milestone. The clip still has two halves: first the camera orbits the object, then the object tumbles while the camera stays fixed.

![Animated faceted sphere with orbit and tumble phases](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.11/doc/output/current.webp)

From a visual perspective, the shape now reads much closer to a sphere while still keeping a deliberately faceted style.

## Building a sphere from triangles

The core geometric idea is [_tessellation_][tessellation]: split a coarse shape into smaller pieces until it approximates the surface we want. Here we start from an [_octahedron_][octahedron] and recursively split each triangle into four triangles.

At each split step:

1. Take one triangle with corners `A`, `B`, and `C`.
2. Compute edge midpoints (`AB`, `AC`, `BC`).
3. Normalize those midpoint vectors so they lie on the unit sphere.
4. Build four child triangles from original corners plus normalized midpoints.

After each pass, the triangle count grows by a factor of 4, so even a few passes add a lot of detail quickly. The implementation keeps a midpoint cache keyed by edge indices, so adjacent triangles reuse the same midpoint vertex instead of creating duplicates.

This approach gives us a practical procedural sphere mesh that fits our current CPU pipeline and keeps topology consistent for culling and shading.

## Why the triangle fill got faster

The second big change is in rasterization. We previously had a [_half-space_][half-space] triangle fill implementation that scans a bounding box and runs inside tests per pixel. That method is robust and easy to reason about, but it does extra work for pixels that are obviously outside the triangle.

The new production path uses [_scanline rasterization_][scanline-rasterization]:

- Sort triangle vertices by `y`.
- Walk edges to get left/right `x` bounds for each row.
- Fill one horizontal span per row.

This reduces per-pixel decision work and makes the hot loop friendlier for CPU rendering. The flamegraphs in this release reflect that shift in where frame time is spent.

![Flamegraph before scanline fill](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.11/doc/performance/animated-scene-before.svg)

![Flamegraph after scanline fill](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.11/doc/performance/animated-scene.svg)

## Implementation details

Sphere generation lives in [`shapes::sphere`][source-sphere], implemented as octahedron subdivision with a midpoint cache and facet-normal recomputation for each generated triangle.

Filled rendering still goes through [`draw_facets`][source-draw-facets], but that path now calls [`ScanlineFillTriangle`][source-scanline-fill] for each projected triangle. The previous half-space implementation is still present as [`HalfSpaceFillTriangle`][source-half-space-fill] for comparison and tests.

The animated output uses [`sphere(4)`][source-animated-scene] for the main demo mesh, while [`still-sphere`][source-still-sphere] remains a lower-detail sanity binary based on `sphere(0)`.

## What comes next

The sphere now has procedural triangular tessellation and a faster fill path, but shading is still faceted. The natural next milestone is smooth shading on the sphere: interpolate normals across each triangle so the lighting reads as a continuous curved surface instead of flat facets.

[version-0-0-11]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.11
[post-camera]: {{site.baseurl}}/{% post_url 2026-05-26-we-can-move-the-camera-now %}
[tessellation]: https://en.wikipedia.org/wiki/Tessellation
[octahedron]: https://en.wikipedia.org/wiki/Octahedron
[scanline-rasterization]: https://en.wikipedia.org/wiki/Scanline_rendering
[half-space]: https://en.wikipedia.org/wiki/Half-space_(geometry)
[source-sphere]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.11/src/shapes/sphere.rs
[source-draw-facets]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.11/src/lib.rs
[source-scanline-fill]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.11/src/framebuffer/scanline_fill_triangle.rs
[source-half-space-fill]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.11/src/framebuffer/half_space_fill_triangle.rs
[source-animated-scene]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.11/src/bin/animated-scene.rs
[source-still-sphere]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.11/src/bin/still-sphere.rs
