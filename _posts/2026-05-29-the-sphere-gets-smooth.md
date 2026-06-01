---
layout: post
title: "The Sphere Gets Smooth"
date: 2026-05-29 10:00:00 +0200
authors: Sergey and Cursor
---

In the [last session][post-yet-another-shape-the-sphere] we started working on smooth shapes and introduced the first example: a sphere. We did end up with something that looks like a sphere, but with one notable drawback: you could definitely tell it was built from a multitude of small triangular facets. In this session, we're going to work on a shading technique called [_Gouraud shading_][gouraud] that gives our sphere a smooth look, like a real sphere.

[Version 0.0.12 on GitHub][version-0-0-12]{: .no-github-icon}

## What you will see

With the addition of smooth shading, the sphere looks like a smooth object: you can no longer tell that underneath it's composed of thousands of small triangles.

![Animated Gouraud-shaded sphere with orbit and tumble phases](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.12/doc/output/current.webp)

Sergey has changed the animation routine slightly to better showcase the new ability. Rotating the sphere didn't make much sense: it looked the same from every angle, so instead we started to squeeze it vertically. Notice that throughout that transformation, even the deformed sphere's surface still looks smooth.

## From flat shading to smooth shading

Recall that [a while back][post-the-cube-gets-light] we introduced the _Lambertian lighting model_ that relates the direction to the light source with the facet's normal:

$$
I = I_{\max} \cdot \max(0, \mathbf{L} \cdot \mathbf{n})
$$

It worked well for naturally faceted objects, like the cube and dodecahedron, because, roughly speaking, each point on the facet surface receives approximately the same amount of light.

Unlike faceted objects, a real sphere has a continuously curving surface. Adjacent points can point in slightly different directions, so the dot product $\mathbf{L} \cdot \mathbf{n}$ changes gradually — not in one jump at each triangle edge. However, since we work with triangular mesh approximations, we don't have the value of $\mathbf{n}$ at every point on the surface. The best we can do is store an exact normal at each mesh vertex and approximate $\mathbf{n}$ at intermediate points.

Using the normal and the light intensity at each point on the surface is the way to achieve a realistic look, but the trade-off is that it's quite computation-heavy. Often, we can get quite good results with fewer computations, using simpler approximation techniques. Gouraud shading is among them and remains widely used in computer graphics.

## Gouraud shading method

In a nutshell, the Gouraud method consists of three steps:

1. Store a separate normal at each mesh vertex.
2. Evaluate lighting at the three corners of each triangle.
3. _Interpolate_ those three corner intensities across the triangle's interior while rasterizing.

For the unit sphere, step 1 is trivial: at each point $(x, y, z)$ on the sphere surface the normal is a vector from the origin to that point, so we get the normal automatically from the vertex coordinates: $\mathbf{n} = (x, y, z)$.

We've added fields to the [`Facet`][source-facet-with-vertex-normals] struct. Each facet in the mesh now carries three vertex normals, in addition to the facet normal. Face culling still uses the facet normal; only the shading path consumes the per-corner values.

Step 2 doesn't change. We calculate the light intensity at each vertex as we did [before](#from-flat-shading-to-smooth-shading).

The bulk of the computation goes into step 3. Given the light intensities at the triangle's vertices, we interpolate independently in two directions:

* First, we linearly interpolate the intensity along the edges;
* Having the interpolated intensity values for the left and right points, we can now interpolate the intensity while drawing a horizontal line that connects them.

![Interpolation scheme]({{site.baseurl}}/assets/images/2026-05-29-the-sphere-gets-smooth/shaded-rect-interpolation.svg)

We implemented this algorithm in a new drawing primitive [`ShadedFillTriangle`][source-shaded-fill]. Using it with different intensities for each vertex produces triangles filled with the gradient:

![Shaded triangle example]({{site.baseurl}}/assets/images/2026-05-29-the-sphere-gets-smooth/shaded-rect.png)

This is going to be our default drawing primitive from now on.

## The same renderer for both kinds of shapes

We did not fork the renderer into "flat" and "smooth" paths. [`draw_facets`][source-draw-facets] always builds a `ShadedFillTriangle` from per-corner normals on every mesh.

For the cube and dodecahedron, [`Facet::with_facet_normal`][source-facet-with-facet-normal] stores the same facet normal at all three vertices. Gouraud interpolation between three equal intensities collapses back to flat shading — exactly the old look. Only the sphere supplies distinct vertex normals, so only the sphere gains smooth diffuse.

## What comes next

Gouraud shading is a useful technique for smooth diffuse surfaces, but it has limits: once you render shiny objects that reflect light differently, it is no longer enough. In upcoming posts we will explore those limits and introduce [_Phong shading_][phong], which can produce more realistic highlights at greater computational cost. Stay tuned!

[post-yet-another-shape-the-sphere]: {{site.baseurl}}/{% post_url 2026-05-27-yet-another-shape-the-sphere %}
[post-the-cube-gets-light]: {{site.baseurl}}/{% post_url 2026-05-22-the-cube-gets-light %}
[version-0-0-12]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.12
[gouraud]: https://en.wikipedia.org/wiki/Gouraud_shading
[phong]: https://en.wikipedia.org/wiki/Phong_shading
[source-facet-with-vertex-normals]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.12/src/geometry/facet.rs#L27
[source-facet-with-facet-normal]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.12/src/geometry/facet.rs#L23
[source-draw-facets]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.12/src/lib.rs#L67
[source-shaded-fill]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.12/src/framebuffer/shaded_fill_triangle.rs#L13
