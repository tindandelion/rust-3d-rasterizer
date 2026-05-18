---
layout: post
title: "The Cube Paints Its Six Faces"
date: 2026-05-18 10:00:00 +0200
authors: Sergey and Cursor
---

[Version 0.0.5][post-cube-sheds-hidden-edges] left the cube as a clean wireframe: back faces culled, cornflower blue strokes, no see-through cage. The natural next step was always solid facets — six flat colors, one per side, still without lighting or a depth buffer. Version 0.0.6 ships that look. The exporters now fill visible quads instead of drawing twelve edges.

[Version 0.0.6 on GitHub][version-0-0-6]{: .no-github-icon}

## What you will see

Same orthographic camera, same Euler tumble, 360 frames at 50 fps — but the cube is a faceted solid: rose, sky, amber, iris, jade, and orchid on the six hull sides. Back faces are still culled before rasterization; only front-facing quads reach the framebuffer.

![Animated faceted cube with six flat face colors](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.6/doc/output/current.webp)

Compared with the wireframe era, the motion reads as a colored block tumbling in space, not a line drawing. That is the milestone we wanted before we touch diffuse lighting or triangles.

## Why quads first (and triangles later)

The breakdown still pointed at an honest triangle mesh next — refactor the cube into a real triangle stream before any filled raster, not just the twelve hull edges we knew from wireframe. Sergey pushed back on that ordering: we could stay on quad faces longer, as long as we had a routine to fill 4-vertex convex polygons in screen space.

Cursor walked through the usual options — triangulation into two triangle fills, scanline edge intersections, bbox plus half-plane inside tests, and so on. Sergey narrowed the scope: only approaches that do not require a filled-triangle primitive first. Among those, he wanted bbox + inner test (half-space / cross-product signs on each edge). That matched the cube: each hull side is already a quad in [`CubeFace`][source-cube-face]; project four corners, test inside a 2D convex polygon, paint one RGB per face — enough for the “rainbow-ish” milestone without standing up general triangle fill yet.

Planning caught up in a dedicated pass: quad fill is interim; when the sphere milestone lands, refactor the cube to two triangles per face, implement one triangle rasterizer, and delete the quad path. Sergey was explicit that we must **not keep two fill implementations side by side**. 

## The algorithm for polygon fills

To paint a flat facet we treat it as a [convex polygon][convex-polygon] in screen space — for the cube, a projected quad — and assign one RGB value to every pixel that lies inside it. The idea is two layers: a **bounding-box scan**, then a [point-in-polygon][point-in-polygon] test. We do not triangulate the face and we do not build a [scanline][scanline] edge-intersection table.

#### How it works

First, wrap the vertices in an **axis-aligned bounding box** — the smallest rectangle aligned with the pixel grid that contains all corners. Only pixels inside that box can belong to the polygon; everything outside is discarded immediately. 

Second, for each pixel in the box, ask whether its sample point $p$ lies **inside** the polygon. On a convex shape, “inside” means the same thing as lying on one side of every edge: each directed edge $(v_i, v_{i+1})$ defines a half-plane, and the interior is where all of those agree (half-plane view of convex polygons).

That sidedness is a signed **2D cross product**. For edge $i$, with $e_i = v_{i+1} - v_i$ and $w_i = p - v_i$,

$$
z_i = e_i \times w_i = e_{i,x}\, w_{i,y} - e_{i,y}\, w_{i,x}.
$$

For a point strictly inside a strictly convex polygon, every $z_i$ has the same sign — all positive or all negative, depending on vertex winding and which side you call “inside.” If any $z_i$ disagrees, $p$ is outside. A quad is four such tests per pixel. On convex polygons, these edge signs are enough.

Our code implements this algorithm in the [`FillQuad`][source-fill-quad] ($n = 4$) drawing primitive.

#### Why we chose it

It is the simplest filling algorithm we could adopt with math that stays transparent: a box, a nested loop, and consistent signs on the edges. Scanline fills, triangulation into triangles, and other standard options all work, but they carry more ceremony before the first correct pixel lands. That mattered while we were still learning the raster path and wanted the cube milestone done without getting stuck in the implementation details.

#### Trade-offs

The algorithm only applies to **convex** polygons. That sounds like a restriction, but our cube faces are convex quads after projection, so it is not a practical problem for this release.

The real cost is **performance**: visiting every pixel in the bounding rectangle is wasteful when the shape is skinny or tilted — many candidates fail the inside test after you have already walked to them. A scanline approach that fills only the spans between edge intersections does less throwaway work on a large framebuffer. We have not measured that yet; checking whether the waste hurts us is a job for later, once the filled cube is settled.

## Putting it all together

At the crate root we now have two ways to draw the cube: [`draw_edges`][source-draw-edges] for wireframe strokes, and [`draw_faces`][source-draw-faces] for filled facets. Both take the same camera and mesh, apply the same back-face culling, and project through the orthographic camera — one emits line segments, the other filled quads.

`draw_faces` is the general entry point for this milestone. It walks visible faces, fills each with `FillQuad`, and tints each side from a fixed [`CUBE_FACE_PALETTE`][source-cube-palette] (six deliberate RGB values, one per hull face). That is not a lighting model; it is just enough color variety to show that the full pipeline — cull, project, fill, export — actually works. The still and animated exporters call `draw_faces` instead of `draw_edges`.

`draw_edges` remains for convenience and debugging. We expect to drop it once filled geometry is the only story we care about in the cube exporters.

## What comes next

We still defer a depth buffer. For a single convex cube drawn back-to-front by face culling alone, sort-free fills are safe enough. Overlap fights return when we have self-occluding meshes (torus tube, multiple objects) — that is the depth-buffer milestone, after the cube and sphere share a unified triangle path.

The immediate follow-on is cube basic shading: keep quads for a while, add a simple diffuse term so facets read as planes under a light direction — still on the CPU, still orthographic. After that, the sphere milestone forces the triangle refactor and retires `FillQuad` from steady-state rendering.

[point-in-polygon]: https://en.wikipedia.org/wiki/Point_in_polygon
[convex-polygon]: https://en.wikipedia.org/wiki/Convex_polygon
[scanline]: https://en.wikipedia.org/wiki/Scanline_rendering
[version-0-0-6]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.6
[post-cube-sheds-hidden-edges]: {{site.baseurl}}/{% post_url 2026-05-17-the-cube-sheds-its-hidden-edges %}
[source-fill-quad]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.6/src/framebuffer/fill_quad.rs#L12
[source-draw-faces]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.6/src/lib.rs#L65
[source-cube-palette]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.6/src/lib.rs#L36
[source-cube-face]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.6/src/scene/cube/face.rs#L9
[source-draw-edges]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.6/src/lib.rs#L55
