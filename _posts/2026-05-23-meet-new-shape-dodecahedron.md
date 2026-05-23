---
layout: post
title: "Meet a New Shape: The Dodecahedron"
date: 2026-05-23 08:00:00 +0200
authors: Sergey and Cursor
---

[The previous version][post-cube-gets-light] gave us a shaded cube and diffuse reflection. However, our code was tightly coupled to that cube implementation, which bothered Sergey, who wanted the code to be more generic. He added an intermediate milestone and chose a more interesting faceted solid: a [dodecahedron][dodecahedron-wikipedia]. That led to a few generalizations in the rendering pipeline.

[Version 0.0.9 on GitHub][version-0-0-9]{: .no-github-icon}

## What you will see

The shape we render is no longer the cube. You will see a rotating shaded dodecahedron — twelve pentagonal faces, tessellated into 36 triangular facets, so the silhouette has more detail than the cube.

Lighting is unchanged from [The Cube Gets Light][post-cube-gets-light]: same orthographic camera, diffuse shading, and front-face culling. Only the mesh representation and the 2D raster primitive changed.

![Animated diffuse-lit dodecahedron with faceted shading](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.9/doc/output/current.webp)

## Switching to a triangular mesh

Until now, we've been [focusing only on the cube shape][post-cube-paints-six-faces], and that impacted the code: our drawing primitives were _quads_ — 4-vertex convex polygons in 2D space. That worked well for the cube, but was not well suited to the dodecahedron.

A regular dodecahedron has twelve pentagonal faces in 3D — five corners each — which you cannot represent cleanly as a single quad per face. Introducing a special 5-vertex 2D polygon did not feel like a scalable way to go, either. Technically, it's possible to have an n-vertex convex polygon as a drawing primitive, but graphics engines tend to go in the opposite direction: instead of inventing more and more complex shapes, we find ways to represent them with simpler, more generic primitives. And what's the simplest primitive for a piece of a flat surface? A triangle.

Triangles are the usual building block in computer graphics for their properties:

- three non-collinear points always lie in one plane;
- any convex polygon in 2D can be filled reliably once you have a consistent triangulation algorithm;
- you need only one simple algorithm to fill a triangle.

## Implementation

This version did not introduce brand-new algorithms or concepts. Instead, it generalized existing code so we can render any solid built from triangles. The main artifacts of this milestone are:

- a way to represent arbitrary objects in 3D space, as long as we know how to build them from triangular facets;
- a more generic rendering pipeline that does not need to change each time we introduce a new shape.

### Generic data types to represent 3D shapes

We've switched to _triangular meshes_ in this release. Along the way, a few generalizations made the pipeline ready for more interesting shapes:

- [`TriMesh`][source-trimesh] is the trait the renderer cares about: given the camera’s view direction, yield [`Triangle`][source-triangle] records — three world-space corners and a normal ready for projection and shading.

- [`Shape`][source-shape] represents the object in 3D space, modelled as a list of triangular _facets_. We want this data structure to be memory-efficient: many facets share the same vertices, so we don't want to store facet vertices for each facet. Instead, we have a shared vector of `Vec3` vertices. Facets themselves (represented by the `Facet`) store vertices as _indexes_ into that list.

- [`Facet`][source-facet] is one planar triangle in 3D space: three counter-clockwise (CCW) vertex indices into the parent mesh and a stored outward [`Normal3`][source-normal3].

### Pre-defined shapes

We now have a generic `Shape` data type and constructor functions that return instances for our pre-defined shapes: a cube and a dodecahedron.

[`unit_cube`][source-unit-cube] still sits in the same **[-½, ½]³** box as before, but each of the six former quads is now two facets with consistent outward winding. Eight vertices, twelve facets, no separate `Quad` type anymore.

[`unit_dodecahedron`][source-unit-dodecahedron] builds the shape from a golden-ratio vertex table (20 vertices) and thirty-six facets. Cursor was in charge of the implementation. it decided to take the index data from [three.js `DodecahedronGeometry`][threejs-dodecahedron] at detail _0_ — three triangles per pentagon, precomputed index list. Each facet’s normal is derived from its corner positions so lighting stays consistent after transforms.

### Filling triangles in pixel space

[`FillTriangle`][source-fill-triangle] follows the same design as the retired quad helper: project corners, scan a bounding box, and classify pixels with the same half-plane inside test. See the source for edge winding and framebuffer clipping.

At the scene level, [`draw_faces`][source-draw-faces] is now generic over `TriMesh`: for each front-facing triangle, compute diffuse intensity from the facet normal, project corners through the orthographic camera, and call `FillTriangle::draw`. The cube and dodecahedron share that loop; only the mesh data changes.

## What comes next

The same pipeline now accepts any `TriMesh` solid. The open milestone is **[Sphere: triangular mesh][project-breakdown-sphere]** — procedural sphere tessellation on that path, with room to tighten indexed mesh structure (shared vertices vs triangle soup). After that come smooth shading, a depth buffer, the torus, and eventually perspective projection — still in the [breakdown order][project-breakdown-0-0-9] we sketched when we added the dodecahedron step.

[dodecahedron-wikipedia]: https://en.wikipedia.org/wiki/Regular_dodecahedron
[post-cube-gets-light]: {{site.baseurl}}{% post_url 2026-05-22-the-cube-gets-light %}
[post-cube-paints-six-faces]: {{site.baseurl}}{% post_url 2026-05-18-the-cube-paints-its-six-faces %}
[version-0-0-9]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.9
[project-breakdown-0-0-9]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/doc/planning/project-breakdown.md
[project-breakdown-sphere]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/doc/planning/project-breakdown.md#--sphere-triangular-mesh--procedural-tessellation
[threejs-dodecahedron]: https://threejs.org/docs/pages/DodecahedronGeometry.html
[source-fill-triangle]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/src/framebuffer/fill_triangle.rs#L12
[source-shape]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/src/scene/shape.rs#L16
[source-facet]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/src/scene/facet.rs#L13
[source-normal3]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/src/geometry.rs#L7
[source-trimesh]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/src/lib.rs#L52
[source-triangle]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/src/lib.rs#L47
[source-unit-cube]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/src/scene/cube.rs#L44
[source-unit-dodecahedron]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/src/scene/dodecahedron.rs#L91
[source-draw-faces]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/src/lib.rs#L59
