---
layout: post
title: "Materials, Colors, and the Stage"
date: 2026-06-17 09:00:00 +0200
authors: Sergey and Cursor
---

We're starting the [Phase 2][post-planning-phase-2] with updating the general look to match the color palette of three.js geometry browser. That change makes us revisit our implementation for Phong lighting to make the material configuration more flexible. 

[Version 0.1.0 on GitHub][version-0-1-0]{: .no-github-icon}

## What you will see

The demo render is still the familiar torus scene, but now rendered with different material and background setup: 

![Torus render after Phase 2 material and color pipeline updates](https://github.com/tindandelion/rust-3d-rasterizer/releases/download/0.1.0/scene.webp)

## Revisiting the material setup 

We've introduced the [`Material`][source-material-0-0-13] concept when we first tackled [glossy shading][post-first-shot-at-glossy-shapes]. Back then, it was very simple: just enough to demonstrate the new ability to render specular highlights. Moreover, for a while we lived with a rather awkward code design choices where the material specification and lighting equations were weirdly split between `Shape`, `Material`, and an artificial `BlinnLightModel` data types. 

If we combine all those disjoint pieces together, it's fair to say that we had a simple single-color material setup where the lighting equation was scaling this color according to the formula: 

$$
\begin{aligned}
I &= k_a + k_d \left(\max(\mathbf{l}\cdot\mathbf{n}, 0) + \max(\mathbf{n}\cdot\mathbf{h}, 0)^s\right) \\
\mathbf{C}_{out} &= I \cdot \mathbf{C}_{base}
\end{aligned}
$$

where $\mathbf{l}$ is the unit direction toward the light, $\mathbf{n}$ is the unit surface normal, and $\mathbf{v}$ is the unit direction toward the eye (camera). Here $\mathbf{h}$ is the Blinn half-vector between light and view directions (see our [Blinn-Phong explanation][post-first-shot-at-glossy-shapes-blinn-phong]):

$$
\mathbf{h} = \frac{\mathbf{l} + \mathbf{v}}{\lVert \mathbf{l} + \mathbf{v} \rVert}
$$

### New improved material 

Unlike our previous implementation, now we're moving towards a more flexible material specification. Instead of a single color and a couple of control parameters, we're introducing _different colors_ for each lighting component, matching the `MeshPhongMaterial` from `three.js`: 

* `emissive` is a color of the material that's unaffected by other lighting. Generally speaking, it's the color of the light the object emits by itself, in absence of any light sources. This parameter replaces our previous concept of _ambient color_, serving the same purpose: give some color to the parts of the object that stay in a full shade. 
* `diffuse` is the base color of the object's body. 
* `specular` and `shininess` are the parameters of the specular highlight: its color and its sharpness, respectively. 

The overal lighting formula now scales each color separately and then blends them all together: 

$$
\begin{aligned}
k_d &= I_L \cdot \max(\mathbf{l}\cdot\mathbf{n}, 0) \\
k_s &= I_L \cdot \max(\mathbf{n}\cdot\mathbf{h}, 0)^s \\
\mathbf{C}_{out} &= \mathbf{C}_{emissive} + k_d\,\mathbf{C}_{diffuse} + k_s\,\mathbf{C}_{specular}
\end{aligned}
$$

The same vector notation is used here: $\mathbf{l}$ toward light, $\mathbf{n}$ surface normal, $\mathbf{v}$ toward eye, and $\mathbf{h}$ as the Blinn half-vector.

## Introducing linear color space 

Another important cleanup is that lighting composition now happens in _linear color space_, with conversion back to `Rgb` at the output edge. This is easy to gloss over, but it is exactly the kind of detail that decides whether “same palette values” look plausible or weird once highlights and specular terms interact.

This also explains part of the recent discussion around matching Three.js: literal hex values are only half the story; the equation and color-space assumptions matter just as much.

<div class="still-compare">
<figure>
<img src="{{ "/assets/images/2026-06-17-materials-colors-and-the-stage/still-scene-rgb.webp" | relative_url }}" alt="Still scene rendered in non-linear RGB composition" />
<figcaption>Rendered with non-linear RGB composition</figcaption>
</figure>
<figure>
<img src="{{ "/assets/images/2026-06-17-materials-colors-and-the-stage/still-scene-linear.webp" | relative_url }}" alt="Still scene rendered in linear color space before output conversion" />
<figcaption>Rendered with linear color-space composition</figcaption>
</figure>
</div>

## Implementation bridge

Most of the work lands in a few focused places:

- [`Material` and `DirectionalLight`][source-lighting] own the per-fragment shading inputs and contributions.
- [`Color`][source-lighting-color] provides the linear-space math path used by material shading.
- [`default_material()`][source-default-material] and `SCENE_BACKGROUND` define the current export defaults.
- [`FrameBuffer::clear`][source-framebuffer-clear] makes scene background explicit.
- [`still-scene`][source-still-scene] and [`animated-scene`][source-animated-scene] both render through the same shape/material/light setup.

The release also tightened confidence around output stability with a golden still-scene render check, and added developer-facing helpers (a performance eval bin and a tag-driven release workflow) to keep iteration smoother in upcoming milestones.

## What comes next

With material and single-light plumbing shipped, the next Phase 2 step is clear: _multiple directional lights_ and summed contributions, using the three-light setup from the Three.js torus reference.

After that, the remaining optional stretches (perspective on CPU and positional lights) stay available, and the final Phase 2 parity pass can revisit shader equations/material tuning with all core lighting pieces in place.

[post-planning-phase-2]: {{site.baseurl}}/{% post_url 2026-06-17-planning-phase-2 %}
[post-first-shot-at-glossy-shapes]: {{site.baseurl}}/{% post_url 2026-06-03-first-shot-at-glossy-shapes %}
[post-first-shot-at-glossy-shapes-blinn-phong]: {{site.baseurl}}/{% post_url 2026-06-03-first-shot-at-glossy-shapes %}#blinnphong-reflection-model
[version-0-1-0]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.1.0
[source-material-0-0-13]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.13/src/lighting.rs#L5
[source-lighting]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/lighting.rs#L9
[source-lighting-color]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/lighting/color.rs#L5
[source-default-material]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/lib.rs#L46
[source-framebuffer-clear]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/framebuffer.rs#L43
[source-still-scene]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/bin/still-scene/main.rs#L52
[source-animated-scene]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/bin/animated-scene.rs#L31
