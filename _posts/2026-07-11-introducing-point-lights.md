---
layout: post
title: "Introducing Point Lights"
date: 2026-07-11 10:00:00 +0200
authors: Sergey and Cursor
---

At the beginning of the project, we introduced [directional light sources][post-cube-gets-light]. Now we're ready to add a different type of light source: _point lights_. As we'll explore in this post, this addition also requires us to make changes to the rasterizer at the lowest level, because for point lights we need to map each pixel of the framebuffer to its coordinates in world space.

[Version 0.1.6 on GitHub][version-0-1-6]{: .no-github-icon}

A quick note on the version jump. The releases between the previous post and this one (0.1.2 through 0.1.4) were related to some house-keeping and the CI/CD pipeline. None of them changed the picture, so we skip straight to point lights.

## What you will see

The torus scene is now lit by a _mix_ of light types: where the [previous version][post-three-lights] used three directional lights of equal strength, the export now uses one directional light from above plus two _point lights_ from the corners. Here is the tumbling torus under the new setup:

<div style="text-align: center;">
<video src="https://github.com/tindandelion/rust-3d-rasterizer/releases/download/0.1.6/scene.webm" alt="Torus lit by one directional and two point lights" autoplay loop muted playsinline
  width="800" style="max-width: 100%; padding-bottom: 1em; padding-top: 1em"></video>
</div>

To be fair, the animation video doesn't show a big difference from the previous renders: with three light sources and the complex surface it's not clearly visible what has changed. To see the differences better, let's look at still images of a torus lit by a single light, one directional and the other being a point light: 

<div class="still-compare">
<figure>
<img src="{{ "/assets/images/2026-07-11-introducing-point-lights/still-scene-dir.webp" | relative_url }}" alt="Directional light" />
<figcaption>Directional light</figcaption>
</figure>
<figure>
<img src="{{ "/assets/images/2026-07-11-introducing-point-lights/still-scene-point.webp" | relative_url }}" alt="Point light" />
<figcaption>Point light</figcaption>
</figure>
</div>

Now the difference in lighting is clearly visible. Let's explore what makes the point light different from the directional one. 

## Introducing point lights

When we [started to work with directional lights][post-cube-gets-light], we said that their real-world equivalent is the sun: it's located very far away from us, so that all light rays come in parallel from the same direction. To continue with real-world analogies, a _point light_ is an emulation of an incandescent light bulb in a dark room. 

Roughly, we can say that a light bulb emits light in all directions from a fixed position inside the scene. Light rays are no longer parallel: instead, they form a radial pattern emanating outward from a single source point. Because of that, they hit the flat surface at different angles at different locations: 

<div class="still-compare">
<figure>
<img src="{{ "/assets/images/2026-07-11-introducing-point-lights/point-light-1.webp" | relative_url }}" alt="Ground plane lit by a point light high above the surface" />
<figcaption>Light high above the plane</figcaption>
</figure>
<figure>
<img src="{{ "/assets/images/2026-07-11-introducing-point-lights/point-light-2.webp" | relative_url }}" alt="Ground plane lit by a point light lowered closer to the surface" />
<figcaption>Light lowered closer</figcaption>
</figure>
<figure>
<img src="{{ "/assets/images/2026-07-11-introducing-point-lights/point-light-3.webp" | relative_url }}" alt="Ground plane lit by a point light just above the surface, with a tight bright hotspot" />
<figcaption>Light just above the surface</figcaption>
</figure>
</div>

When it comes to the lighting model, we still use our familiar Phong lighting equations. Remember that for directional lights, the light vector $\mathbf{l}$ was the same for each surface point. With point lights, it's no longer a constant: now we need to calculate the value of $\mathbf{l}$ for each surface point $\mathbf{p}$ separately: 

$$
\mathbf{l}(\mathbf{p}) = \frac{\mathbf{p}_{\text{light}} - \mathbf{p}}{\lVert \mathbf{p}_{\text{light}} - \mathbf{p} \rVert}
$$

That's the biggest difference that makes point lights distinct from directional ones. 

Another distinct property is that point lights are subject to _distance falloff_: the further away we are from the light, the less energy we receive. In the real world, the energy is inversely proportional to the square of the distance to the light. Computer graphics, however, adopted a slightly different approach to calculate the falloff, to compensate for the fact that we use approximate models of physical lights. 

There are different formulas to calculate the falloff. In our project, we model it with the classic polynomial [_attenuation_][inverse-square] curve. For a fragment at distance $d$ from the light, the contribution is scaled by:

$$
f(d) = \frac{1}{k_c + k_l\, d + k_q\, d^2}
$$

The three coefficients — constant, linear, and quadratic — dial the shape of the falloff curve, from no falloff at all ($k_c = 1$, the rest zero) to a physically flavoured inverse-square drop dominated by $k_q$. The picture above demonstrates the falloff behaviour with $k_q > 0$: the closer the light is moved to the surface, the brighter the lit spot becomes. 

## Interpolation of surface points 

So now we know how to implement point lights from the lighting model perspective. There's a missing piece, though, that we need to address. 

As we discussed above, a point light needs to know the world-space position (i.e., its coordinates) of each point on the surface, to calculate the light vector $\mathbf{l}$. However, we don't have this information immediately available. Recall that we represent our shapes as a triangular mesh, where we have the coordinates of vertices, but we don't have the coordinates of _every_ point on the shape's surface. How can we get around this limitation? 

Fortunately, the solution is quite familiar at this stage: linear interpolation. Recall that we had a similar problem before with surface normals, when we first started with [Phong shading][post-phong-shading]. In order to calculate the illumination of each point, we had to know the normal to the surface at that point, but we only had normal values for mesh vertices. We solved it back then by interpolating the intermediate normal values between vertices. 

Now that we need point positions as well, we can apply exactly the same interpolation technique, using vertex positions as anchor values. Our `PhongShadedTriangle` now needs to interpolate two distinct values for each framebuffer pixel: 
* the surface normal - as we did before; 
* the world position - the new addition to support point lights, 

and that gives us all the necessary inputs to calculate the illumination of that pixel. 

Implementation-wise, we've noticed that the point position and the normal are very frequently used together in the code, so to make the code cleaner we've decided to bundle them into a new geometry type, [`SurfacePoint`][source-surface-point]. This type also makes reasoning about the interpolation a bit easier: now we can say that we interpolate `SurfacePoint`s, which includes both the position and the normal. 

## Light implementation 

Up until now, we used a struct [`DirectionalLight`][source-directional-light] to represent a directional light source. Now we deprecate this data type, and introduce a new one, [`Light`][source-light], that handles both directional and point lights. 

The instances of `Light` can be created by means of two constructors, [`Light::directional`][source-light-directional] and [`Light::point`][source-light-point]. The public interface of the `Light` type is nearly identical to the old `DirectionalLight`, with one notable extension: instead of passing a `normal` vector, we're now passing an instance of `SurfacePoint` that carries both the normal and the position, so that we can use that data to handle both light types. 

Finally, to showcase the new abilities, we change the light setup for the [animated scene binary][source-animated-scene]. From now on, we're going to use a heterogeneous light setup: 

* One directional light above the torus toward $(0, 2, 0)$ lights the shape from above; 
* Two point lights placed at $(1, 2, -1)$ and $(-1, -2, 1)$ light the shape from the sides, above and below the shape, respectively. 

This is roughly the same light setup that three.js Geometry Browser uses, except that our setup now contains different light types. 

## What's next

Frankly speaking, at this point it feels that we've finished all our major goals [planned for phase 2][post-planning-phase-2]. One notable milestone we haven't approached yet is [perspective projection][perspective-projection]. 

Even though introducing perspective projection would affect the rendering pipeline code, it doesn't look like it's going to change the visual result dramatically. At the moment, we're tempted to push this milestone further down the line and do something more interesting. 

[post-three-lights]: {{site.baseurl}}/{% post_url 2026-06-19-three-directional-lights %}
[post-cube-gets-light]: {{site.baseurl}}/{% post_url 2026-05-22-the-cube-gets-light %}#the-light-source-directional-light
[post-phong-shading]: {{site.baseurl}}/{% post_url 2026-06-06-phong-shading-natural-highlights %}
[post-planning-phase-2]: {{site.baseurl}}/{% post_url 2026-06-17-planning-phase-2 %}
[version-0-1-6]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.1.6
[source-directional-light]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.1/src/lighting.rs#L51
[source-light]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.6/src/lighting/light.rs#L5
[source-light-directional]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.6/src/lighting/light.rs#L55
[source-light-point]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.6/src/lighting/light.rs#L62
[source-surface-point]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.6/src/geometry/surface_point.rs#L10
[source-animated-scene]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.6/src/bin/animated-scene.rs
[inverse-square]: https://en.wikipedia.org/wiki/Inverse-square_law
[perspective-projection]: https://en.wikipedia.org/wiki/3D_projection#Perspective_projection
