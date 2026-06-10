---
layout: post
title: "Solving Occlusion with Depth Buffer"
date: 2026-06-10 10:00:00 +0200
authors: Sergey and Cursor
---

With [Phong shading][post-phong-shading-natural-highlights] in place, we're almost ready to finish Phase 1: rendering a torus. One last piece of the puzzle is still missing, though. A torus is more complicated than anything we've rendered so far. In particular, it is _self-occluding_: parts of the surface can hide other parts of the same object from the camera. To make the torus look realistic, we need to solve occlusion first.

[Version 0.0.16 on GitHub][version-0-0-16]{: .no-github-icon}

## What you will see

Our rasterizer can now render objects that occlude each other.

To demonstrate this, we've changed the animated scene. You'll now see two spheres sitting next to each other. As the camera circles the scene, one sphere passes behind the other. Overlapping shapes like these are handled by a popular technique called the [_depth buffer_][depth-buffer].

![Two Phong-shaded spheres with correct occlusion as the camera orbits](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.16/doc/output/current.webp)

## The motivation

In the past, we already dealt with the hidden parts of the scene: remember that we had to deal with [back-facing facets][post-cube-sheds-hidden-edges] way back when we were still learning how to render a cube. Then we solved the problem with pure geometry, detecting and dropping back-facing facets based on their orientation with respect to the view direction.

That technique works fine for single convex shapes, such as a cube or a sphere. However, it doesn't work when front-facing surfaces can still block one another — on a torus, parts of the tube pass in front of other parts of the same mesh, and orientation alone cannot tell us which one is behind the other.

The same problem occurs when we start rendering more than one shape in the scene: from the viewer's perspective, parts of one shape can be hidden behind the other. For example, let's look at two pictures of overlapping shapes:

<div class="still-compare">
<figure>
<img src="{{site.baseurl}}/assets/images/2026-06-10-introducing-depth-buffer/still-scene-with-depth.webp" alt="Two overlapping shapes with correct occlusion" />
<figcaption>What we intend to render</figcaption>
</figure>
<figure>
<img src="{{site.baseurl}}/assets/images/2026-06-10-introducing-depth-buffer/still-scene-no-depth.webp" alt="The scene without a depth buffer" />
<figcaption>What you see without the occlusion test</figcaption>
</figure>
</div>

As you can see, without an additional occlusion test, the disc completely obscures the blue sphere in the render, even though geometrically it cuts through it. That's the kind of problem the depth buffer is intended to solve, at the level of individual pixels.


## Depth buffer

Conceptually, a depth buffer is a very simple idea: for each pixel in the frame buffer, along with the color, we store a single number that represents its depth. In our orthographic setup, we can use _view-space_ $z$ value — the third coordinate after the camera transform, measured along the camera's forward axis. Nearer surfaces have smaller $z$; farther ones have larger $z$.

Initially, the depth buffer is filled with a sentinel value representing $+\infty$. When we first write a pixel to the frame buffer, we also record its depth. Now we can perform a _depth test_ when the same pixel gets overwritten:

$$\text{write pixel if } z_{\text{new}} < z_{\text{stored}}$$

If the test passes, we overwrite both the color and the depth. Otherwise, we keep the existing values.

Implementation-wise, the depth buffer is an array of `f32` values, one entry per pixel alongside the RGB data. It costs extra memory, but the appeal of this technique is the ease of implementation.

## Using the _z_ coordinate as depth

As simple as the depth buffer idea is, real implementations have plenty of details to sort out. We're not diving into those yet: orthographic projection leaves view-space $z$ unchanged, so we can use that coordinate directly as the depth value.

The main work was computing $z$ for each pixel. We use our familiar tool for that: linear interpolation. Just as we interpolate `(x, y)` coordinates across a facet from its vertices, we interpolate `z` the same way. The [`PhongShadedTriangle`][source-phong-shaded-triangle] type already handled the interpolation scaffolding; we only had to extend it to interpolate one more value.

[`FrameBuffer`][source-framebuffer] is extended with the additional vector for depth values, and its [`write_pixel()`][source-write-pixel] method has been changed to accept a struct [`FbPixel`][source-fb-pixel], which bundles `(x, y)` coordinates and the `depth` value. The depth test is now a part of the `write_pixel()` body.

## This is still work in progress

As noted above, our depth-buffer implementation is deliberately simple. We'll revisit it later when the project outgrows this approach. Introducing [perspective projection][perspective-projection] will likely require a revisit, because that projection transforms coordinates in ways that make raw $z$ values less reliable as depth.


## What's next

It seems that with the occlusion problem solved, we're finally ready to render and see the **torus** shape in all its splendor!

[post-phong-shading-natural-highlights]: {{site.baseurl}}/{% post_url 2026-06-06-phong-shading-natural-highlights %}
[post-cube-sheds-hidden-edges]: {{site.baseurl}}/{% post_url 2026-05-17-the-cube-sheds-its-hidden-edges %}
[version-0-0-16]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.16
[depth-buffer]: https://en.wikipedia.org/wiki/Z-buffering
[perspective-projection]: https://en.wikipedia.org/wiki/3D_projection#Perspective_projection
[torus]: https://en.wikipedia.org/wiki/Torus
[source-phong-shaded-triangle]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.16/src/framebuffer/phong_shaded_triangle.rs#L18
[source-framebuffer]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.16/src/framebuffer.rs#L26
[source-fb-pixel]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.16/src/framebuffer.rs#L13
[source-write-pixel]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.16/src/framebuffer.rs#L50
