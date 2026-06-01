---
layout: post
title: "The Sphere Gets Smooth"
date: 2026-05-29 10:00:00 +0200
authors: Sergey and Cursor
---

In the [last session][post-yet-another-shape-the-sphere] we've started working on smooth shapes and introduced the first example: a sphere. We did indeed ended up with something that looks like a sphere, but with one notable drawback: looking at the [picture][link-to-last-what-you-will-see?] you could definitely say that is was created from a multitude of small triangular facets. In this session, w're going to work on an asvanced shading technique called _Gouraud shading_ that gives our sphere a nice smooth look, just like the spherical objects look in the real life. 

[Version 0.0.12 on GitHub][version-0-0-12]{: .no-github-icon}

## What you will see

With the addition of smooth shading, the sphere looks like smooth object: you can no longer say that underneath it's composed of thousands of small triangles. 

![Animated Gouraud-shaded sphere with orbit and tumble phases](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.12/doc/output/current.webp)

Sergey has changed the animation routine slightly to better show-case the new ability. Rotating the sphere didn't make much sense: it looked the same from every angle, so instead we started to squeeze it vertically. Notice that throughout that transformation, even the deformed spehere's surface still looks smooth. 

## From flat shading to smooth shading 

Recall that [a while back][link-to-lighting-post?] we introduced the _Lambertian lighting model_ that relates the direction to the light source with the facet's normal: 

$$
I = I_{\max} \cdot \max(0, \mathbf{L} \cdot \mathbf{n})
$$

It worked well for the natrually faceted objects, like the cube and dodecahedron because, roughly speaking, each point at the facet surface receives approximatly the same amount of light. 

Unlike faceted objects,  real sphere has a continuously curving surface. Adjacent points can point in slightly different directions, so the dot product $\mathbf{L} \cdot \mathbf{n}$ changes gradually — not in one jump at each triangle edge. However, since we work with triangular mesh approximations, we don't have access to the value of $\mathbf{n}$ at any arbitrary point. The best we can do is to calculate the exact value of the normal for each mesh vertex, and use some sort of the approximation method to calculate the value of the $\mathbf{n}$ vector at intermadiate points. 

Calculating the normal and the light intensity at each point of the surface is the way to achieve a realistic look, but the trade-off is that it's quite computation-heavy. Often, we can get quite good results with fewer computations, using simpler approximation techniques. [_Gouraud shading_][gouraud] is one of those, widely used in computer graphics. 

## Gouraud shading method 

In a nutshell, Gouraud method consists of those three steps: 

1. Store a separate normal at each mesh vertex.
2. Evaluate lighting at the three projected corners of each triangle.
3. _Interpolate_ those three corner intensities across the triangle's interior while rasterizing.

For the unit sphere, step 1 is trivial: at each point $(x, y, z)$ on the sphere surface the normal is a vector from the origin to that point, so we get the normal automatically from the vertex coordinates: $\mathbf{n} = (x, y, z)$.  Our octasphere builder already places vertices on the unit sphere; at [`shapes::sphere`][source-sphere] we assign `vertex_normals` from those positions when we seed the eight octahedron faces and again when we subdivide edges (new midpoints are normalized onto the sphere, and their normals follow).

Each [`Facet`][source-facet-with-vertex-normals] in the mesh therefore carries three vertex normals, not just one facet normal. Face culling still uses the facet normal (the average of the three vertex normals); only the shading path consumes the per-corner values.

Step 2 doesn't change. We calculate the light intensity at each vertex as we did [before][ref to " From flat shading to smooth shading"]. 

The bulk of computations goes into step 3. Given the light intensities at the triangle's vertices, we do the interpolation independently in two directions: 

![Interpolation scheme][Link to shaded-rect-interpolation.svg]

* First, we linearly interpolate the intensity **along the edges**; 
* Having the interpolated intencity values for left and right points, we can now interpolate the intensity while drawing a horizontal line that connects them. 

We had this algorithm implemented in a new drawing primitive [`ShadedFillTriangle`][source-shaded-fill]. Using it with different intensities for each vertex produces smoothly filled triangles: 

![Shaded triangle example][link to shaded-rect.png]

## The same renderer for both kinds of shapes

We did not fork the renderer into "flat" and "smooth" paths. [`draw_facets`][source-draw-facets] always builds a [`ShadedFillTriangle`][source-shaded-fill] from per-corner normals on every mesh.

For the cube and dodecahedron, [`Facet::with_facet_normal`][source-facet-with-facet-normal] stores the same face normal at all three vertices. Gouraud interpolation between three equal intensities collapses back to flat shading — exactly the old look. Only the sphere supplies distinct vertex normals, so only the sphere gains smooth diffuse.


## Implementation notes

The visible mesh iterator now yields [`Triangle`][source-triangle] values with `[UnitVec3; 3]` normals. `draw_facets` maps each corner through the camera, calls `calc_intensity` per normal, and hands the triple to `ShadedFillTriangle::draw`. [`ScanlineFillTriangle`][source-scanline-fill] remains for unshaded fills and geometry-only tests.

We grew [`ShadedFillTriangle`][source-shaded-fill] incrementally: copy the scanline rasterizer, add intensity to corners, wire tests for gradients and clipping, then connect the sphere's radial normals and switch the main export path. That TDD-style sequence kept the geometry regressions stable while the shading layer came online.

## What comes next

Gouraud smooths diffuse lighting but still evaluates the light model only at vertices. Specular highlights and very tight shading features tend to smear or disappear — a known limitation that motivates [_Phong shading_][phong]: interpolate normals (or renormalize them per pixel) and evaluate lighting at each fragment, with a Blinn–Phong specular term on top. That is the next open milestone on the sphere before we tackle a depth buffer and the torus.

[post-yet-another-shape-the-sphere]: {{site.baseurl}}/{% post_url 2026-05-27-yet-another-shape-the-sphere %}
[post-the-cube-gets-light]: {{site.baseurl}}/{% post_url 2026-05-22-the-cube-gets-light %}
[version-0-0-12]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.12
[gouraud]: https://en.wikipedia.org/wiki/Gouraud_shading
[phong]: https://en.wikipedia.org/wiki/Phong_shading
[project-breakdown]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.12/doc/planning/project-breakdown.md
[source-diffuse-light]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.12/src/lighting.rs#L8
[source-calc-intensity]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.12/src/lighting.rs#L34
[source-sphere]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.12/src/shapes/sphere.rs#L6
[source-facet-with-vertex-normals]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.12/src/geometry/facet.rs#L27
[source-facet-with-facet-normal]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.12/src/geometry/facet.rs#L23
[source-triangle]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.12/src/lib.rs#L55
[source-draw-facets]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.12/src/lib.rs#L67
[source-shaded-fill]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.12/src/framebuffer/shaded_fill_triangle.rs#L13
[source-shaded-corner]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.12/src/framebuffer/shaded_fill_triangle.rs#L8
[source-linear-fn]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.12/src/framebuffer/linear_fn.rs#L1
[source-shape-base-color]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.12/src/lib.rs#L41
[source-scanline-fill]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.12/src/framebuffer/scanline_fill_triangle.rs#L6
