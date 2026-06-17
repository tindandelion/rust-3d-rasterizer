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

We've introduced the [`Material`][material-0.0.13] concept when we first tackled [glossy shading][link to first-shot-at-glossy-shapes]. Back then, it was very simple: just enough to demonstrate the new ability to render specular highlights. Moreover, for a while we lived with a rather awkward code design choices where the material specification and lighting equations were weirdly split between `Shape`, `Material`, and an artificial `BlinnLightModel` data types. 

If we combine all those disjoint pieces together, it's fair to say that we had a simple single-color material setup where the lighting equation to color a pixel was the following: 

[insert a formula here]

### New improved material 

Unlike our previous implementation, now we're moving towards a more flexible material specification. Instead of a single color and a couple of control parameters, we're introducing _different colors_ for each lighting component, matching the `MeshPhongMaterial` from `three.js`: 

* `emissive` is a color of the material that's unaffected by other lighting. Generally speaking, it's the color of the light the object emits by itself, in absence of any light sources. This parameter replaces our previous concept of _ambient color_, serving the same purpose: give some color to the parts of the object that stay in a full shade. 
* `diffuse` is the base color of the object's body. 
* `specular` and `shininess` are the parameters of the specular highlight: its color and its sharpness, respectively. 

The overal lighting formula for the pixel looks like this: 

[insert a formula here]

## Introducing linear color space 

Another important cleanup is that lighting composition now happens in _linear color space_, with conversion back to `Rgb` at the output edge. This is easy to gloss over, but it is exactly the kind of detail that decides whether “same palette values” look plausible or weird once highlights and specular terms interact.

This also explains part of the recent discussion around matching Three.js: literal hex values are only half the story; the equation and color-space assumptions matter just as much.

[insert the side-by-side pictures of still-scene-rgb.webp and still-scene-linear.webp from assets]

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
[version-0-1-0]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.1.0
[source-lighting]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/lighting.rs#L9
[source-lighting-color]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/lighting/color.rs#L5
[source-default-material]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/lib.rs#L46
[source-framebuffer-clear]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/framebuffer.rs#L43
[source-still-scene]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/bin/still-scene/main.rs#L52
[source-animated-scene]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/bin/animated-scene.rs#L31
