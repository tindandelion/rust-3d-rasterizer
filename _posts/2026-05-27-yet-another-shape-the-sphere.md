---
layout: post
title: "Yet Another Shape: The Sphere"
date: 2026-05-27 17:00:00 +0200
authors: Sergey and Cursor
---

Until now, the shapes we chose to render had a common property: they are _faceted objects_ in real life too. Now we are moving towards the next level: how to render the shapes that have _smooth surfaces_ in real life? The simplest example to start with is a solid sphere. In this part of our project, we learn how to build a spherical shape procedurally. Along that journey, we've also hit a performance issue that forced us to revisit the algorithm for drawing filled trianges in 2D.  

[Version 0.0.11 on GitHub][version-0-0-11]{: .no-github-icon}

## What you will see

The animated demo now shows a dense faceted sphere instead of the dodecahedron we used in the previous milestone. The clip still has two halves: first the camera orbits the object, then the object tumbles while the camera stays fixed.

![Animated faceted sphere with orbit and tumble phases](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.11/doc/output/current.webp)

For an observer, this definitely looks like something spherical. It still has a faceted texture, though: making it look like a more realistinc sphere is going to be the objective of our [upcoming milestones][link-to-breakdown?]

## Building a sphere from triangles

So a sphere is a smooth surface object, but a 3D rasterizer we're building doesn't know how to work with such surfaces. It does know how to render an object composed of tiangular facets, though. So a natural way to represent a sphere for the reasterizer to render is to make an approximation composed of just enough small triangular facets. Generally, this process is called [_tessellation_][tessellation]: split a coarse shape into smaller pieces until it approximates the surface we want. 

### Common approaches to tesselate a sphere

One common way to represent a sphere as a triangular mesh is [_UV (lat-lon) sphere][link?]. Essentially, we divide the sphere with the latitude/longitude grid, and calculate vertex coordinate for each crosspoint. The downside of this approach, though, is that the resulting rectanges differ greatly in shape: they are getting thinner as we move closer to the poles. 

Another common approach that gives a more regular rectangular grid is [icosphere][link?]. Here, we start with icosahedron and create a sphere-like shape by iteratively subdividing each of its edges multiple times. 

### _Octasphere:_ the golden middle for beginners

We've decided to take an icosphere approach, but use a simpler base shape: an [_octahedron_][octahedron]. That choice was made mainly because of octahedron's simplicity: one can easily write down the coordinates of all its 6 vertices by hand. 

So we start from an unit octahedron and recursively split each triangle into four triangles.

At each split step:

1. Take one triangle with corners `A`, `B`, and `C`.
2. Compute edge midpoints (`AB`, `AC`, `BC`).
3. Normalize those midpoint vectors so they lie on the unit sphere.
4. Build four child triangles from original corners plus normalized midpoints.

After each pass, the triangle count grows by a factor of 4, so even a few passes add a lot of detail quickly. One trick worth noting is that in the subdivision process we have to **keep a midpoint cache** keyed by edge indices, so adjacent triangles reuse the same midpoint vertex instead of creating duplicates.

The implementation of this algorithm lives in [`shapes::sphere`][source-sphere].

## Performance improvement: faster triangle fill

A realistically looking sphere contains much more triangular facets, compared to a cube or dodecahedron. For example, in the  [picture above][internal-link-to-what-you-will-see?] the sphere was created by subdividing the octahedron 4 times, which resulted in a shape with 2048 rectangular facets. 

With that level of detail, we started to notice the significant performance hit: it would take full **28 seconds** to build the clip with the debug build! The release build was significantly faster, obviously: only 4 seconds, but it still felt like a performance degradation. 

So Sergey decided to explore the bottleneck 

### Flamegraphing the executable

To investigate the problem, we used our familiar tool: [_flamegraph_][link-to-flamegraph]: 

![Flamegraph before scanline fill](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.11/doc/performance/animated-scene-before.svg)

So this framegraph shows that we spend most of the time in 2 parts of the program: 

* producing the WebP image takes about 66% of the overall time;
* the rest we spend in `FillTriangle` drawing primitive. 

Leaving WebP generation aside for now, we decided to focus on improving our triangle fill algorithm. 

#### The legacy: half-space triangle fill 

Actually, we've inherited this algorithm from the times when we were [only rendering the cube]: back then we worked with quad shapes and used the [_half-space_][half-space] polygon fill implementation that scans a bounding box and runs inside tests per pixel. That method is robust and easy to reason about, but it comes with an extra cost of testing each pixel inside the bounding box.

#### Rectangle fill is simpler

Half-space polygon fill is a nice algorithm for generic convex polygons, but in fact for triangles we can use a more cost-efficient algorithm of [_scanline rasterization_][scanline-rasterization]:

- Sort triangle vertices by `y`;
- Walk edges to get left/right `x` bounds for each `y` from top to bottom;
- Having found `x_min` and `x_max`, draw a horizontal line.

The full implementation of this algorithm lives in [`ScanlineFillTriangle`][source-scanline-fill] drawing primitive. The previous half-space implementation is still present as [`HalfSpaceFillTriangle`][source-half-space-fill], mostly for nostalgic purposes. 

Using the new algorithm, we can see its impact on the flamegraph: 

![Flamegraph after scanline fill](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.11/doc/performance/animated-scene.svg)

To generate the animated scene, it now takes 12 seconds for the debug build, and 3 seconds for the release. It looks like there's still room for improvement there, but the lion share of the time is spent on WebP generation, anyways. Should we want to make further improvements, that should be the area to focus. 

## Next step towards a realistic sphere

So, now we can render a sphere. It's still far from a realistically looking one, though. That's going to be our primary goal for the next few milestones: we're going to explore advanced shading techniques that make sphere look like, well, a sphere.

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
