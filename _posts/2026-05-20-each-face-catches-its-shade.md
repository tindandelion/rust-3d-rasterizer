---
layout: post
title: "The Cube Gets Light"
date: 2026-05-20 
authors: Sergey and Cursor
---

[Version 0.0.7][post-near-face-classified-as-back] fixed front-face culling so we were finally painting the side of the cube that faces the camera. The filled cube from [Version 0.0.6][post-cube-paints-its-six-faces] still used six flat tints — enough to prove that we can render the cube, but still looking a bit cartoonish. This new version closes the the important milestone: we learn how to apply _shading_ to cube's faces, so is starts to look as if it was illuminated by the sun.

[Version 0.0.8 on GitHub][version-0-0-8]{: .no-github-icon}

## What you will see

We still use the same orthographic camera, same rotating cube — but the cube is no longer a rainbow block. Every face of the cube how has the same base color; what varies is how much light reaches it. Facets turned toward the “sun” read brighter; facets in shadow fall darker. The picture now start to look slightly more realistic.

![Animated diffuse-lit blue cube with per-face shading](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.8/doc/output/current.webp)

## The lighting model that we used 

Introducing light adds "realism" to rendered 3D scenes. However, if we try to model the light closely to the physical world, it becomes very complex very fast. First, the physics of how light interacts with different surfaces is pretty complicated. Second, it requires **a lot** of computation power to model the light realistically. 

To avoid these traps, in 3D graphics we usually use approximate models that are easier to understand and less compute-intensive, but still give us quite good results, depending on the circumstances and requirements. 

For this miletone, we've chosen one of the simplest models to implement: _directional light source_ combined with _diffuse shading_. Let's dive into what that means. 

### The light source: directional light 

Out of many light source models, [directional light][directional-light] is the simplest one. It is characterised by a single vector: _the direction_ from which the light is coming from;  all rays emitted by such a light are parallel to each other. The effect of this kind of light source is that, if you have a flat surface, it will be illuminated uniformly at each point. 

In nature, we see the example of the directional light every day: it's the Sun. Though it's not 100% precise, we can think of the Sun as the directional light: it's located so far away from Earth, that all light rays that reach us are effectively parallel. 

Two other characteristics to reason about are light _intensity_ and _color_. For the purposes of our current miletone, we'll assume that the light is **white** (just like the Sun), and the intensity is just a coefficient in the range of [0.0, 1.0]. 

### Ambient light 

_Ambient light_ is yet another approximation of a physical world. To understand it, let's consider a sunny day on Earth. There's no complete darkness: even the objects in shade receive some light, indirectly: mostly from the atmosphere, or reflected light from other objects nearby. 

Modelling this type of lighting precisely using the Sun as the light source and tracking all reflections adds a lot of complexity to the lighting model, so we come up with an approximate solution: an _ambient light_, which is characterised only by its intensity. We declare that an ambient light contributes some light to every point in the scene, no matter where that point is. 

The ambient light of non-zero intensity allows us to see the obejcts in there scene even if the light from the light source doesn't reach them. 

### Surface material: diffuse reflection 

The way the object's sureface interacts with light is another big part of the entire equation. We see the solid non-transparent objects because their surface absorbs and reflects light. Depending on how objects reflect light, we can classify them broadly into "matte" and "glossy" ones. For the time being, we're going to focus only on the matte objects. They're simpler to reason about, which makes them a good starting point to implement. 

The property that makes such surfaces simple is that their color or luminosity stays the same no matter from which point you're looking at them. In other words, the intensity of the reflected light is independent of the viewer's position. It does depend on the orientation of the surface with respect to the light's source though. Let's see how we can derive the base equation for the intensity of the reflected light. 

## Modeling diffuse reflection 

Intuitively, we know that the objects perpendicular to the rays of light shine the brightest, and the more you turn them, the dimmer they look. Let's take a look at the picture to see why that happens: 

[Picture]

On the left, you see a cube face that's oriented perpendicular to the light's direction. We could say that it is exposed to the beam of light with the "width" $I_{\max}$. Intuitively, the "wider" this beam, the more overall light energy the cube's face receives. 

Now let's look at the right picture. Here, the same cube face is exposed to the light beam at an angle. We can see from the picture that the cube face is exposed to a much "narrower" light beam $I$. We can calculate this value with respect to $I_{\max}$ looking at the triangle: 

$$
I = I_{\max}\sin(\alpha) = I_{\max}\sin(90^{\circ} - \beta) = I_{\max}\cos(\beta)
$$

To calculate $\beta$, we can look at the vectors $\mathbf{L}$ (direction towards light), and $\mathbf{n}$. Both are *normals*, therefore by the properties of the dot product, we know that $\cos(\beta) = \mathbf{L} \cdot \mathbf{n}$. Practically, we should also clamp the value of $\cos(\beta)$ to the range $[0, 1]$, because for negative values that means that the cube face is facing away from the light, so it doesn't get illuminated at all. 

Therefore, we arrive at the final formula that relates face's illumination, its orientation, and the light direction: 

$$
I = I_{\max} \cdot \max(0, \mathbf{L} \cdot \mathbf{n})
$$

In computer graphics, this formula is called [Lambertian diffuse][lambert]. 

## Implementation details

We've implemented shading in the [`DiffuseLight`][source-diffuse-light] data type. The name of this data type is rather misleading, because in fact it packs a few concepts together: 

* The directional light, represented by `toward_light` vector; 
* Both ambient light and directional light contributions: `ambient_factor` and `diffuse_factor`; 
* Lighting model, represented by [`calc_intensity`][source-calc-intensity] method that calculates the light intensity for a surface represented by the argument `normal`. 

As one might argue, this data type lumps together different concepts, such as the light properties, material properties, and general scene properties. This critique is true: we might need to refactor that data type (or at least rename it) in the future. For now, it serves the purpose of providing a simple interface to calculate the cube's face shading though.

## What comes next

Basic shading on the cube is done. The [project breakdown][project-breakdown] points at **Sphere: triangular mesh** next — procedural sphere facets, triangular facets do describe shapes, and refactoring the cube to two triangles per face so we do not maintain quad and triangle rasterizers in parallel. 

[post-near-face-classified-as-back]: {{site.baseurl}}/{% post_url 2026-05-19-the-near-face-was-classified-as-back %}
[post-cube-paints-its-six-faces]: {{site.baseurl}}/{% post_url 2026-05-18-the-cube-paints-its-six-faces %}
[version-0-0-8]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.8
[lambert]: https://en.wikipedia.org/wiki/Lambertian_reflectance
[directional-light]: https://en.wikipedia.org/wiki/Shading#Light_sources
[project-breakdown]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/doc/planning/project-breakdown.md
[source-cube-palette-0-0-6]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.6/src/lib.rs#L36
[source-cube-albedo]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/src/lib.rs#L31
[source-draw-faces]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/src/lib.rs#L55
[source-diffuse-light]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/src/lighting.rs#L9
[source-calc-intensity]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/src/lighting.rs#L40
[source-rgb-scale]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/src/framebuffer/colors.rs#L13
[source-lighting-tests]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/src/lighting.rs#L47
[source-cube-face]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/src/scene/cube/face.rs#L9
[source-quad]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/src/scene/cube.rs#L26
[source-face-normal]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/src/scene/cube/face.rs#L41
[source-visible-faces]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/src/scene/cube.rs#L78
[source-fill-quad]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/src/framebuffer/fill_quad.rs#L12
[source-is-front-facing]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/src/scene/cube/face.rs#L47
[source-still-unit-cube]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.7/src/bin/still-unit-cube.rs#L1
[source-draw-unit-cube]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.8/tests/draw_unit_cube.rs#L1
