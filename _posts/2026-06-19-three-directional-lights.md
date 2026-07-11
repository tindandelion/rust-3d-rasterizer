---
layout: post
title: "Three Lights Are Better Than One"
date: 2026-06-19 12:00:00 +0200
authors: Sergey and Cursor
---

In the [previous step][post-materials-colors-stage] we promised to bring our scene closer to the [three.js Geometry Browser][geometry-browser] torus demo, and that demo is lit by _three_ directional lights, not one. So this milestone is about teaching our renderer to handle several light sources at once.

[Version 0.1.1 on GitHub][version-0-1-1]{: .no-github-icon}

## What you will see

Up to now every render was lit by a single directional light coming from one side. With three lights placed around the torus, more of its surface catches some light, and the shading reads as a softer, more sculpted shape. Compare the old single-light render with the new three-light one:

<div class="still-compare">
<figure>
<img src="https://github.com/tindandelion/rust-3d-rasterizer/releases/download/0.1.0/scene.webp" alt="Torus lit by a single directional light" />
<figcaption>Before: One directional light</figcaption>
</figure>
<figure>
<img src="https://github.com/tindandelion/rust-3d-rasterizer/releases/download/0.1.1/scene.webp" alt="Torus lit by three directional lights" />
<figcaption>Now: Three directional lights</figcaption>
</figure>
</div>

The torus itself, the camera, and the material are the same. Only the lighting setup changed.

## Lights simply add up

The pleasant surprise here is that supporting many lights requires almost no new theory. Light obeys the [_superposition principle_][superposition]: the illumination from several sources is just the sum of the illumination each source would produce on its own. There is no special blending rule — two lights hitting the same spot are simply brighter than either one alone.

This is exactly where the [linear color space][post-materials-colors-stage-linear] work from the previous milestone pays off. Adding intensities only makes physical sense when the numbers we add are _actual_ intensities, not sRGB-encoded bytes. Because we already do all shading math in linear space, summing contributions from multiple lights is correct by construction.

Recall the single-light equation from last time. For a light with incoming intensity $I_L$, surface normal $\mathbf{n}$, direction toward the light $\mathbf{l}$, and Blinn half-vector $\mathbf{h}$:

$$
\begin{aligned}
k_d &= I_L \cdot \max(\mathbf{l}\cdot\mathbf{n}, 0) \\
k_s &= I_L \cdot \max(\mathbf{n}\cdot\mathbf{h}, 0)^s \\
\mathbf{C}_{out} &= \mathbf{C}_{emissive} + k_d\,\mathbf{C}_{diffuse} + k_s\,\mathbf{C}_{specular}
\end{aligned}
$$

To support a set of lights, we compute the diffuse and specular factors $k_{d,i}$ and $k_{s,i}$ for each light $i$ and sum them. The material colors are constant, so we can pull them out of the sum:

$$
\mathbf{C}_{out} = \mathbf{C}_{emissive}
  + \left(\sum_i k_{d,i}\right)\mathbf{C}_{diffuse}
  + \left(\sum_i k_{s,i}\right)\mathbf{C}_{specular}
$$

One detail worth calling out: the _emissive_ term is added **once**, not per light. Emissive color is light the material gives off by itself, independent of any source, so it has no business being multiplied by the number of lights in the scene.

## Implementation

Recall that [directional lights][post-cube-gets-light] are fully specified by the direction vector. We take these vectors straight from the Geometry Browser's scene setup. With adjustments to coordinate systems — `three.js` uses a [right-handed][handedness] coordinate system with $+\mathrm{Z}$ pointing back toward the viewer, while our renderer is left-handed with $+\mathrm{Z}$ pointing forward into the scene — the light directions become 

$$
(0, 2, 0) \quad (1, 2, -1) \quad (-1, -2, 1)
$$

The change in code to support multiple lights is quite small. Basically, the only substantial change has been made to [`Material::shade`][source-shade], which now takes a slice of `DirectionalLight` values and calculates the pixel's color value using the formula from above. And that's basically it!


## What's next

With multiple lights in place, the obvious follow-ups are [_point lights_][point-light] — sources at a finite position whose direction and intensity vary across the surface — and [perspective projection][perspective-projection], which we've been postponing. We spent some time sketching what a `PointLight` would require, and it turns out the two are intertwined: point lights want per-fragment world positions, which perspective projection touches as well. 

We're going to go on now with implementing [point lights][post-point-lights].


[post-point-lights]: {{site.baseurl}}/{% post_url 2026-07-11-introducing-point-lights %}
[post-materials-colors-stage]: {{site.baseurl}}/{% post_url 2026-06-19-materials-colors-and-the-stage %}
[post-cube-gets-light]: {{site.baseurl}}/{% post_url 2026-05-22-the-cube-gets-light %}#the-light-source-directional-light
[post-materials-colors-stage-linear]: {{site.baseurl}}/{% post_url 2026-06-19-materials-colors-and-the-stage %}#discovering-linear-color-space
[version-0-1-1]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.1.1
[source-shade]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.1/src/lighting.rs#L29
[geometry-browser]: https://threejs.org/docs/scenes/geometry-browser.html
[superposition]: https://en.wikipedia.org/wiki/Superposition_principle
[handedness]: https://en.wikipedia.org/wiki/Cartesian_coordinate_system#Orientation_and_handedness
[point-light]: https://en.wikipedia.org/wiki/Shading#Point_lighting
[perspective-projection]: https://en.wikipedia.org/wiki/3D_projection#Perspective_projection
