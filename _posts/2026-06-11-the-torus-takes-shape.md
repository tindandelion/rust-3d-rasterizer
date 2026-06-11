---
layout: post
title: "The Torus Takes Shape"
date: 2026-06-11 08:00:00 +0200
authors: Sergey and Cursor
---

[Introducing the depth buffer][post-introducing-depth-buffer] closed the last gap before we could render a shape that hides parts of itself. With that in place, we finally built the mesh we had been aiming at since the project started: a smooth, [Phong-shaded][post-phong-shading-natural-highlights] [_torus_][torus] — the classic donut — tumbling under a fixed camera. This final piece **completes the orthographic CPU rasterizer track for Phase 1**.

[Version 0.0.17 on GitHub][version-0-0-17]{: .no-github-icon}

## What you will see

At last, you will see the rotating torus in our animation clip! As the torus spins, the rasterization pipeline keeps doing all the hard work: the positioning, the lighting, and the [depth buffer][depth-buffer] all play together to render the complete picture.

![Phong-shaded torus tumbling under a fixed orthographic camera](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.17/doc/output/current.webp)

## Why the torus is a capstone mesh

The torus combines all the parts we have built so far:

- **High triangle count** — like the sphere, it needs dense [_tessellation_][tessellation] before it looks round.
- **Smooth shading** — vertex normals vary around the tube, so we lean on the same [Phong shading algorithm][post-phong-shading-natural-highlights] as the sphere.
- **Self-occlusion** — unlike a convex cube or sphere, a torus can block itself: the near side of the tube passes in front of the far side, and the hole creates overlap that [back-face culling][post-cube-sheds-hidden-edges] alone cannot sort out.

That makes the torus an ideal guinea pig to demonstrate all capabilities of our 3D rasterizer.

## Torus generation

The rasterizer pipeline was already prepared to complete this milestone. The only thing missing was the function to generate the torus mesh programmatically. Sergey decided to leave this work to Cursor and not interfere, as long as the end result came out well.

As expected, Cursor did an excellent job of going through the detailed math and ended up with a plausible implementation that we decided to put forward. Here is Cursor's account of the work.

### Two radii, two angles

A _torus_ is defined by two radii:

- **Major radius** $R$ — distance from the origin to the center of the tube as it travels around the ring.
- **Minor radius** $r$ — radius of the tube cross-section itself.

Our generator uses $R = 0.7$ and $r = 0.3$ by default. The major circle lies in the **XZ** plane; the tube is displaced along **Y**, so the hole runs vertically through the middle of the ring.

We sample the surface with two angles, each sweeping a full turn over $[0, 2\pi)$:

- $u$ — position around the major ring.
- $v$ — position around the tube cross-section.

The parametric position is:

$$
\mathbf{p}(u, v) =
\begin{pmatrix}
(R + r\cos v)\cos u \\
r\sin v \\
(R + r\cos v)\sin u
\end{pmatrix}
$$

The outward-pointing normal comes from the same parameterization (no finite-difference tricks needed):

$$
\hat{\mathbf{n}}(u, v) =
\begin{pmatrix}
\cos u \cos v \\
\sin v \\
\sin u \cos v
\end{pmatrix}
$$

Because $\hat{\mathbf{n}}$ is known analytically at every sample, we can store it as a smooth vertex normal and feed it straight into Phong shading — the same pipeline the sphere already used.

### Tessellating the torus grid

As with the sphere, the rasterizer only sees triangles. We turn the parametric surface into an indexed grid:

- **`ring_segments`** — samples of $u$ around the major circle.
- **`tube_segments`** — samples of $v$ around each tube cross-section.

Each grid cell becomes two triangles, exactly like a latitude-longitude patch on a sphere but without poles that pinch. The export binaries use **`torus(48, 32)`** — 1536 vertices and 3072 facets — which is dense enough for a smooth silhouette at 800×600.

Higher segment counts make the torus rounder at the cost of more raster work. The API takes both counts as arguments so we can trade quality for speed without touching the parametric math.

### Implementation details

The mesh lives in [`meshes::torus`][source-torus]. The core sampling function [`torus_frame`][source-torus-frame] evaluates $(\mathbf{p}, \hat{\mathbf{n}})$ for a single $(u, v)$ pair; the public `torus` function walks the segment grid, builds vertices and per-vertex normals, then emits triangle pairs with [`Facet::with_vertex_normals`][source-facet-with-vertex-normals].

From there, nothing torus-specific remains in the draw path. [`Shape::render`][source-shape-render] transforms the mesh through the camera, culls back faces, and hands visible triangles to [`PhongShadedTriangle`][source-phong-shaded-triangle] — depth interpolation and the depth test in [`FrameBuffer::write_pixel`][source-write-pixel] handle tube overlap the same way they handled the [two-sphere demo][post-introducing-depth-buffer].

[`still-scene`][source-still-scene] writes a single lossless WebP of the torus with a shiny blue material. [`animated-scene`][source-animated-scene] reuses the same mesh and camera, applying a uniform scale of $0.8$ and the tumble matrix $R_z(t)\, R_y(t)\, R_x(t)$ with $\alpha = \beta = \gamma = t$ over 360 frames.

## What's next

The orthographic torus completes Phase 1 of our project.

Next, we're going to reflect on the progress so far, do a recap of accomplished tasks, and decide what to focus on next. There's still a lot of work to be done: stay tuned!

[post-introducing-depth-buffer]: {{site.baseurl}}/{% post_url 2026-06-10-introducing-depth-buffer %}
[post-phong-shading-natural-highlights]: {{site.baseurl}}/{% post_url 2026-06-06-phong-shading-natural-highlights %}
[post-cube-sheds-hidden-edges]: {{site.baseurl}}/{% post_url 2026-05-17-the-cube-sheds-its-hidden-edges %}
[version-0-0-17]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.17
[torus]: https://en.wikipedia.org/wiki/Torus
[tessellation]: https://en.wikipedia.org/wiki/Tessellation_(computer_graphics)
[depth-buffer]: https://en.wikipedia.org/wiki/Z-buffering
[source-torus]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.17/src/meshes/torus.rs#L20
[source-torus-frame]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.17/src/meshes/torus.rs#L67
[source-facet-with-vertex-normals]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.17/src/geometry/facet.rs#L27
[source-shape-render]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.17/src/lib.rs#L48
[source-phong-shaded-triangle]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.17/src/framebuffer/phong_shaded_triangle.rs#L18
[source-write-pixel]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.17/src/framebuffer.rs#L50
[source-still-scene]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.17/src/bin/still-scene.rs#L1
[source-animated-scene]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.17/src/bin/animated-scene.rs#L1
