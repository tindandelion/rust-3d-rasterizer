---
layout: post
title: "Bugfix: Transforming Surface Normals"
date: 2026-06-04 10:00:00 +0200
authors: Sergey and Cursor
tags: [bugfix]
---

In the [last post][post-first-shot-at-glossy-shapes] we left a teaser that we've discovered a long-living bug in our renderer, without explaining what it was. Now, it's time to dive deep into the details and fix that nasty error that have been living in the codebase for far too long, to be honest. 

[Version 0.0.14 on GitHub][version-0-0-14]{: .no-github-icon}

## What you will see

The animated scene itself hasn't changed much, but you'll notice that we now squash the sphere much more heavily [than in previous iterations][Link to prev what you will see]. That's deliberate: the bug we've been working on has been unnoticed because we were not brave enough with deforming our sample shapes. 

![Animated Blinn–Phong sphere with corrected normals under squash](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.14/doc/output/current.webp)

When you look at this animation, pay attention to the shadows and the highlight: the way it behaves when the sphere gets heavily squashed. They look quite natural, don't they? Well, it wouldn't be the case if we hadn't fixed the bug. 

Let's jump into it. 

## How we found the bug 

Sergey was revisiting a book [_The Ray Tracer Challenge_][link-to-good-reads], while suddenly in the chapter _"Light and Shading"_ he found a section dedicated to applying transformations to surface normals. Having read that section, he realized that we've being doing it wrong all along! 

A [quick unit test][link to the test code] that used a non-uniform scaling transform (i. e. the transform that scaled the shape by different amounts along different axes) revealed the bug. Essentially, that test verified the following invariant: 

> After transformation, the facet normal should be the same as if we took the transformed vertices, and recalculated the normal from their new coordinates. 

In other words, the facet normal must stay perpendicular to the facet's plane. And it was broken. But the nasty thing that let this bug live is that not every transformation was broken, essentially. Rotations, uniform scaling - the transformations we routinely used - worked just fine, by their mathematical nature. 

The things get really wrong when we apply non-uniform scaling to the shape. let's have a look at how it manifests itself in the render. Take the sphere and squash it into a flat pebble:  

<div class="still-compare">
<figure>
<img src="{{site.baseurl}}/assets/images/2026-06-04-bugfix-transforming-surface-normals/still-scene-orig.webp" alt="Squashed sphere with corrected normals" />
<figcaption>Original sphere</figcaption>
</figure>
<figure>
<img src="{{site.baseurl}}/assets/images/2026-06-04-bugfix-transforming-surface-normals/still-scene-buggy.webp" alt="Squashed sphere with incorrect specular and shading" />
<figcaption>Buggy image after squashing</figcaption>
</figure>
</div>

Indeed, the shadows and the highlight don't look as they would've appeared on a pebble-shaped object. Instead, it looks like someone just took the sphere and resized the raster image! That's all because of the broken surface normals. 

## Surface normals need a special kind of transform 

Let's explore in details what was wrong in the code. If you look at the previous version of [`Shape::transform()`][link-to-prev-impl] and its companion [`Facet::transform()`][link-to-prev-impl], you'd see that we applied _the same transform matrix_ everything: vertices, facet's normal, and vertex normals. Visually, that's what was going on when we did that: 

![Picture][apply-scaling-to-normal]

As you can see, the normal vector $n$ gets scaled as well, and now its new value $n'$ is no longer perpendicular to the plane! What we actually need is to make the normal transform into $n''$. 

It turns out that there's a special kind of transform, called _normal transform_, that needs to be applied to the normals, to stay perpendicular to the surface. It's related to the original model transform $\mathbf{T}$ by the following equation: 

$$
\mathbf{T_n} = (\mathbf{T}^{-1})^T \quad \quad \mathbf{n}' = \mathbf{T_n}\, \mathbf{n}
$$

That formular also reveals why we haven't noticed that error before, when we only used rotation transforms. Rotation matrix is _orthogonal_ ($\mathbf{T}^{-1} = \mathbf{T}^T$), so $\mathbf{T_n} = (\mathbf{T}^T)^T = \mathbf{T}$. 

### Derivation of the normal transform

TODO: To be written 


## The fix

We introduced a small internal type, [`NormalTransform`][source-normal-transform], built once in [`Shape::transform`][source-shape-transform] from the original model transform, using the formula above:

```rust
let normal_transform = NormalTransform::from_model(m);
// ...
.map(|f| f.transform(normal_transform))
```

That small fix has dramatic consequences. Now, when we squeeze the sphere into the pebble, it renders correctly: 

<div class="still-compare">
<figure>
<img src="{{site.baseurl}}/assets/images/2026-06-04-bugfix-transforming-surface-normals/still-scene-orig.webp" alt="Squashed sphere with corrected normals" />
<figcaption>Original sphere</figcaption>
</figure>
<figure>
<img src="{{site.baseurl}}/assets/images/2026-06-04-bugfix-transforming-surface-normals/still-scene-fixed.webp" alt="Squashed sphere with incorrect specular and shading" />
<figcaption>Correct image after squashing</figcaption>
</figure>
</div>

The shadow and the light highlight look exactly what they would look like on a pebble-shaped object. Nice!



## What comes next

With normals trustworthy under deformation, we can return to the plan from the glossy-shapes milestone: replace Gouraud shading with [_Phong_][phong-shading], for more natural specular highlights on glossy surfaces.


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
