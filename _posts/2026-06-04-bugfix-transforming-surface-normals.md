---
layout: post
title: "Bugfix: Transforming Surface Normals"
date: 2026-06-04 10:00:00 +0200
authors: Sergey and Cursor
tags: [bugfix]
---

In the [last post][post-first-shot-at-glossy-shapes] we left a teaser that we'd discovered a long-living bug in our renderer, without explaining what it was. Now it's time to dive into the details and fix that nasty error that has been living in the codebase for far too long, to be honest.

[Version 0.0.14 on GitHub][version-0-0-14]{: .no-github-icon}

## What you will see

The animated scene itself hasn't changed much, but you'll notice that we now squash the sphere much more heavily [than in the previous iteration][post-first-shot-at-glossy-shapes]. That's deliberate: the bug went unnoticed because we were not brave enough with deforming our sample shapes.

![Animated Blinn–Phong sphere with corrected normals under squash](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.14/doc/output/current.webp)

When you look at this animation, pay attention to the shadows and the highlight: the way they behave when the sphere gets heavily squashed. They look quite natural, don't they? Well, that wouldn't be the case if we hadn't fixed the bug.

Let's jump into it.

## How we found the bug

Sergey was revisiting a book called [_The Ray Tracer Challenge_][link-to-good-reads] when, in the chapter _"Light and Shading"_, he came across a section on applying transformations to surface normals. After reading it, he realized we had been doing it wrong all along!

A [quick unit test][source-normal-transform-test] that used a non-uniform scaling transform (i.e., the transform that scaled the shape by different amounts along different axes) revealed the bug. Essentially, that test verified the following invariant:

> After transformation, the facet normal should be the same as if we took the transformed vertices and recalculated the normal from their new coordinates.

In other words, the facet normal must stay perpendicular to the facet's plane. And it was broken. The nasty thing that let this bug live so long is that not every transformation was affected: rotations and uniform scaling—the transforms we routinely used—worked just fine, by their mathematical nature.

Things get really wrong when we apply non-uniform scaling to the shape. Let's have a look at how it manifests itself in the render. Take the sphere and squash it into a flat pebble:

<div class="still-compare">
<figure>
<img src="{{site.baseurl}}/assets/images/2026-06-04-bugfix-transforming-surface-normals/still-scene-orig.webp" alt="Original sphere before squashing" />
<figcaption>Original sphere</figcaption>
</figure>
<figure>
<img src="{{site.baseurl}}/assets/images/2026-06-04-bugfix-transforming-surface-normals/still-scene-buggy.webp" alt="Squashed sphere with incorrect specular and shading" />
<figcaption>Buggy image after squashing</figcaption>
</figure>
</div>

Indeed, the shadows and the highlight don't look as they would've appeared on a pebble-shaped object. Instead, it looks like someone just took the sphere and resized the raster image! That's all because of the broken surface normals.

## Surface normals need a special kind of transform

Let's explore in detail what was wrong in the code. If you look at the previous version of [`Shape::transform()`][source-shape-transform-old] and its companion [`Facet::transform()`][source-facet-transform-old], you'll see that we applied _the same transform matrix_ to everything: vertices, the facet's normal, and vertex normals. Visually, that's what was going on when we did that:

![Applying the same scaling transform to a normal]({{site.baseurl}}/assets/images/2026-06-04-bugfix-transforming-surface-normals/apply-scaling-to-normal.svg)

As you can see, the normal vector $n$ gets scaled as well, and now its new value $n'$ is no longer perpendicular to the plane! What we actually need is for the normal to transform into $n''$.

It turns out that there's a special kind of transform, called _normal transform_, that needs to be applied to the normals to keep them perpendicular to the surface. It's related to the original model transform $\mathbf{T}$ by the following equation:

$$
\mathbf{T_n} = (\mathbf{T}^{-1})^T \quad \quad \mathbf{n}' = \mathbf{T_n}\, \mathbf{n}
$$

That formula also reveals why we hadn't noticed that error before, when we only used rotation transforms. Rotation matrices are _orthogonal_ ($\mathbf{T}^{-1} = \mathbf{T}^T$), so $\mathbf{T_n} = (\mathbf{T}^T)^T = \mathbf{T}$.

### Derivation of the normal transform

Where does the equation above come from? The answer comes from one geometric requirement we care about: after transforming the facet, the stored normal must still be perpendicular to the facet plane. In other words, the original property of the normal was 

$$
\mathbf{n} \cdot \mathbf{t} = 0
$$

for any vector $\mathbf{t}$ that lies on the facet's plane. We would like to find a transform $\mathbf{T_n}$ that we can apply to the normal, so that after the transformation the transformed normal $\mathbf{n}'$ would still be perpendicular to vectors $\mathbf{t}'$ transformed with the model transform $\mathbf{T}$. In mathematical terms, we can write it as: 

$$
\begin{gather}
\mathbf{n}' \cdot \mathbf{t}' = 0 \\
(\mathbf{T_n}\mathbf{n}) \cdot (\mathbf{T}\mathbf{t}) = 0 
\end{gather}
$$

Remembering that we can write the dot product as a matrix multiplication $\mathbf{a} \cdot \mathbf{b} = \mathbf{a}^T\mathbf{b}\$ and that $(\mathbf{x}\mathbf{y})^T = \mathbf{y}^T\mathbf{x}^T$, we can apply a couple of tricks to this equation: 

$$
(\mathbf{T_n}\mathbf{n}) \cdot (\mathbf{T}\mathbf{t}) = 0 
\qquad\Rightarrow\qquad
(\mathbf{T_n}\mathbf{n})^T(\mathbf{T}\mathbf{t}) = 0 
\qquad\Rightarrow\qquad
\mathbf{n}^T\left(\mathbf{T_n}^T\mathbf{T}\right)\mathbf{t} = 0
$$

Since $\mathbf{n}^T \mathbf{t} = 0$ by the properties of the normal vector, the equation above is satisfied if $\mathbf{T_n}^T\mathbf{T} = \mathbf{I}$, so: 

$$
\mathbf{T_n}^T \mathbf{T} = \mathbf{I}
\qquad\Rightarrow\qquad
\mathbf{T_n}^T = \mathbf{T}^{-1}
\qquad\Rightarrow\qquad
\boxed{\mathbf{T_n} = (\mathbf{T}^{-1})^T}
$$

## The fix

To make the code more error-prone, so that we don't accidentally apply a wrong type of transform to the facet's normals, we've introduced a small _newtype_ [`NormalTransform`][source-normal-transform], to build a normal transform: 

```rust
pub(crate) struct NormalTransform(Mat3);

impl NormalTransform {    
    pub fn from_model(model: Mat4) -> Self {
        let linear = Mat3::from_mat4(model);
        Self(linear.inverse().transpose())
    }

    #[inline]
    pub(crate) fn apply(self, v: Vec3) -> Vec3 {
        self.0.mul_vec3(v)
    }
}
```

We build this kind of transform once in in [`Shape::transform`][source-shape-transform], and apply it to the facets: 
```rust
impl Shape {
    pub fn transform(&self, m: Mat4) -> Shape {
        let normal_transform = NormalTransform::from_model(m);
        Shape {
            vertices: self
                .vertices
                .iter()
                .copied()
                .map(|v| m.transform_point3(v))
                .collect(),
            facets: self
                .facets
                .iter()
                .map(|f| f.transform(normal_transform))
                .collect(),
        }
    }
}
```

Notice that `Facet::transform()` now requires its argument to be of type `NormalTransform`, so that we can't pass an arbitrary `Mat4` into it. That's a small guardrail to avoid nasty hard-to-detect errors in the future. 

### The visual effect of the fix

That small fix has dramatic consequences. Now, when we squeeze the sphere into the pebble, it renders correctly:

<div class="still-compare">
<figure>
<img src="{{site.baseurl}}/assets/images/2026-06-04-bugfix-transforming-surface-normals/still-scene-orig.webp" alt="Original sphere before squashing" />
<figcaption>Original sphere</figcaption>
</figure>
<figure>
<img src="{{site.baseurl}}/assets/images/2026-06-04-bugfix-transforming-surface-normals/still-scene-fixed.webp" alt="Squashed sphere with corrected specular and shading" />
<figcaption>Correct image after squashing</figcaption>
</figure>
</div>

The shadow and the light highlight look exactly like what they would on a pebble-shaped object. Nice!

## What comes next

With normals trustworthy under deformation, we can return to the plan from the [glossy-shapes milestone][post-first-shot-at-glossy-shapes]: replace Gouraud with [_Phong shading_][phong-shading], for more natural specular highlights on glossy surfaces.

[post-first-shot-at-glossy-shapes]: {{site.baseurl}}/{% post_url 2026-06-03-first-shot-at-glossy-shapes %}
[version-0-0-14]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.14
[link-to-good-reads]: https://www.goodreads.com/book/show/39933047-the-ray-tracer-challenge
[phong-shading]: https://en.wikipedia.org/wiki/Phong_shading
[source-normal-transform-test]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.14/src/geometry/shape.rs#L138
[source-shape-transform-old]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.13/src/geometry/shape.rs#L37
[source-facet-transform-old]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.13/src/geometry/facet.rs#L71
[source-shape-transform]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.14/src/geometry/shape.rs#L36
[source-normal-transform]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.14/src/geometry/facet.rs#L91
