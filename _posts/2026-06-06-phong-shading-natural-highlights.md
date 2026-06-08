---
layout: post
title: "Phong Shading: Natural Highlights"
date: 2026-06-06 10:00:00 +0200
authors: Sergey and Cursor
---

In [First Shot at Glossy Shapes][post-first-shot-at-glossy-shapes], we extended our lighting model to support glossy materials. Our first attempt was to render the specular highlight on the sphere using the Gouraud shading algorithm. The result was encouraging, but the highlight still looked quite artificial. In this post, we're going to implement a more advanced shading algorithm: [_Phong shading_][phong-shading]. This algorithm produces rasterizations with more natural-looking specular highlights.

[Version 0.0.15 on GitHub][version-0-0-15]{: .no-github-icon}

## What you will see

With Phong shading, the highlight on the sphere looks much more convincing than in our [previous attempt][post-first-shot-at-glossy-shapes]. We no longer see artifacts from the underlying triangular mesh. 

![Animated Phong-shaded sphere with Blinn–Phong specular, camera orbit and vertical squash](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.15/doc/output/current.webp)

## Why Gouraud isn't enough for glossy surfaces

In [First Shot at Glossy Shapes][post-first-shot-at-glossy-shapes], we paired the [_Blinn–Phong_][blinn-phong] lighting model with [Gouraud shading][post-the-sphere-gets-smooth]. That combination is sometimes called _Gouraud specular_, and it was a useful first step to extend the lighting model — but the highlight still looked wrong if you knew what to look for.

The issue is what we interpolate. Gouraud evaluates lighting at the three triangle corners, then linearly blends those scalar intensities across the triangle interior. For [matte surfaces][lambertian], that shortcut works well: across a small facet, brightness tends to change gradually, so interpolating corner values usually produces a smooth matte surface.

Specular reflections are different. In Blinn–Phong model, the highlight comes from a term like $(\mathbf{n} \cdot \mathbf{h})^s$ — a dot product raised to a shininess exponent $s$. That function is sharply peaked: it changes quickly as the normal swings even slightly. Light intensity is no longer "almost linear" across the triangle; it can rise and fall much faster between corners than a straight-line blend would suggest.

So when Gouraud interpolates three corner intensities, it tends to smear the highlight, flatten its peak, or leave faint triangular fingerprints on the surface. A dense sphere mesh hides some of that, but the faceted look returns whenever the highlight is small or the tessellation is coarse. 

Phong shading fixes this by interpolating normals instead, and evaluating the nonlinear lighting model at each pixel.

## From Gouraud shading to Phong shading

Recall the idea of [Gouraud shading][post-the-sphere-gets-smooth] for a triangular mesh:

* calculate the reflected light intensity at each vertex;
* interpolate the intensity along the triangle's edges;
* then interpolate the intensity along each horizontal line that makes up the triangle interior.

Phong shading uses the same interpolation idea, but takes it further: what if instead of light intensities we interpolated the normals themselves? Then, using the approximated normal values, we could calculate the light intensity at each pixel of the triangle's interior.

This algorithm is more involved from a computational standpoint: instead of interpolating a single floating-point number (light intensity), we now need to interpolate three coordinates of the normal vector. We also need to invoke [`BlinnLightModel::calc_intensity()`][source-calc-intensity] at each pixel now, which adds to the computation cost.

However, this cost pays off because we're able to produce much more plausible-looking raster images.

## Implementation notes

We've implemented the Phong shading algorithm in the [`PhongShadedTriangle`][source-phong-shaded-triangle] type. Gouraud shading is still there too, in [`GouraudShadedTriangle`][source-gouraud-shaded-triangle]. If you look at both sources, you'll see that there's a great deal of duplication in the code between these two implementations. It's not surprising: as we've alluded above, both algorithms use a similar interpolation idea.

This is a theme for a future refactoring: extract the shared interpolation algorithm from both implementations. However, at this point it looks a bit premature. Most probably, we'll get rid of Gouraud shading in the near future, so this code duplication won't be an issue.

We've done a bit of refactoring around this area, though, when it comes to the linear interpolation algorithm. Now we have a generic [`Interpolator<T>`][source-interpolator] type that implements linear interpolation for any value type `T`, as long as it supports basic arithmetic operations. This type is reused in the code to calculate the coordinates of rectangle edges (`Interpolator<f32>`), and also to interpolate the normals (`Interpolator<Vec3>`).

## What's next

Phong shading concludes our journey through lighting models and shading algorithms. 

Next we're going to look at a completely different topic: how can we draw a scene that includes several objects that occlude each other? In particular, can we convincingly render two spheres where one of them is partially hidden behind another? 

We can't do it yet, but soon we will!

[post-first-shot-at-glossy-shapes]: {{site.baseurl}}/{% post_url 2026-06-03-first-shot-at-glossy-shapes %}
[post-the-sphere-gets-smooth]: {{site.baseurl}}/{% post_url 2026-05-29-the-sphere-gets-smooth %}
[version-0-0-15]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.15
[phong-shading]: https://en.wikipedia.org/wiki/Phong_shading
[blinn-phong]: https://en.wikipedia.org/wiki/Blinn%E2%80%93Phong_reflection_model
[lambertian]: https://en.wikipedia.org/wiki/Lambertian_reflectance
[depth-buffer]: https://en.wikipedia.org/wiki/Z-buffering
[source-calc-intensity]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.15/src/lighting.rs#L42
[source-phong-shaded-triangle]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.15/src/framebuffer/phong_shaded_triangle.rs#L15
[source-interpolator]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.15/src/framebuffer/interpolator.rs#L3
[source-gouraud-shaded-triangle]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.15/src/framebuffer/gouraud_shaded_triangle.rs#L13
