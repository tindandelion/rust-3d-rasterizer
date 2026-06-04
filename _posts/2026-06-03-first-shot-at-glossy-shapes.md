---
layout: post
title: "First Shot at Glossy Shapes"
date: 2026-06-03 10:00:00 +0200
authors: Sergey and Cursor
---

In [the last session][post-the-sphere-gets-smooth], we explored Gouraud shading and how it renders a smooth matte sphere. Now we face a new challenge: make the sphere look glossy! We will see in practice how Gouraud shading limits shiny shapes with _specular highlights_ — and that motivates the search for a more realistic shading algorithm.

[Version 0.0.13 on GitHub][version-0-0-13]{: .no-github-icon}

## What you will see

We still see a blue sphere, but notice the difference from the [previous iteration][post-the-sphere-gets-smooth]: a bright specular highlight now appears on the surface. We upgraded the lighting model and made the material shinier.

![Animated Blinn–Phong sphere with specular highlight, camera orbit and vertical squash](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.13/doc/output/current.webp)

This animation demonstrates a few crucial areas for improvement, though:

- The white highlight does not look very natural. That shows a limitation of Gouraud shading: it works well for matte surfaces, but specular highlights suffer when intensities are interpolated across triangle corners.
- If you look carefully, you will notice that the shadows and the highlight behave somewhat unexpectedly when the sphere gets squeezed in the second part of the animation. We chased that in [Bugfix: Transforming Surface Normals][post-bugfix-transforming-surface-normals] — it turned out that we were applying a wrong transform to the surface normals.

## From matte to glossy

Back in [The Cube Gets Light][post-the-cube-gets-light], we introduced a lighting model that supported rendering of matte objects, as if made of cardboard or clay. Now we are going to extend our model to include _glossy surfaces_ that also reflect a concentrated patch of light toward the viewer's eye.

Imagine a billiard ball, for example. Apart from the broad diffuse shading, you will notice a bright spot of light that seems to move as you move around the ball. Unlike matte reflection, which stays much the same no matter where you stand, that specular highlight actually depends on your point of view.

This new effect complements the diffuse reflection we implemented earlier. Several lighting models approximate glossiness. Among them, the [_Blinn–Phong_][blinn-phong] model is probably the simplest: it is not physically accurate, but it is computationally cheap and produces plausible visual effects.

## Blinn–Phong reflection model

Let's recall the law of specular reflection:

> The angle of incidence equals the angle of reflection, both measured from the surface normal at the point of contact.

That law gives us the relation between three vectors: the direction toward the light $\mathbf{l}$, the reflection vector $\mathbf{r}$, and the surface normal $\mathbf{n}$. For a perfectly smooth flat surface, all light is reflected along $\mathbf{r}$. Real surfaces deviate from this, though. Depending on the material, they reflect light imperfectly at different angles, but it still concentrates around $\mathbf{r}$:

![Specular reflection: light, reflection, view, and normal vectors]({{site.baseurl}}/assets/images/2026-06-03-first-shot-at-glossy-shapes/reflection-angle.svg)

We can find $\mathbf{r}$ from $\mathbf{l}$ and $\mathbf{n}$ with:

$$
\mathbf{r} = 2 \mathbf{n}\, (\mathbf{l} \cdot \mathbf{n}) - \mathbf{l}
$$

Once we have $\mathbf{r}$, we can reason about the angle $\beta$ between the view and reflection vectors. The larger that angle, the less reflected light reaches the viewer. The [Phong reflection model][phong] writes specular intensity as:

$$
I_s = I \cdot \left(\frac{\mathbf{v} \cdot \mathbf{r}}{|\mathbf{v}| |\mathbf{r}|}\right)^s
$$

where $\mathbf{v}$ is a unit vector toward the viewer's eye, $I$ is the light's intensity, and $s$ is a _shininess exponent_. The value of $s$ is a material parameter that controls how sharp the highlight will be: the larger $s$, the sharper and smaller the highlight appears in the render.

### Blinn–Phong variation

The original Phong model uses the reflection vector $\mathbf{r}$ to calculate the fraction of reflected light. Blinn's finding was that a similar result can be achieved with a cheaper calculation: use a [_half vector_][half-vector] between $\mathbf{l}$ and $\mathbf{v}$.

![Half vector between light and view directions]({{site.baseurl}}/assets/images/2026-06-03-first-shot-at-glossy-shapes/half-vector.svg)

Notice the relation between $\mathbf{r}$, $\mathbf{h}$, and the normal $\mathbf{n}$. When $\mathbf{n}$ aligns with $\mathbf{h}$, the surface is oriented so that the view vector $\mathbf{v}$ aligns with the reflection vector $\mathbf{r}$: the highlight is brightest. The larger the angle $\gamma$ between $\mathbf{h}$ and $\mathbf{n}$, the larger the angle between $\mathbf{v}$ and $\mathbf{r}$, and the less reflected light reaches the eye.

So Blinn's model uses the same idea as Phong's, but it avoids calculating the reflection vector directly and uses the half vector as a substitute:

$$
\mathbf{h} = \frac{\mathbf{l} + \mathbf{v}}{|\mathbf{l} + \mathbf{v}|} \quad \quad

I_s = I \cdot \left(\mathbf{n} \cdot \mathbf{h}\right)^s
$$

That is essentially the approach we use in the code. [`BlinnLightModel`][source-blinn-light] replaces the old diffuse-only light model and implements [`calc_intensity`][source-calc-intensity] that takes into account both diffuse and specular terms.

## What comes next

We now combine the Blinn–Phong lighting model with Gouraud shading, which produces an effect often called _Gouraud specular_. Because our sphere mesh is dense enough, the highlight looks plausible, although it still reveals the faceted nature of the sphere. Our next step is to replace Gouraud shading with [_Phong shading_][phong-shading], which interpolates normals and yields more realistic-looking specular highlights.

However, the first thing to do is to [fix a bug with transforming surface normals][post-bugfix-transforming-surface-normals], that has been living in the code undetected for a while now. Let's do some bug hunting!


[post-bugfix-transforming-surface-normals]: {{site.baseurl}}/{% post_url 2026-06-04-bugfix-transforming-surface-normals %}
[post-the-sphere-gets-smooth]: {{site.baseurl}}/{% post_url 2026-05-29-the-sphere-gets-smooth %}
[post-the-cube-gets-light]: {{site.baseurl}}/{% post_url 2026-05-22-the-cube-gets-light %}
[version-0-0-13]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.13
[blinn-phong]: https://en.wikipedia.org/wiki/Blinn%E2%80%93Phong_reflection_model
[phong]: https://en.wikipedia.org/wiki/Phong_reflection_model
[phong-shading]: https://en.wikipedia.org/wiki/Phong_shading
[half-vector]: https://en.wikipedia.org/wiki/Blinn%E2%80%93Phong_reflection_model#Specular_term
[source-blinn-light]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.13/src/lighting.rs#L29
[source-calc-intensity]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.13/src/lighting.rs#L42
