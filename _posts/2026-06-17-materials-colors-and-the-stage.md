---
layout: post
title: "Materials, Colors, and the Scene"
date: 2026-06-17 09:00:00 +0200
authors: Sergey and Cursor
---

We're starting [Phase 2][post-planning-phase-2] by updating the general look to match the color palette of the [three.js Geometry Browser][geometry-browser]. That change makes us revisit our Phong lighting implementation to make the material configuration more flexible. 

[Version 0.1.0 on GitHub][version-0-1-0]{: .no-github-icon}

## What you will see

The [`animated-scene`][source-animated-scene] demo is still the familiar torus scene, but now rendered with a different material and background setup: 

![Torus render after Phase 2 material and color pipeline updates](https://github.com/tindandelion/rust-3d-rasterizer/releases/download/0.1.0/scene.webp)

## Revisiting the material setup 

We've introduced the [`Material`][source-material-0-0-13] concept when we first tackled [glossy shading][post-first-shot-at-glossy-shapes]. Back then, it was very simple: just enough to demonstrate the new ability to render specular highlights. Moreover, for a while we lived with rather awkward design choices where the material specification and lighting equations were weirdly split between `Shape`, `Material`, and an artificial `BlinnLightModel` data type in the [`lighting`][source-lighting] module. 

If we combine all those disjoint pieces together, it's fair to say that we had a simple single-color material setup where the lighting equation was scaling this color according to the formula: 

$$
\begin{aligned}
I &= k_a + k_d \left(\max(\mathbf{l}\cdot\mathbf{n}, 0) + \max(\mathbf{n}\cdot\mathbf{h}, 0)^s\right) \\
\mathbf{C}_{out} &= I \cdot \mathbf{C}_{base}
\end{aligned}
$$

where $\mathbf{l}$ is the unit direction toward the light, $\mathbf{n}$ is the unit surface normal, and $\mathbf{v}$ is the unit direction toward the eye (camera). Here $\mathbf{h}$ is the Blinn half-vector between light and view directions (see our [Blinn-Phong explanation][post-first-shot-at-glossy-shapes-blinn-phong]):

$$
\mathbf{h} = \frac{\mathbf{l} + \mathbf{v}}{\lVert \mathbf{l} + \mathbf{v} \rVert}
$$

### New improved material 

Unlike our previous implementation, now we're moving towards a more flexible material specification. Instead of a single color and a couple of control parameters, we're introducing _different colors_ for each lighting component, matching the [`MeshPhongMaterial`][mesh-phong-material] from [three.js][threejs] (see [`default_material`][source-default-material] in our code): 

* `emissive` is a color of the material that's unaffected by other lighting. Generally speaking, it's the color of the light the object emits by itself, in absence of any light sources. This parameter replaces our previous concept of _ambient color_, serving the same purpose: give some color to the parts of the object that stay in deep shadow. 
* `diffuse` is the base color of the object's body. 
* `specular` and `shininess` are the parameters of the specular highlight: its color and its sharpness, respectively. 

The overall lighting formula now scales each color separately and then blends them all together: 

$$
\begin{aligned}
k_d &= I_L \cdot \max(\mathbf{l}\cdot\mathbf{n}, 0) \\
k_s &= I_L \cdot \max(\mathbf{n}\cdot\mathbf{h}, 0)^s \\
\mathbf{C}_{out} &= \mathbf{C}_{emissive} + k_d\,\mathbf{C}_{diffuse} + k_s\,\mathbf{C}_{specular}
\end{aligned}
$$

The same vector notation is used here: $\mathbf{l}$ toward light, $\mathbf{n}$ surface normal, $\mathbf{v}$ toward eye, and $\mathbf{h}$ as the Blinn half-vector. The scalar $I_L$ is the incoming light intensity.

## Discovering linear color space 

Color is a surprisingly complex subject when it comes to computer graphics. We're not diving deeply into color theory yet — it hasn't mattered much for our project so far — but there's one very important detail we've discovered: _linear color space_ vs _sRGB color space_. That discovery revealed that we've been doing lighting calculations slightly wrong. Not in terms of mathematics, but in terms of input values. Let's look at the problem in detail. 

On one hand, we have the equations that define the lighting model (such as the ones above). All of them take _intensity_ of incoming light as the input, and then calculate the intensity of the reflected light. When color comes into play, we can represent it as a triplet of intensities `(r, g, b)`: one intensity value for each color channel. That implies that the color is a linear value: if we have a color $C_0$ and we divide it by 2, the result is a color $C_1$ with half of the intensity. The same rule applies for addition of color values. 

On the other hand, we have our beloved [_sRGB_][srgb] encoding for colors that is widely used today to specify color values. But the key word here is _encoding_: numeric values for `r`, `g`, and `b` channels are used to represent specific intensities, but **this encoding is not linear**! In particular, darker intensities are over-represented: 

* The bottom half of sRGB values (0–127) maps to only about the bottom 5% of linear light
* The top half (128–255) covers the remaining 95%

What it means in practice is that you shouldn't use the sRGB values directly in the lighting equations because of that non-linearity. For example, if you have sRGB gray color $C_0$ encoded `(128, 128, 128)` and you halve it, you'll get the color code $C_1$ `(64, 64, 64)`. But the _actual_ color intensity that's represented by that triplet is going to be darker than if you took the intensity of $C_0$ and divided it by two. 

The bottom line is that sRGB is **an encoding** and so arithmetic operations over its values have no physical sense. In order to perform mathematics with colors, we first need to convert sRGB values into _linear color space_, where components `r`, `g`, and `b` represent actual intensities. 

Fortunately, the algorithm for such conversion is rather simple and [standardized][srgb-standard]. For each channel, treat the sRGB byte value as a normalized quantity $c_s \in [0, 1]$ (divide by 255), then decode to linear light $c_{\text{lin}}$:

$$
c_{\text{lin}} = \begin{cases}
c_s / 12.92 & \text{if } c_s \le 0.04045 \\[4pt]
\left(\dfrac{c_s + 0.055}{1.055}\right)^{2.4} & \text{if } c_s > 0.04045
\end{cases}
$$

To store the shaded result back into an image, encode linear values back to sRGB with the inverse mapping:

$$
c_s = \begin{cases}
12.92 \cdot c_{\text{lin}} & \text{if } c_{\text{lin}} \le 0.0031308 \\[4pt]
1.055 \cdot c_{\text{lin}}^{1/2.4} - 0.055 & \text{if } c_{\text{lin}} > 0.0031308
\end{cases}
$$

Apply the same formula independently to each of the `r`, `g`, and `b` channels.

### Implementation

Having discovered that important subject, we introduced a new type to represent color in linear space: [`Color`][source-lighting-color]. We've also moved the definition of arithmetic operations, such as `*` and `+`, to that data type, previously defined over `Rgb` values. So now we have two data types to represent colors in our program, with different use cases: 

* `Color` represents color in linear space; we can do mathematical operations with these values; 
* `Rgb` represents color in sRGB encoding; we use these values to represent color constants and to store the rendered image in a binary format. 

There are also `From` and `Into` implementations that convert between these two data types and implement in code the formulas above.

To see the effect this change had in practice, compare these two images: 

<div class="still-compare">
<figure>
<img src="{{ "/assets/images/2026-06-17-materials-colors-and-the-stage/still-scene-rgb.webp" | relative_url }}" alt="Still scene rendered in non-linear RGB composition" />
<figcaption>Shading math in sRGB</figcaption>
</figure>
<figure>
<img src="{{ "/assets/images/2026-06-17-materials-colors-and-the-stage/still-scene-linear.webp" | relative_url }}" alt="Still scene rendered in linear color space before output conversion" />
<figcaption>Shading math in linear color space</figcaption>
</figure>
</div>

The main effect is that the dark side of the sphere is lighter, and the border between the lit and dark sides is more pronounced. 

## What's next 

With the completion of this step, we have a more flexible [`Material`][source-material] data type and correct shading calculations. Our next step is to add three directional light sources to the scene, to match the setup of the [Geometry Browser][geometry-browser] from `three.js`. 

[post-planning-phase-2]: {{site.baseurl}}/{% post_url 2026-06-17-planning-phase-2 %}
[post-first-shot-at-glossy-shapes]: {{site.baseurl}}/{% post_url 2026-06-03-first-shot-at-glossy-shapes %}
[post-first-shot-at-glossy-shapes-blinn-phong]: {{site.baseurl}}/{% post_url 2026-06-03-first-shot-at-glossy-shapes %}#blinnphong-reflection-model
[version-0-1-0]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.1.0
[source-material-0-0-13]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.13/src/lighting.rs#L5
[source-lighting]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/lighting.rs#L1
[source-material]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/lighting.rs#L10
[source-lighting-color]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/lighting/color.rs#L5
[source-default-material]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/lib.rs#L46
[source-framebuffer-clear]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/framebuffer.rs#L43
[source-still-scene]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/bin/still-scene/main.rs#L31
[source-animated-scene]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/bin/animated-scene.rs#L31
[threejs]: https://threejs.org/
[geometry-browser]: https://threejs.org/docs/scenes/geometry-browser.html
[mesh-phong-material]: https://threejs.org/docs/#api/en/materials/MeshPhongMaterial
[srgb]: https://en.wikipedia.org/wiki/SRGB
[srgb-standard]: https://en.wikipedia.org/wiki/SRGB#Specification
