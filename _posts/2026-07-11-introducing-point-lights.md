---
layout: post
title: "Introducing Point Lights"
date: 2026-07-11 10:00:00 +0200
authors: Sergey and Cursor
---

At the beginning of the project, we introduced [directional light sources][post-cube-gets-light]. Now we're ready to add a different type of light source: _point lights_. As we'll explore in this post, this addition also requires us to make changes to the rasterizer on the lowest level, because for the point lights we need to map each pixel of the frame buffer to the coordinates in the world space. 

[Version 0.1.6 on GitHub][version-0-1-6]{: .no-github-icon}

A quick note on the version jump. The releases between the previous post and this one (0.1.2 through 0.1.4) were related to some house-keeping and the CI/CD pipeline. None of them changed the picture, so we skip straight from three directional lights to point lights.

## What you will see

The torus scene is now lit by a _mix_ of light types: where the [previous version][post-three-lights] used three directional lights of equal strength, the export now uses one directional light from above plus two _point lights_ from the corners. Here is the tumbling torus under the new setup:

<div style="text-align: center;">
<video src="https://github.com/tindandelion/rust-3d-rasterizer/releases/download/0.1.6/scene.webm" alt="Torus lit by one directional and two point lights" autoplay loop muted playsinline
  width="800" style="max-width: 100%; padding-bottom: 1em; padding-top: 1em"></video>
</div>

To be fair, the animation video doesn't show big difference from the previous renders: with three light sources and the complex surface it's not clearly visible what has changed. To see the differences better, let's look at the still image of a torus, lit by a single light, one with directional, and another with the point light: 

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

When we [introduced the directional lights][link?], we said that the real-world equivalent for it is our sun: it's located very far away from us, so that all light rays come in parallel from the same direction. 

A point light can be said to emulate an incadescent light bulb in the dark room. With some simplifications, we can say that a light bulb emits light in all directions from a fixed position in the scene. Light rays are no longer parallel: instead, they form a radial pattern emanating outward from a single point. Because of that, they hit the flat surface at different angles at different locations: 

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

When it comes to the lighting model, we still use our fmailiar Phong lighting equations. Remember that for directional lights, the light vector $\mathbf{l}$ was the same for each surface point. With point lights, it's no longer a constant: now we need to calculate the value of $\mathbf{l}$ for each surface point $\mathbf{p}$ separately: 

$$
\mathbf{l}(\mathbf{p}) = \frac{\mathbf{p}_{\text{light}} - \mathbf{p}}{\lVert \mathbf{p}_{\text{light}} - \mathbf{p} \rVert}
$$

That's the biggest difference that makes point lights distinct from directional ones. 

Another distinct property is that point lights are a subject to _distance falloff_: the further away we are from the light, the less energy we receive. In real world, the energy is inverse proportional to the square of the distance to the light. Computer graphics, however, adopted a slightly different approach to calculate the falloff, to compensate to the fact that we use approximate models of physical lights. 

There are different formulas to calculate the falloff. In our project, we model it with the classic polynomial [_attenuation_][inverse-square] curve. For a fragment at distance $d$ from the light, the contribution is scaled by:

$$
f(d) = \frac{1}{k_c + k_l\, d + k_q\, d^2}
$$

The three coefficients — constant, linear, and quadratic — dial the shape of the falloff curve, from no falloff at all ($k_c = 1$, the rest zero) to a physically flavoured inverse-square drop dominated by $k_q$. The picture above demonstrates the falloff behaviour with $k_q > 0$: the closer the light is moved to the surface, the brighter the lit spot becomes. 

## Interpolation of surface points 

So now we know how to implement point lights. There's a missing piece, though, that we need to address. 

As we discussed above, a point light needs to know the world-space position (i.e., its coordinates) of each point on the surface, to calculate the light vector. However, we don't have this information immediately available. Recall that we represent our shapes as a trianglular mesh, where we have the coordiantes of vertices, but we don't have the coordinates of _every_ point on the shape's surface. How can we get round this limitation? 

Fortunately, the solution is quite familiar at this stage: linear interpolation. Recall that we had a similar problem before with surface normals, when we first started with [Phong shading][link-to-post?]. In order to calculate the illumination of each point, we had to know the normal to the surface at that point, but we only had normal values for mesh vertices. We solved it back then by interpolating the intermediate normal values between vertices. 

Now that we need point positions as well, we can do exactly the same interpolation technique, using vertex positions as anchor values. Our `PhongShadedTriangle` now needs to interpolate two distinct values for each framebuffer pixel: 
* the surface normal - as we did before; 
* the world position - the new addition to support point lights, 

and that give us all necessary inputs to calculate the illumination of that pixel. 

Implementation-wise, we've noticed that the point position and the normal are very frequently used together in the code, so to make the code cleaner we've decided to bundle them into a a new geomery type, [`SurfacePoint`][source-surface-point]. This type also makes reasoning about the interpolation a bit easier: now we can say that we interpolate `SurfacePoint`s, which includes both the position and the normal. 


## Implementation

The lighting types were reshaped around a single [`Light`][source-light] type with two constructors, [`Light::directional`][source-light-directional] and [`Light::point`][source-light-point]; the old standalone `DirectionalLight` retired into it. A private [`factors`][source-factors] helper hides the branch — for a directional light it returns the stored direction and a falloff of 1; for a point light it computes both the direction $\mathbf{p}_{\text{light}} - \mathbf{p}$ and the distance falloff from that same displacement — so the diffuse and specular routines stay identical for both kinds and simply multiply by whatever falloff comes back.

Carrying position through the rasterizer needed a small new geometry type, [`SurfacePoint`][source-surface-point], bundling a world position and a normal. It implements the arithmetic operators the scanline interpolator expects, so [`PhongShadedTriangle`][source-phong] now interpolates a whole `SurfacePoint` per pixel instead of just a normal. [`Material::shade`][source-shade] takes that `SurfacePoint` and hands it to each light.

The falloff itself lives in a small [`DistanceFalloff`][source-distance-falloff] struct holding the three coefficients. The export recipe in [`default_lights`][source-default-lights] uses the mix: one directional light toward $(0, 2, 0)$ at intensity 0.5, plus two point lights at $(1, 2, -1)$ and $(-1, -2, 1)$ — the same browser-derived positions as before, but now interpreted as places rather than directions. Each point light carries a quadratic falloff and a boosted intensity of 3.0 to make up for the light it now loses to distance. The standalone [`point-light`][source-point-light-bin] binary renders the ground-plane demo.

Two design notes worth remembering. First, the per-fragment view direction $\mathbf{t}$ is still a constant across the frame — under our orthographic camera the eye is effectively at infinity, so `toward_eye` remains $-\mathbf{camera.direction()}$ for every pixel, point light or not. That will have to change alongside perspective projection. Second, there is a genuine singularity when a point light sits exactly on a surface: $\mathbf{p}_{\text{light}} - \mathbf{p}$ becomes the zero vector and cannot be normalized. We know about it and left it as a loud panic rather than papering over it, since it only arises from a degenerate scene.

## What's next

The two obvious follow-ons remain [perspective projection][perspective-projection] — still an optional stretch, and the piece that will finally make `toward_eye` vary per fragment — and, after that, a pass at aligning our shading equation more closely with the three.js reference.

[post-three-lights]: {{site.baseurl}}/{% post_url 2026-06-19-three-directional-lights %}
[post-cube-gets-light]: {{site.baseurl}}/{% post_url 2026-05-22-the-cube-gets-light %}#the-light-source-directional-light
[version-0-1-6]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.1.6
[source-light]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.6/src/lighting/light.rs#L5
[source-light-directional]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.6/src/lighting/light.rs#L55
[source-light-point]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.6/src/lighting/light.rs#L62
[source-factors]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.6/src/lighting/light.rs#L42
[source-distance-falloff]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.6/src/lighting/light.rs#L11
[source-surface-point]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.6/src/geometry/surface_point.rs#L10
[source-phong]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.6/src/framebuffer/phong_shaded_triangle.rs#L28
[source-shade]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.6/src/lighting.rs#L32
[source-default-lights]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.6/src/lib.rs#L59
[source-point-light-bin]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.6/src/bin/point-light.rs#L51
[point-light]: https://en.wikipedia.org/wiki/Shading#Point_lighting
[lambert]: https://en.wikipedia.org/wiki/Lambertian_reflectance
[inverse-square]: https://en.wikipedia.org/wiki/Inverse-square_law
[perspective-projection]: https://en.wikipedia.org/wiki/3D_projection#Perspective_projection
