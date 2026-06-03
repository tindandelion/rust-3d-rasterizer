---
layout: post
title: "First Shot at Glossy Shapes"
date: 2026-06-03 10:00:00 +0200
authors: Sergey and Cursor
---

In [the last session][post-the-sphere-gets-smooth] explored Gouraud shading and how it can be used to render a smooth matte sphere. Now we're up to a new challenge: let's make the sphere look glossy! We'll explore in practice the limitations of Gouraud shading when it comes to drawing shiny shapes with specular highlights and gives us a motivation to search for a more realistic shading algorithm.

[Version 0.0.13 on GitHub][version-0-0-13]{: .no-github-icon}

## What you will see

We still see a blue sphere, but notice the difference with the [previous iteration]: now there's a spec of light reflected from the surface! We've added some glossiness to the material. 

![Animated Blinn–Phong sphere with specular highlight, camera orbit and vertical squash](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.13/doc/output/current.webp)

This animation demonstrates a few crucial areas for improvement, though: 

- The white spec doesn't look very natural. This demonstrates the limitation of the Gouraud shading algorithm: it's good for matte surfaces, but when it comes to rendering specs of light, it's not that good; 
- If you look carefully, you'll  notice that the shadows and the light spec behave somewhat unexpectedly when the sphere gets squeezed. This is actually a bug in our rendering pipeline that we plan to address next. 

## From matte to glossy

Back in the [The Cube Gets Light][post-the-cube-gets-light] session, we introduced a lighting model that supported rendering of matte objects, as if made of carboard or clay. Now we're going to extend our model to include _glossy surfaces_, that also reflect a concentrated patch of light toward the viewer's eye.

Imagine a billard ball, for example. Apart from the broad diffuse shading, you'll notice a bright spot of light that seems to move as you move around the ball. Unlike matte reflection that stays the same no matter where you stand, that _specular hightlight_ actually depends on your point of view. 

This new effect complements the diffuse reflection we implemented earlier. There is a number of lighting models that approximate glossiness. Among them, [_Blinn–Phong_][blinn-phong] model is probably the simplest: it's not physically accurate, but it's computationally cheap and produces plausible visual effects. 

## The derivation of Blinn–Phong specular term

Let's recall the law of specular reflection: 

> The angle of incidence equals the angle of reflection, both measured from the surface normal at the point of contact.

That law gives us the relation between three vectors: the direction towards the light $\mathbf{l}$, the reflection vector $\mathbf{r}$, and the surface normal $\mathbf{n}$. For a perfectly smooth flat surface, all light is reflected along $\mathbf{r}$. Real surfaces deviate from this, though. Depending on the material, they will reflect the light imperfectly at different angles, but they still will be concentrated around $\mathbf{r}$: 

![Picture]

We can find $\mathbf{r}$ from $\mathbf{l}$ and $\mathbf{n}$ by the following formula: 

$$
\mathbf{r} = 2 \mathbf{n}\, (\mathbf{l} \cdot \mathbf{n}) - \mathbf{n}
$$

Once we have $\mathbf{r}$, we can write the equation for the Phong's version of specular reflection: 

$$
I_s = I \cdot \left(\frac{\mathbf{v} \cdot \mathbf{r}}{|\mathbf{v}| |\mathbf{r}|}\right)^s
$$

where $\mathbf{v}$ is a unit vector towards the viewer's eye, $I$ is the light's intensity, and $s$ is a _shininess exponent_. The value of $s$ is a parameter of the material that controls how sharp the highlight will be: the bigger the value of $s$, the sharper and smaller the highlight will be in the render. 

### Blinn-Phong variation 

Original Phong model uses the reflection vector $\mathbf{r}$ to calculate the fraction of the reflected light. Blinn's finding was that the simiar result can be achieved with a cheaper calculation: use a _half vector_ between $\mathbf{l}$ and $\mathbf{v}$: 

![Picture]

Notice the relation between $\mathbf{r}$, $\mathbf{h}$ and the normal $\mathbf{n}$. The idea is that when $\mathbf{n}$ aligns with $\mathbf{h}$, the surface is orientesd so that the view vector $\mathbf{v}$ aligns with the reflection vector $\mathbf{r}$: the highlight is the brightest. The bigger the angle between $\mathbf{h}$ and $\mathbf{n}$, the bigger is the angle between $\mathbf{v}$ and $\mathbf{r}$: the less reflected light reaches the eye. 

So Blinn's model uses the same idea as Phong's, but it avoids calculating the reflection vector directly, using the half vector as a substitute: 

$$
\mathbf{h} = \frac{\mathbf{l} + \mathbf{v}}{|\mathbf{l} + \mathbf{v}|} \quad \quad

I_s = I \cdot \left(\mathbf{n} \cdot \mathbf{h}\right)^s
$$

That's essentially the approach we use in the code. [`BlinnLightModel`][source-blinn-light] replaces the old diffuse-only light type and implements [`calc_intensity`][source-calc-intensity] using both diffuse and specular terms.

## What's next

So now we combine Phong-Bling lighting model with Gouraud shading, which results in a visual effect called _Gouraud specular_. Because our sphere's mesh is dense enough, the specular looks plausible enough, although it clearly reveals the faceted nature of the sphere. Our next step will be to replace Gouraud shading with Phong shading that uses the interpolation of the normals and leads to more realistically looking specular highlights. 

However, before we move to this step, we need to fix a critical bug in our render pipeline. Stay tuned!


[post-the-sphere-gets-smooth]: {{site.baseurl}}/{% post_url 2026-05-29-the-sphere-gets-smooth %}
[post-the-cube-gets-light]: {{site.baseurl}}/{% post_url 2026-05-22-the-cube-gets-light %}
[version-0-0-13]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.13
[blinn-phong]: https://en.wikipedia.org/wiki/Blinn%E2%80%93Phong_reflection_model
[phong]: https://en.wikipedia.org/wiki/Phong_reflection_model
[half-vector]: https://en.wikipedia.org/wiki/Blinn%E2%80%93Phong_reflection_model#Specular_term
[source-material]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.13/src/lighting.rs#L4
[source-material-matte]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.13/src/lighting.rs#L12
[source-material-shiny]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.13/src/lighting.rs#L16
[source-blinn-light]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.13/src/lighting.rs#L29
[source-calc-intensity]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.13/src/lighting.rs#L42
[source-toward-eye]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.13/src/lib.rs#L73
[source-draw-facets]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.13/src/lib.rs#L66
[source-shaded-fill]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.13/src/framebuffer/shaded_fill_triangle.rs#L13
[source-rgb-scale]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.13/src/framebuffer/colors.rs#L16
[source-still-scene]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.13/src/bin/still-scene.rs#L25
[source-animated-scene]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.13/src/bin/animated-scene.rs#L108
