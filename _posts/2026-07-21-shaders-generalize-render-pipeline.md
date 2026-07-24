---
layout: post
title: "Shaders: Generalize Render Pipeline"
date: 2026-07-21 10:00:00 +0200
authors: Sergey and Cursor
tags: refactor
---

In this session, we have a look at our rasterizer implementation and notice that for quite a while it was a story about data interpolation in a two-dimentional space. That motivated us to go on a refactoring session, and decouple the value interpolation work from shading code. Essentially, we've arrived at the idea of _programmable shaders_ - similar to how the same concept is implemented in GPUs. 

[Version 0.1.7 on GitHub][version-0-1-7]{: .no-github-icon}

## Shading is all about interpolation 

If we look back at the progression of our features, we can notice that it's a story about data interpolation in two-dimentional space.

We started with [flat shading][cube-gets-light-post] where the shade of the entire surface was determined by its orientation towards the light source. That approach allowed us render faceted objects, such as a cube and a dodecahedron.

Next, we moved on to our first smooth surface object: a sphere. The flat shading approach didn't work with sphere, so we implemented our first interpolated shading algorithm: [Gouraud shading][sphere-gets-smooth-post]. As you remember, the essence of Gouraud shading is that we calculate the color of each vertex of the mesh, and then interpolate these values to compute the color of the triangles' interior pixels.

Gouraud shading worked well for matte objects, but later we decided to extend our lighting model to support [glossy materials][first-shot-at-glossy-materials-post], and interolating a single color value was no longer enough. We moved to [Phong shading][phong-shading-post], where instead of color values we interpolate surface normals. 

Finally, we [added point lights][point-lights-post] and discovered that only normals were no longer sufficient. Anong with the sufrace normal, we also had to interpolate the world coordinates of each pixel. 

## Generalized rasterizer pipeline 

Throughout the entire journey, the rendering pipeline itself stayed relatively stable. The thing that was changing is the set of data for interpolation, and the way we used interpolated values to calculate the colors of framebuffer pixels: 

* Vertex colors - for Gouraud shading; 
* Surface normals - for Phong shading; 
* Surface normals and world-space positions - for Phong shading and point lights. 

Having made this observation, we can split our rendering code into two generally independent parts: the _rasterization pipeline_ and the _shader_:

![Rasterizer pipeline with shader]({{site.baseurl}}/assets/images/2026-07-21-shaders-generalize-render-pipeline/rasterizer-pipeline.svg)

In this picture, the rasterization pipeline doesn't know or care _what_ data it interpolates. All it needs to know is _how_ to interpolate these values. We represent this knowledge with [`Interpolatable`][source-interpolatable] trait that provides the following functionality to the pipeline: 

* [`calc_coefficients()`][link-to-code] is called to calculate the slope and the intercept of the interpolator function; 
* [`interpolate()`] is called to calculate the interpolated value at a given input value `x`. 

For convenience, we also provide a blanket implementation of `Interpolatable` for types that implement basic arithmetic operations: `Add<T, Output = T>`, `Sub<T, Output = T>` and `Mul<f32, Output = T>`.

The shader acts as a plugin to the rasterization pipeline. The pipeline turns to the shader when: 

* it needs to calculate the anchor points for interpolation for each mesh vertex. It calls [`Shader::shade_vertex()`][link-to-code], passing in the vertex position and normal, and receives the anchor point value; 
* it needs to calculate the color of an individual pixel. In this case, it calls [`Shader::shade_pixel()`], passing in the interpolated value, calculated for that specific pixel, and receives the color value. 

In fact, such organization of a rendering pipeline is nothing new. Modern GPUs solve this exact problem using a similar approach, called [_programmable pipeline_][graphics-pipeline]. Instead of one monolithic renderer, the pipeline is fixed machinery with two programmable shaders:

* The _vertex shader_ runs once per vertex. It receives one vertex's attributes (position, normal, ...) and outputs whatever per-vertex data the rest of the pipeline needs.
* The rasterizer — fixed-function hardware — figures out which pixels each triangle covers and _linearly interpolates_ the vertex shader's outputs across the triangle's interior.
* The _fragment shader_ runs once per covered pixel. It receives the interpolated data and computes the final color.

Our pipeline essentially implements the same idea as separate methods of a single `Shader` trait.

## Same pipeline, different shaders 

Separating the responsibilities between the rasterization pipeline and shaders has a pleasant side-effect. Now we can have different shading models in the code living side by side. To demonstrate this capability, we've implemented both Gouraud and Phong shading models in  [`PhongShader`][source-phong-shader] and  [`GouraudShader`][source-gouraud-shader], respectively. 

Though we've moved away from Gouraud shading to Phong quite long ago, having both implementation works as a proof of concept of our new design. Simply by replacing the shader implementation, we can now make use of different shading models using the same rendering pipeline: 

<div class="still-compare">
<figure>
<img src="{{ "/assets/images/2026-07-21-shaders-generalize-render-pipeline/still-scene-phong.webp" | relative_url }}" alt="Phong shading" />
<figcaption>Phong shading</figcaption>
</figure>
<figure>
<img src="{{ "/assets/images/2026-07-21-shaders-generalize-render-pipeline/still-scene-gouraud.webp" | relative_url }}" alt="Point light" />
<figcaption>Gouraud shading</figcaption>
</figure>
</div>

These pictures look very similar, if you look closely, you can see Gouraud-specific artifacts of the specular highlight in the right picture. 

## Wrappping up phase 2

With that refactoring, we're reaching the end of Phase 2. Next we're going to make a quick recap of what's been done and lay out the plans for the future of this project. 


[post-point-lights]: {{site.baseurl}}/{% post_url 2026-07-11-introducing-point-lights %}
[post-point-lights-interpolation]: {{site.baseurl}}/{% post_url 2026-07-11-introducing-point-lights %}#interpolation-of-surface-points
[post-phong-shading]: {{site.baseurl}}/{% post_url 2026-06-06-phong-shading-natural-highlights %}
[post-the-sphere-gets-smooth]: {{site.baseurl}}/{% post_url 2026-05-29-the-sphere-gets-smooth %}
[post-cube-paints-faces]: {{site.baseurl}}/{% post_url 2026-05-18-the-cube-paints-its-six-faces %}
[post-linear-color]: {{site.baseurl}}/{% post_url 2026-06-19-materials-colors-and-the-stage %}#discovering-linear-color-space
[post-three-lights]: {{site.baseurl}}/{% post_url 2026-06-19-three-directional-lights %}
[post-baseline-benchmarks]: {{site.baseurl}}/{% post_url 2026-06-15-some-baseline-performance-benchmarks %}
[version-0-1-7]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.1.7
[source-shader-trait]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.7/src/lib.rs#L86
[source-interpolatable]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.7/src/framebuffer/interpolator.rs#L5
[source-shaded-triangle]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.7/src/framebuffer/shaded_triangle.rs#L15
[source-shaders-module]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.7/src/shaders.rs
[source-phong-shader]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.7/src/shaders.rs#L9
[source-gouraud-shader]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.7/src/shaders.rs#L28
[source-render-phong]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.7/src/lib.rs#L114
[source-render-gouraud]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.7/src/lib.rs#L105
[source-material-shade]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.7/src/lighting.rs#L30
[source-color]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.1.7/src/lighting/color.rs#L9
[graphics-pipeline]: https://en.wikipedia.org/wiki/Graphics_pipeline
[shader]: https://en.wikipedia.org/wiki/Shader
[wgpu]: https://crates.io/crates/wgpu
