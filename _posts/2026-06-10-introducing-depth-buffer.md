---
layout: post
title: "Introducing Depth Buffer"
date: 2026-06-10 10:00:00 +0200
authors: Sergey and Cursor
---

With [Phong shading][post-phong-shading-natural-highlights], we're almost ready to complete our goal of the phase 1: rendering a torus shape. One last piece of the puzzle is missing, though: a torus is a more complicated shape than what we've dealt before. In particular, it is _self-occluding_: [todo: explain what it means]. To make the torus look realistic, we need to solve the occlusion problem first. 

[Version 0.0.16 on GitHub][version-0-0-16]{: .no-github-icon}

## What you will see

The new ability of our rasterizer is that it can now render objects that occlude each other. To demonstrate this, we've changed the animated scene. 

You'll see now two spheres sitting next to each other. As the camera makes a circle around the scene, one of the spheres gets hidden behind another. As this demo presents, we can now render the shapes that overlap each other, thanks to the technique called [_depth buffer_][depth-buffer]. 

![Two Phong-shaded spheres with correct occlusion as the camera orbits](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.16/doc/output/current.webp)


## The motivation 

In the past, we already dealt with the hidden parts of the scene: remember that we had to deal with [back-facing facets][link to the post] way back when we learned how to render a cube. Then we solved the problem with pure geometry, detecting and dropping back-facing facets based on their orientation with respect to the view direction. 

That technique works fine for single convex shapes, such as a cube or a sphere. However, it doesn't work for the complex shapes where [complete this sentence please]

The same problem occurs when we start rendering ore than one shape in the scene: from the viewer's perspecrive, parts of one shape can be partially hidden behind the other. 

## Depth buffer

Conceptually, a depth buffer is a very simple idea: for each pixel in the frame buffer, along with the color, we'll store a single number that represents its depth. In the simplest case, the bigger that number, the deeper in the scene that pixel is. 

At the beginning, the, the depth buffer is filled with some initial value: $+inf$. When we write a pixel to the frame buffer, along with the color we pass its new depth value. Now we can perform a _depth test_: if the new depth value is smaller than the stored one, we can overwrite that pixel. Otherwise, we retain the old value for the pixel's color and depth. 

Implementation-wise, the depth buffer is an array of numbers, one entry per each pixel. It costs additional memory, but the appeal of this techique is the ease of implementation. 

## Using _z_ coordinate as depth 

As simple as a depth buffer idea is, there's a bunch of implementation-specific details to be taken care of. We're not diving into those details yet: our setup is simple enough to use $z$ coordinate directly as a depth value. 

The only challenge we had was to calculate $z$ value for each pixel. To do that, we use our familiar tool: linear interpolation. Just as we interpolate `(x, y)` coordinates of the facet interior pixels using vertex coordinates, in the same manner we can interpolate `z` coordinate. Our adjusted implementation of [`PhongShadedTriangle`][source-phong-shaded-triangle], that already does all interpolation work, is now charged with inerpolating one more value, and that's it. 

## This is still work in progress

As we've mentioned above, our implementation of the depth buffer is deliberately simple. We're going to keep an eye on it and will get back to it later, when the progression of our project starts demanding the change in that area. Specifically, it seems that introducing [perspective projection][perspective-projection] will require revisiting the implementation of the depth buffer, because of the way this projection transforms coordinates. 


## What's next

It seems that with the occlusion problem solved, we're finally ready to render and see the [_torus_][torus] shape in all its splendor! 

[post-phong-shading-natural-highlights]: {{site.baseurl}}/{% post_url 2026-06-06-phong-shading-natural-highlights %}
[version-0-0-16]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.16
[depth-buffer]: https://en.wikipedia.org/wiki/Z-buffering
[painters-algorithm]: https://en.wikipedia.org/wiki/Painter%27s_algorithm
[perspective-projection]: https://en.wikipedia.org/wiki/3D_projection#Perspective_projection
[torus]: https://en.wikipedia.org/wiki/Torus
[source-framebuffer]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.16/src/framebuffer.rs#L25
[source-fb-pixel]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.16/src/framebuffer.rs#L12
[source-clear]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.16/src/framebuffer.rs#L43
[source-write-pixel]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.16/src/framebuffer.rs#L49
[source-camera-transform]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.16/src/ortho_camera.rs#L95
[source-phong-shaded-triangle]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.16/src/framebuffer/phong_shaded_triangle.rs#L15
[source-phong-corner]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.16/src/framebuffer/phong_shaded_triangle.rs#L12
[source-shape-render]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.16/src/lib.rs#L47
[source-occlusion-test]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.16/tests/draw_unit_cube.rs#L33
