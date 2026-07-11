---
layout: post
title: "Introducing Point Lights"
date: 2026-07-11 10:00:00 +0200
authors: Sergey and Cursor
---

At the beginning of the project, we introduced [directional light sources][post-cube-gets-light]. Now we're ready to add a different type of light source: _point light_. As we'll explore in this post, this addition requires us to make changes to the rasterizer on the lowest level, because for the point lights we need to map each pixel of the frame buffer to the coordinates in the world space. 

[Version 0.1.5 on GitHub][version-0-1-5]{: .no-github-icon}

A quick note on the version jump. The releases between the previous post and this one (0.1.2 through 0.1.4) were related to some house-keeping and the CI/CD pipeline. None of them changed the picture, so we skip straight from three directional lights to point lights.

## What you will see

The torus scene is now lit by a _mix_ of light types: where [previous version][link-to-prev-post] used three directional lights of equal strength, the export now uses one directional light from above plus two _point lights_ from the corners. Here is the tumbling torus under the new setup:

<div style="text-align: center;">
<video src="https://github.com/tindandelion/rust-3d-rasterizer/releases/download/0.1.5/scene.webm" alt="Torus lit by one directional and two point lights" autoplay loop muted playsinline
  width="800" style="max-width: 100%;"></video>
</div>

To be fair, the difference from the earlier renders is very subtle: with three lights and the complex surface it's not that evident what has changed. To explore the difference between the directional and point light sources, we've added a small separate showcase: the additional binary `point-light` that allows us to play with light settings and see the lighting effects on a simple horizontal plane. 

## The difference between directional and point light sources

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

## Directional lights live at infinity

Recall how a [_directional light_][post-cube-gets-light] works. It has no position — only a direction. Every point on every surface receives light coming from the same fixed angle, as if the source were infinitely far away. That is a good model for the sun: across a room, or across a torus, the sun's rays are effectively parallel.

That assumption is exactly what made the shading math cheap. To light a fragment, the renderer only needed its surface normal $\mathbf{n}$; the direction toward the light $\mathbf{l}$ was a constant carried by the light itself.

## A point light has an address

A [_point light_][point-light] is different: it emits from a specific position $\mathbf{p}_{\text{light}}$ in the scene, radiating outward in all directions. Now the direction toward the light is no longer a shared constant — it depends on _where you are_. For a fragment at world position $\mathbf{p}$:

$$
\mathbf{l}(\mathbf{p}) = \frac{\mathbf{p}_{\text{light}} - \mathbf{p}}{\lVert \mathbf{p}_{\text{light}} - \mathbf{p} \rVert}
$$

A directional light is really just the limiting case of this, where $\mathbf{p}_{\text{light}}$ recedes to infinity and $\mathbf{l}$ stops depending on $\mathbf{p}$. That symmetry is worth holding onto: once we have $\mathbf{l}$ for a fragment, everything downstream is identical. The [Lambertian diffuse][lambert] term and the Blinn half-vector specular term from the previous milestones don't change at all. Only the way we _obtain_ $\mathbf{l}$ is new.

## The renderer suddenly needs to know where each pixel is

Here is the crux of the milestone, and it is more about plumbing than about lighting theory. With directional lights, the shader only ever saw a surface normal. A point light forces a new piece of information all the way down the pipeline: the fragment's **world-space position**.

That position has to be interpolated per pixel, exactly the way normals already were. Each triangle vertex knows its world position; as the rasterizer walks the scanlines it blends those positions across the triangle, so every fragment can compute its own $\mathbf{l}$ toward each point light.

## What we are not doing yet: distance falloff

Real point lights also get dimmer the farther you are from them, following an inverse-square [_attenuation_][inverse-square] law. We deliberately left that out of this first pass. Our point light varies only in _direction_ with position, not in _intensity_ — a light one unit away and a light ten units away illuminate an aligned surface equally.

We did sketch how attenuation would slot in (a small distance-based multiplier applied to both the diffuse and specular terms, with directional lights pinned at 1.0), but chose to keep the first cut simple, as the plan asked. It becomes worth adding the moment a scene actually needs a light to feel local. So the ground-plane hotspot you see in the showcase is pure cosine falloff from the changing angle, not distance falloff.

## Implementation

The lighting types were reshaped around a single [`Light`][source-light] enum with two constructors, [`Light::directional`][source-light-directional] and [`Light::point`][source-light-point]; the old standalone `DirectionalLight` type retired into it. A private [`toward_light`][source-toward-light] helper hides the branch — it returns the stored direction for a directional light, or computes $\mathbf{p}_{\text{light}} - \mathbf{p}$ for a point light — so the diffuse and specular routines stay identical for both kinds.

Carrying position through the rasterizer needed a small new geometry type, [`SurfacePoint`][source-surface-point], bundling a world position and a normal. It implements the arithmetic operators the scanline interpolator expects, so [`PhongShadedTriangle`][source-phong] now interpolates a whole `SurfacePoint` per pixel instead of just a normal. [`Material::shade`][source-shade] takes that `SurfacePoint` and hands it to each light.

The export recipe in [`default_lights`][source-default-lights] changed to demonstrate the mix: one directional light toward $(0, 2, 0)$ at intensity 0.5, plus two point lights at $(1, 2, -1)$ and $(-1, -2, 1)$ at intensity 1.0 each — the same browser-derived positions as before, but two of them now interpreted as places rather than directions. The standalone [`point-light`][source-point-light-bin] binary renders the ground-plane demo.

Two design notes worth remembering. First, the per-fragment view direction $\mathbf{t}$ is still a constant across the frame — under our orthographic camera the eye is effectively at infinity, so `toward_eye` remains $-\mathbf{camera.direction()}$ for every pixel, point light or not. That will have to change alongside perspective projection. Second, there is a genuine singularity when a point light sits exactly on a surface: $\mathbf{p}_{\text{light}} - \mathbf{p}$ becomes the zero vector and cannot be normalized. We know about it and left it as a loud panic rather than papering over it, since it only arises from a degenerate scene.

## What's next

The two obvious follow-ons remain [perspective projection][perspective-projection] — still an optional stretch, and the piece that will finally make `toward_eye` vary per fragment — and, after that, a pass at aligning our shading equation more closely with the three.js reference. Distance attenuation is now a small, well-understood addition whenever a scene calls for a light that feels genuinely local.

[post-three-lights]: {{site.baseurl}}/{% post_url 2026-06-19-three-directional-lights %}
[post-cube-gets-light]: {{site.baseurl}}/{% post_url 2026-05-22-the-cube-gets-light %}#the-light-source-directional-light
[version-0-1-5]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.1.5
[source-light]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.5/src/lighting/light.rs#L10
[source-light-directional]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.5/src/lighting/light.rs#L16
[source-light-point]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.5/src/lighting/light.rs#L23
[source-toward-light]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.5/src/lighting/light.rs#L44
[source-surface-point]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.5/src/geometry/surface_point.rs#L10
[source-phong]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.5/src/framebuffer/phong_shaded_triangle.rs#L28
[source-shade]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.5/src/lighting.rs#L31
[source-default-lights]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.5/src/lib.rs#L58
[source-point-light-bin]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.5/src/bin/point-light.rs#L44
[point-light]: https://en.wikipedia.org/wiki/Shading#Point_lighting
[lambert]: https://en.wikipedia.org/wiki/Lambertian_reflectance
[inverse-square]: https://en.wikipedia.org/wiki/Inverse-square_law
[perspective-projection]: https://en.wikipedia.org/wiki/3D_projection#Perspective_projection
