---
layout: post
title: "Meet New Shape: Dodecahedron"
date: 2026-05-23 08:00:00 +0200
authors: Sergey and Cursor
---

[Previous version][post-cube-gets-light] gave us a shaded cube and diffuse reflection. However, our code was tightly coupled to that cube implementation, which bothered Sergey since he wanted the code to be more generic. So he decided to add an intermediate milestone and have a bot more fun with faceted objects. We've decided to introduce one more shape into our codebase: a dodecahedron (link to Wikipedia?). That led to a few interesting changes and generalizations in the rendering pipeline. 

[Version 0.0.9 on GitHub][version-0-0-9]{: .no-github-icon}

## What you will see

The shape we're rendering is no longer the cube. Instead, you will see a rotating shaded dodecahedron: a more interesting shape with more details. 

![Animated diffuse-lit dodecahedron with faceted shading](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.9/doc/output/current.webp)

## Switching to a triangular mesh

Until now, we've been [focusing only on the cube shape][post-cube-paints-six-faces], and that impacted the code: our drawing primitives were **quads** - 4-vetrex convex polygons in the 2D space. That served well for the cube, but wasn't suited well for the dodecahedron. 

As you can see, dodecahedron has 5-vertex facets: something you can't efficiently represent with quads. Introducing a special 5-vertex 2D polygon didn't feel like a scalable way to go, either. Technically, it's possible to have an n-vertex convex ploygon as a drwaing primitive, but the computer graphics engines go in the opposite direction: instead of inventing more and more complex shapes, we invent the way to represent these shapes with simpler and more generic primitives. And what's the simplest primitive to represent a piece of a flat surface? A triangle. 

Triangles are the usual building block in computer graphics for their properties: 
- **three non-collinear points always lie in one plane**; 
- **any convex polygon in 2D can be filled reliably once you have a consistent triangulation algorithm**;
- **you need only one simple algorithm to fill a triangle**.  

## Implementation 

#### Generic data types to represent 3D shapes

So, we've switched to triangular meshes in this release. Along the way, we've also done a few generalizations in the code that made our pipeline more generic and ready to render more interesting shapes: 

- [`TriMesh`][source-trimesh] is the trait the renderer cares about: given the camera’s view direction, yield [`Triangle`][source-triangle] records — three **world-space** corners and a normal ready for projection and shading.

- [`Shape`][source-shape] represents the object in 3D space, modelled as a list of triangular _facets_. We want this data structure to be memory-efficient: many facets share the same vertices, so we don't want to store facet vertices for each facet. Instead, we have a shared vector of  `Vec3` vertices. Facets themselves (reprsesented by the [`Facet`][source-facet] data type) store vertices as _indexes_ into that list. 

- [`Facet`][source-facet] is one planar triangle in 3D space: three **CCW** vertex indices into the parent mesh and a stored outward [`Normal3`][source-normal3].

#### Pre-defined shapes 

So we have a pretty generic `Shape` data type now. We also have a couple of constructor functions that return `Shape` instacnes for our pre-defined shapes: a cube and a dodecahedron.

[`unit_cube`][source-unit-cube] still sits in the same **[-½, ½]³** box as before, but each of the six quads is now **two** facets — a wedge split **`(w, x, y)`** and **`(w, y, z)`** with consistent counter-clockwise winding when seen from outside along the face normal. Eight vertices, **twelve** facets, no separate `Quad` type anymore.

[`unit_dodecahedron`][source-unit-dodecahedron] builds a [`Shape`][source-shape] from the golden-ratio vertex table (scaled so **max |x|, |y|, |z| = 0.5**, same framing as the cube) and **thirty-six** facets. Cursor took the data from from [three.js `DodecahedronGeometry`][threejs-dodecahedron] at detail **0** — three triangles per pentagon, precomputed index list. Each facet’s normal is derived from its corner positions so lighting stays consistent after transforms.

#### Filling triangles in pixel space

[`FillTriangle`][source-fill-triangle] follows the same design as the retired quad helper: project integer pixel corners, build an **axis-aligned bounding box**, scan it, and classify each pixel with a **half-plane** test using `Vec2::perp_dot` on the three edges. Consistent winding means “inside” is where all three cross products agree in sign; edges are inclusive, and out-of-bounds samples are dropped by the framebuffer.

At the scene level, [`draw_faces`][source-draw-faces] is now generic over `TriMesh`: for each front-facing triangle, compute diffuse intensity from the facet normal, project corners through the orthographic camera, and call `FillTriangle::draw`. The cube and dodecahedron share that loop; only the mesh data changes.

## What we've achieved 

So this version was mostly about generalizing our existing code, not introducing new algorithms. As the main artifacts of this milestone we now have: 

- A way to represent arbitrary objects in 3D space, as long as we know how to build it from triangular facets; 
- A more generic rendering pipeline that doesn't need to change each time we introduce a new kind of shape into the codebase. 

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
