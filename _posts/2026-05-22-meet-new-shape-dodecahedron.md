---
layout: post
title: "Meet new shape: Dodecahedron"
date: 2026-05-22 12:00:00 +0200
authors: Sergey and Cursor
---

[Version 0.0.8][post-cube-gets-light] gave us a shaded cube: Lambert diffuse, directional light, one blue albedo — but the geometry path was still built around **quads**. Each hull side was four corners submitted to a quad fill routine. The [project breakdown][project-breakdown-0-0-8] already named **Sphere: triangular mesh** as the next big step: refactor the cube to two triangles per face, land one triangle rasterizer, and stop maintaining quad and triangle fill paths in parallel.

We changed the plan. Instead of jumping straight to a procedural sphere, we inserted an intermediate milestone: a **regular dodecahedron** as the showcase solid. It keeps the scene **faceted** (flat shading per planar facet, same diffuse model as the cube), but **pentagonal faces cannot be honest quads** in 3D — so the shape **forces** a _triangular mesh_. The cube is refactored onto that same stack in the same release, and the old quad fill path is gone.

[Version 0.0.9 on GitHub][version-0-0-9]{: .no-github-icon}

## What you will see

The default animated export is no longer a tumbling cube. **`animated-scene`** rotates a shaded **dodecahedron**: twenty vertices, thirty-six triangular facets, same orthographic camera and Euler tumble as before (**`R_z R_y R_x`** with a common angle, **360** frames at **50 fps**). Facets toward the light read brighter; facets in shadow fall darker — the same [`DiffuseLight`][source-diffuse-light] we used on the cube, still multiplied into [`CUBE_ALBEDO`][source-cube-albedo] (the name is historical).

![Animated diffuse-lit dodecahedron with faceted shading](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.9/doc/output/current.webp)

The silhouette is busier than a cube: twelve pentagons, each split into three coplanar triangles, so edges and shading bands read as a gem-like polyhedron rather than six flat squares. **`still-cube`** still exports a single-frame **cube** for tests and snapshots; only the animated demo switched shapes.

## Why a triangular mesh

A _mesh_ is how we store a solid for the rasterizer: a list of **vertex positions** plus a list of **faces** that reference those positions. Until now, each cube face was a **quad** — four indices, one normal, one flat color. That matched the interim [`FillQuad`][fill-quad-wikipedia] path from [Version 0.0.6][post-cube-paints-six-faces].

Triangles are the usual building block in real-time graphics for a simple reason: **three non-collinear points always lie in one plane**, and **any convex polygon in 2D can be filled reliably once you pick a consistent inside test on three edges**. Quads are convenient for boxes, but a pentagon in 3D is not a single convex quad in facet form — you **triangulate** it (split it into triangles that share edges and stay coplanar per face).

For this milestone we wanted:

1. **One** filled primitive in the framebuffer — [`FillTriangle`][source-fill-triangle], not quad plus triangle.
2. **One** submission type at the raster boundary — three world-space corners per facet, plus an outward normal for lighting and back-face culling.
3. A **nicer demo shape** than “cube, but now with twelve triangles” — the dodecahedron delivers that while staying procedurally defined and faceted.

So the dodecahedron is not a detour from triangles; it is the **driver** for switching the whole filled pipeline.

## Indexed facets: `Shape`, `Facet`, and `TriMesh`

We introduced a small mesh layer in `scene`:

- [`Shape`][source-shape] holds `Vec<Vec3>` vertices and `Vec<Facet>` faces. Construction is generic: arbitrary vertex table plus facet list.
- [`Facet`][source-facet] is one planar triangle: three **CCW** vertex indices into the parent mesh and a stored outward [`Normal3`][source-normal3]. It mirrors the old per-face metadata (normal, winding, front-facing test) but for three corners only.
- [`TriMesh`][source-trimesh] is the trait the renderer cares about: given the camera’s view direction, yield [`Triangle`][source-triangle] records — three **world-space** corners and a normal ready for projection and shading.

[`unit_cube`][source-unit-cube] still sits in the same **[-½, ½]³** box as before, but each of the six hull quads is now **two** facets — a wedge split **`(w, x, y)`** and **`(w, y, z)`** with consistent CCW winding when seen from outside along the face normal. Eight vertices, **twelve** facets, no separate `Quad` type on the filled path.

[`unit_dodecahedron`][source-unit-dodecahedron] builds a [`Shape`][source-shape] from the golden-ratio vertex table (scaled so **max |x|, |y|, |z| = 0.5**, same framing as the cube) and **thirty-six** facets taken from [three.js `DodecahedronGeometry`][threejs-dodecahedron] at detail **0** — three triangles per pentagon, precomputed index list. Each facet’s normal is derived from its corner positions so lighting stays consistent after transforms.

## Filling triangles in pixel space

[`FillTriangle`][source-fill-triangle] follows the same design as the retired quad helper: project integer pixel corners, build an **axis-aligned bounding box**, scan it, and classify each pixel with a **half-plane** test using `Vec2::perp_dot` on the three edges. Consistent winding means “inside” is where all three cross products agree in sign; edges are inclusive, and out-of-bounds samples are dropped by the framebuffer.

At the scene level, [`draw_faces`][source-draw-faces] is now generic over `TriMesh`: for each front-facing triangle, compute diffuse intensity from the facet normal, scale albedo, project corners through the orthographic camera, and call `FillTriangle::draw`. The cube and dodecahedron share that loop; only the mesh data changes.

We removed [`FillQuad`][fill-quad-wikipedia] entirely. Regression coverage for convex quad silhouettes lives on as **pairs** of triangle draws in unit tests (two triangles that partition a rectangle or slanted quad), so we do not lose clipping and winding cases when only `FillTriangle` remains.

## Implementation notes

The public surface area is intentionally small:

| Piece | Role |
| --- | --- |
| `FillTriangle` | 2D filled triangle raster |
| `Facet` / `Shape` | Indexed triangle mesh storage |
| `TriMesh` + `Triangle` | World-space facets for `draw_faces` |
| `unit_cube` / `unit_dodecahedron` | Procedural meshes |
| `animated-scene` | Filled, shaded, tumbling dodecahedron export |

Renaming **`animated-cube`** → **`animated-scene`** reflects that the default animation is no longer cube-specific. Version **0.0.9** is tagged on GitHub; the still above is pinned to that tag.

## What comes next

The triangle stack is in place for all filled solids. The open milestone is **[Sphere: triangular mesh][project-breakdown-sphere]** — procedural sphere tessellation on the same `TriMesh` path, with room to tighten indexed mesh structure (shared vertices vs triangle soup). After that come smooth shading, a depth buffer, the torus, and eventually perspective projection — still in the [breakdown order][project-breakdown-0-0-9] we sketched when we added the dodecahedron step.

[post-cube-gets-light]: {{site.baseurl}}/{% post_url 2026-05-22-the-cube-gets-light %}
[post-cube-paints-six-faces]: {{site.baseurl}}/{% post_url 2026-05-18-the-cube-paints-its-six-faces %}
[version-0-0-9]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.9
[project-breakdown-0-0-8]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/doc/planning/project-breakdown.md
[project-breakdown-0-0-9]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/doc/planning/project-breakdown.md
[project-breakdown-sphere]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/doc/planning/project-breakdown.md#sphere-triangular-mesh--procedural-tessellation
[fill-quad-wikipedia]: https://en.wikipedia.org/wiki/Polygon#Convex_polygons
[threejs-dodecahedron]: https://threejs.org/docs/#api/en/geometries/DodecahedronGeometry
[source-diffuse-light]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/src/lighting.rs#L9
[source-cube-albedo]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/src/lib.rs#L34
[source-fill-triangle]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/src/framebuffer/fill_triangle.rs#L12
[source-shape]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/src/scene/shape.rs#L16
[source-facet]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/src/scene/facet.rs#L13
[source-normal3]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/src/geometry.rs
[source-trimesh]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/src/lib.rs#L52
[source-triangle]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/src/lib.rs#L47
[source-unit-cube]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/src/scene/cube.rs#L44
[source-unit-dodecahedron]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/src/scene/dodecahedron.rs#L91
[source-draw-faces]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/src/lib.rs#L59
