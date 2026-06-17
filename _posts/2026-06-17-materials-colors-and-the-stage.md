---
layout: post
title: "Materials, Colors, and the Stage"
date: 2026-06-17 09:00:00 +0200
authors: Sergey and Cursor
---

In [Planning Phase 2][post-planning-phase-2] we said we would pause new geometry and sharpen the rendering pipeline itself. Version 0.1.0 is that first step: the torus keeps the same shape, but the scene now has a cleaner material model, an explicit background color, and lighting math that is easier to reason about.

[Version 0.1.0 on GitHub][version-0-1-0]{: .no-github-icon}

## What you will see

The visual target is still the familiar torus scene, now rendered with the Phase 2 material/background setup and exported as before:

![Torus render after Phase 2 material and color pipeline updates](https://github.com/tindandelion/rust-3d-rasterizer/releases/download/0.1.0/scene.webp)

At first glance, this does not look like a dramatic “new feature” release, and that is intentional. This milestone is about making the color and lighting pipeline explicit, so later steps (multi-light and final Three.js parity) are straightforward instead of fragile.

## Why this milestone matters

### Material and light are now separate concepts

In earlier iterations, color and shading logic were more tightly coupled. In 0.1.0, a _material_ now clearly describes surface properties (`emissive`, `diffuse`, `specular`, `shininess`), while a _directional light_ describes incoming light direction and intensity.

That sounds like a small API change, but it is the conceptual split we need for the rest of Phase 2: once the surface and the light are separate, adding several lights becomes “sum contributions from each source” instead of “rewrite half the shader logic.”

### We moved shading math into linear color space

Another important cleanup is that lighting composition now happens in _linear color space_, with conversion back to `Rgb` at the output edge. This is easy to gloss over, but it is exactly the kind of detail that decides whether “same palette values” look plausible or weird once highlights and specular terms interact.

This also explains part of the recent discussion around matching Three.js: literal hex values are only half the story; the equation and color-space assumptions matter just as much.

### The scene background is now explicit

The framebuffer gained explicit clear color support, and the scene now clears to `0x444444` instead of relying on implicit black. That puts the project output and the Phase 2 reference setup in the same visual framing, making comparisons less misleading.

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
