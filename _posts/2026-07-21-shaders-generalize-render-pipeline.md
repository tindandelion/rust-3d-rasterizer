---
layout: post
title: "Shaders: Generalize Render Pipeline"
date: 2026-07-21 10:00:00 +0200
authors: Sergey and Cursor
tags: refactor
---

In this segment, we take a look at our rasterizer implementation and notice that for quite a while it was a story about data interpolation in two-dimensional space. That motivated us to go on a refactoring session and decouple the value interpolation work from the shading code. Essentially, we've arrived at the idea of [_programmable shaders_][shader] — similar to how the same concept is implemented in GPUs. 

[Version 0.1.7 on GitHub][version-0-1-7]{: .no-github-icon}

## Shading is all about interpolation 

If we look back at the progression of our features, we can notice that it's a story about data interpolation in two-dimensional space.

We started with [flat shading][post-cube-gets-light] where the color of the entire surface was determined by its orientation towards the light source. That approach allowed us to render faceted objects, such as a cube and a dodecahedron.

Next, we moved on to our first smooth surface object: a sphere. The flat shading approach didn't work with the sphere, so we implemented our first interpolated shading algorithm: [Gouraud shading][post-the-sphere-gets-smooth]. As you remember, the essence of Gouraud shading is that we calculate the color of each vertex of the mesh, and then interpolate these values to compute the color of the triangles' interior pixels.

Gouraud shading worked well for matte objects, but later we decided to extend our lighting model to support [glossy materials][post-first-shot-at-glossy-shapes], and interpolating a single color value was no longer enough. We moved to [Phong shading][post-phong-shading], where instead of color values we interpolate surface normals. 

Finally, we [added point lights][post-point-lights] and discovered that normals alone were no longer sufficient. Along with the surface normal, we also had to interpolate the world coordinates of each pixel. 

## Generalized rasterizer pipeline 

Throughout the entire journey, the rendering pipeline itself stayed relatively stable. The thing that was changing was the set of data for interpolation, and the way we used interpolated values to calculate the colors of framebuffer pixels: 

* vertex colors — for Gouraud shading; 
* surface normals — for Phong shading; 
* surface normals and world-space positions — for Phong shading and point lights. 

Having made this observation, we decided to split our rendering code into two generally independent parts: the _rasterization pipeline_ and the _shader_:

![Rasterizer pipeline with shader]({{site.baseurl}}/assets/images/2026-07-21-shaders-generalize-render-pipeline/rasterizer-pipeline.svg)

In this picture, the rasterization pipeline doesn't know or care _what_ data it interpolates. All it needs to know is _how_ to interpolate these values. We represent this knowledge with the [`Interpolatable`][source-interpolatable] trait that provides the following functionality to the pipeline: 

* [`calc_coefficients()`][source-calc-coefficients] is called to calculate the slope and the intercept of the interpolator function; 
* [`interpolate()`][source-interpolate] is called to calculate the interpolated value at a given input value `x`. 

For convenience, we also provide a blanket implementation of `Interpolatable` for types that implement basic arithmetic operations: `Add<T, Output = T>`, `Sub<T, Output = T>` and `Mul<f32, Output = T>`.

The shader acts as a plugin to the rasterization pipeline. The pipeline turns to the shader when: 

* it needs to calculate the anchor points for interpolation for each mesh vertex. It calls [`Shader::shade_vertex()`][source-shade-vertex], passing in the vertex position and normal, and receives the anchor point value; 
* it needs to calculate the color of an individual pixel. In this case, it calls [`Shader::shade_pixel()`][source-shade-pixel], passing in the interpolated value calculated for that specific pixel, and receives the color value. 

In fact, such an organization of a rendering pipeline is nothing new. Modern GPUs solve this exact problem using a similar approach, called [_programmable pipeline_][graphics-pipeline]. Instead of one monolithic renderer, the pipeline is fixed machinery with two programmable shaders:

* The _vertex shader_ runs once per vertex. It receives one vertex's attributes (position, normal, ...) and outputs whatever per-vertex data the rest of the pipeline needs.
* The rasterizer — fixed-function hardware — figures out which pixels each triangle covers and _linearly interpolates_ the vertex shader's outputs across the triangle's interior.
* The _fragment shader_ runs once per covered pixel. It receives the interpolated data and computes the final color.

Our pipeline essentially implements the same idea as separate methods of a single [`Shader`][source-shader-trait] trait.

## Same pipeline, different shaders 

Separating the responsibilities between the rasterization pipeline and shaders has a pleasant side effect. Now we can have different shading models in the code living side by side. To demonstrate this capability, we've implemented both Gouraud and Phong shading models in [`GouraudShader`][source-gouraud-shader] and [`PhongShader`][source-phong-shader], respectively. 

Though we moved away from Gouraud shading to Phong quite a while ago, having both implementations works as a proof of concept of our new design. Simply by replacing the shader implementation, we can now make use of different shading models using the same rendering pipeline: 

<div class="still-compare">
<figure>
<img src="{{ "/assets/images/2026-07-21-shaders-generalize-render-pipeline/still-scene-phong.webp" | relative_url }}" alt="Phong shading" />
<figcaption>Phong shading</figcaption>
</figure>
<figure>
<img src="{{ "/assets/images/2026-07-21-shaders-generalize-render-pipeline/still-scene-gouraud.webp" | relative_url }}" alt="Gouraud shading" />
<figcaption>Gouraud shading</figcaption>
</figure>
</div>

These pictures look very similar, but if you look closely, you can see Gouraud-specific artifacts in the specular highlight in the right picture, proving that we do indeed invoke different shading models. 

## Wrapping up phase 2

With that refactoring, we're reaching the end of Phase 2. Next we're going to make a quick recap of what's been done and lay out the plans for the future of this project. 


[post-point-lights]: {{site.baseurl}}/{% post_url 2026-07-11-introducing-point-lights %}
[post-phong-shading]: {{site.baseurl}}/{% post_url 2026-06-06-phong-shading-natural-highlights %}
[post-the-sphere-gets-smooth]: {{site.baseurl}}/{% post_url 2026-05-29-the-sphere-gets-smooth %}
[post-cube-gets-light]: {{site.baseurl}}/{% post_url 2026-05-22-the-cube-gets-light %}
[post-first-shot-at-glossy-shapes]: {{site.baseurl}}/{% post_url 2026-06-03-first-shot-at-glossy-shapes %}
[version-0-1-7]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.1.7
[source-shader-trait]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.7/src/lib.rs#L86
[source-shade-vertex]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.7/src/lib.rs#L89
[source-shade-pixel]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.7/src/lib.rs#L90
[source-interpolatable]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.7/src/framebuffer/interpolator.rs#L5
[source-calc-coefficients]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.7/src/framebuffer/interpolator.rs#L6
[source-interpolate]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.7/src/framebuffer/interpolator.rs#L7
[source-phong-shader]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.7/src/shaders.rs#L9
[source-gouraud-shader]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.7/src/shaders.rs#L28
[graphics-pipeline]: https://en.wikipedia.org/wiki/Graphics_pipeline
[shader]: https://en.wikipedia.org/wiki/Shader
