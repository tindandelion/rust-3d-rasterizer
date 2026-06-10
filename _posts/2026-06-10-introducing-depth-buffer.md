---
layout: post
title: "Solving Occlusion with Depth Buffer"
date: 2026-06-10 10:00:00 +0200
authors: Sergey and Cursor
---

With [Phong shading][post-phong-shading-natural-highlights] in place, we're almost ready to finish Phase 1: rendering a torus. One last piece of the puzzle is still missing, though. A torus is more complicated than anything we've rendered so far. In particular, it is _self-occluding:_ parts of the surface can hide other parts of the same object from the camera. To make the torus look realistic, we need to solve occlusion first.

[Version 0.0.16 on GitHub][version-0-0-16]{: .no-github-icon}

## What you will see

Our rasterizer can now render objects that occlude each other. 

To demonstrate this, we've changed the animated scene. You'll now see two spheres sitting next to each other. As the camera circles the scene, one sphere passes behind the other. Overlapping shapes like these are handled by a common technique called the [_depth buffer_][depth-buffer].

![Two Phong-shaded spheres with correct occlusion as the camera orbits](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.16/doc/output/current.webp)


## The motivation

In the past, we already dealt with the hidden parts of the scene: remember that we had to deal with [back-facing facets][post-cube-sheds-hidden-edges] way back when we still learned how to render a cube. Then we solved the problem with pure geometry, detecting and dropping back-facing facets based on their orientation with respect to the view direction.

That technique works fine for single convex shapes, such as a cube or a sphere. However, it doesn't work when front-facing surfaces can still block one another — on a torus, parts of the tube pass in front of other parts of the same mesh, and orientation alone cannot tell us which pixel should win.

The same problem occurs when we start rendering more than one shape in the scene: from the viewer's perspective, parts of one shape can be partially hidden behind the other. 

For now, we use two spheres in the scene to motivate the enhancement of our rasterizer. 

## Depth buffer

Conceptually, a depth buffer is a very simple idea: for each pixel in the frame buffer, along with the color, we'll store a single number that represents its depth. In the simplest case, the bigger that number, the deeper in the scene that pixel lies.

Initially, the depth buffer is filled with a sentinel value representing $+\infty$. When we write a pixel to the frame buffer, we also record its depth. Now we can perform a _depth test_: if the new depth is smaller than the stored one, we overwrite both the color and the depth. Otherwise, we keep the existing values.

Implementation-wise, the depth buffer is an array of numbers, one entry per pixel. It costs additional memory, but the appeal of this technique is the ease of implementation.

## Using _z_ coordinate as depth

As simple as the depth-buffer idea is, real implementations have plenty of details to sort out. We're not diving into those yet: our setup is simple enough to use the $z$ coordinate directly as a depth value.

The only challenge was computing the $z$ value for each pixel. We use our familiar tool for that: linear interpolation. Just as we interpolate `(x, y)` coordinates across a facet from its vertices, we can interpolate the `z` coordinate the same way. The [`PhongShadedTriangle`][source-phong-shaded-triangle] type already handled all the interpolation work for us; we only had to extend it to interpolate one more value.

## This is still work in progress

As noted above, our depth-buffer implementation is deliberately simple. We'll revisit it later when the project outgrows this approach. Introducing [perspective projection][perspective-projection] will likely require that revisit, because that projection transforms coordinates in ways that make raw $z$ values less reliable as depth.


## What's next

It seems that with the occlusion problem solved, we're finally ready to render and see the [_torus_][torus] shape in all its splendor!

[post-phong-shading-natural-highlights]: {{site.baseurl}}/{% post_url 2026-06-06-phong-shading-natural-highlights %}
[post-cube-sheds-hidden-edges]: {{site.baseurl}}/{% post_url 2026-05-17-the-cube-sheds-its-hidden-edges %}
[version-0-0-16]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.16
[depth-buffer]: https://en.wikipedia.org/wiki/Z-buffering
[perspective-projection]: https://en.wikipedia.org/wiki/3D_projection#Perspective_projection
[torus]: https://en.wikipedia.org/wiki/Torus
[source-phong-shaded-triangle]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.16/src/framebuffer/phong_shaded_triangle.rs#L15
