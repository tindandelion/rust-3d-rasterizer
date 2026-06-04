---
layout: post
title: "Bugfix: Transforming Surface Normals"
date: 2026-06-04 10:00:00 +0200
authors: Sergey and Cursor
tags: [bugfix]
---

In [First Shot at Glossy Shapes][post-first-shot-at-glossy-shapes], we shipped Blinn–Phong specular on the sphere and called out a rendering glitch: during the vertical squash in the demo animation, shadows and the highlight did not move the way we expected. That was not a lighting-model bug — our stored _surface normals_ were being transformed the wrong way whenever the mesh was non-uniformly scaled.

[Version 0.0.14 on GitHub][version-0-0-14]{: .no-github-icon}

## What you will see

The animated scene is unchanged in structure — camera orbit, then a squash phase — but lighting now tracks the deformed surface. The [`still-scene`][source-still-scene] export (heavy Y squash) makes the difference easy to see side by side:

<div class="still-compare">
<figure>
<img src="{{site.baseurl}}/assets/images/2026-06-04-bugfix-transforming-surface-normals/still-scene-orig.webp" alt="Squashed sphere with corrected normals" />
<figcaption>Original sphere</figcaption>
</figure>
<figure>
<img src="{{site.baseurl}}/assets/images/2026-06-04-bugfix-transforming-surface-normals/still-scene-buggy.webp" alt="Squashed sphere with incorrect specular and shading" />
<figcaption>After squashing: buggy transform</figcaption>
</figure>
</div>

<div class="still-compare">
<figure>
<img src="{{site.baseurl}}/assets/images/2026-06-04-bugfix-transforming-surface-normals/still-scene-orig.webp" alt="Squashed sphere with corrected normals" />
<figcaption>Original sphere</figcaption>
</figure>
<figure>
<img src="{{site.baseurl}}/assets/images/2026-06-04-bugfix-transforming-surface-normals/still-scene-fixed.webp" alt="Squashed sphere with incorrect specular and shading" />
<figcaption>After squashing: correct transform</figcaption>
</figure>
</div>


After the fix, the same deformation in the full animation keeps highlights and diffuse shading aligned with the squeezed sphere:

![Animated Blinn–Phong sphere with corrected normals under squash](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.14/doc/output/current.webp)

The squash phase is also more extreme than in 0.0.13 (Y down to **0.2**, X/Z up to **0.95** at peak) so the bug would have been even harder to miss without the fix.

## How we spotted it

The symptom showed up in motion first. With [`BlinnLightModel`][source-blinn-light] and Gouraud interpolation, a stretched sphere should still read as glossy: the specular patch should slide as the surface tilts and compresses. Instead, during the second half of [`animated-scene`][source-animated-scene], the highlight and diffuse shading looked glued to the wrong parts of the mesh — as if the triangles moved but their normals did not follow.

To pin it down without staring at every frame, Sergey added an integration test on [`Shape::transform`][source-shape-transform]: build a triangle, apply a non-uniform scale, and compare the stored facet normal to the normal recomputed from the transformed corner positions (`UnitVec3::from_points_ccw`). Vertices were already correct; **only the stored normals failed**.

## Directions are not normals under scale

When we pose a mesh with a model matrix $M$, we transform vertex positions with $M$ as points (homogeneous coordinates, then perspective divide if needed). For a pure rotation or uniform scale, you can also multiply a normal $\mathbf{n}$ by the same $3 \times 3$ linear part and renormalize — that is what `Mat4::transform_vector3` does.

A _non-uniform_ scale is different. Normals are covectors: they describe which direction is "perpendicular to the surface" in a way that stays consistent with tangent vectors. If you scale the mesh by $2$ along $+\mathrm{Y}$ only, tangent vectors along $+\mathrm{Y}$ stretch, but the outward normal should shrink along $+\mathrm{Y}$ so that lighting still uses the true slant of the squashed surface. Applying the same scale matrix to $\mathbf{n}$ overshoots along that axis.

The correct linear map for normals is the **inverse transpose** of the model's upper-left $3 \times 3$ block $L$:

$$
\mathbf{n}' = (L^{-1})^{\mathsf T}\, \mathbf{n}
$$

then renormalize to a unit vector. Rotations and uniform scales are special cases where this matches `transform_vector3`; anisotropic squash is not.

Our old [`Facet::transform`][source-facet-transform-old] path used `m.transform_vector3` on the facet normal and each vertex normal. That was fine for the tumbling cube and orbit-only sphere in earlier releases, but wrong as soon as [`still-scene`][source-still-scene] and the animation's squash phase applied different scale factors on different axes.

## The fix

We introduced a small crate-internal type, [`NormalTransform`][source-normal-transform], built once per pose in [`Shape::transform`][source-shape-transform]:

```rust
let normal_transform = NormalTransform::from_model(m);
// ...
.map(|f| f.transform(normal_transform))
```

`NormalTransform::from_model` takes the inverse transpose of $L$; each facet only applies that fixed `Mat3` to its stored normals — no matrix inverse per triangle. Uniform and rotation-only poses still match the old `transform_vector3` behavior; the new unit test on non-uniform scale (`scale(1, 0.5, 1)` doubles the normal's $+\mathrm{Y}$ component before normalization) guards the regression.

We kept a before snapshot ([`still-scene-buggy.webp`][still-scene-buggy]) and the corrected still export from main ([`still-scene.webp`][still-scene-orig]) for comparison; the animation [`current.webp`][current-output] was refreshed after the fix. [`still-scene`][source-still-scene] uses `Mat4::from_scale(Vec3::new(0.7, 0.15, 0.7))` as a compact reproducer for the squash case.

## What comes next

With normals trustworthy under deformation, we can return to the plan from the glossy-shapes milestone: [_Phong shading_][phong-shading] (per-pixel normals) instead of Gouraud specular, for sharper highlights on the sphere.


[post-first-shot-at-glossy-shapes]: {{site.baseurl}}/{% post_url 2026-06-03-first-shot-at-glossy-shapes %}
[version-0-0-14]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.14
[phong-shading]: https://en.wikipedia.org/wiki/Phong_shading
[source-blinn-light]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.14/src/lighting.rs#L29
[source-animated-scene]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.14/src/bin/animated-scene.rs#L1
[source-still-scene]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.14/src/bin/still-scene.rs#L14
[source-shape-transform]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.14/src/geometry/shape.rs#L36
[source-normal-transform]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.14/src/geometry/facet.rs#L91
[source-facet-transform-old]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.13/src/geometry/facet.rs#L70
[still-scene-buggy]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.14/doc/output/still-scene-buggy.webp
[still-scene-orig]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/doc/output/still-scene.webp
[current-output]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.14/doc/output/current.webp
