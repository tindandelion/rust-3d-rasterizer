---
layout: post
title: "Materials, Colors, and the Stage"
date: 2026-06-17 09:00:00 +0200
authors: Sergey and Cursor
---

We're starting the [Phase 2][post-planning-phase-2] with updating the general look to match the color palette of three.js geometry browser. That change makes us revisit our implementation for Phong lighting to make the material configuration more flexible. 

[Version 0.1.0 on GitHub][version-0-1-0]{: .no-github-icon}

## What you will see

The demo render is still the familiar torus scene, but now rendered with different material and background setup: 

![Torus render after Phase 2 material and color pipeline updates](https://github.com/tindandelion/rust-3d-rasterizer/releases/download/0.1.0/scene.webp)

## Revisiting the material setup 

We've introduced the [`Material`][source-material-0-0-13] concept when we first tackled [glossy shading][post-first-shot-at-glossy-shapes]. Back then, it was very simple: just enough to demonstrate the new ability to render specular highlights. Moreover, for a while we lived with a rather awkward code design choices where the material specification and lighting equations were weirdly split between `Shape`, `Material`, and an artificial `BlinnLightModel` data types. 

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

Unlike our previous implementation, now we're moving towards a more flexible material specification. Instead of a single color and a couple of control parameters, we're introducing _different colors_ for each lighting component, matching the `MeshPhongMaterial` from `three.js`: 

* `emissive` is a color of the material that's unaffected by other lighting. Generally speaking, it's the color of the light the object emits by itself, in absence of any light sources. This parameter replaces our previous concept of _ambient color_, serving the same purpose: give some color to the parts of the object that stay in a full shade. 
* `diffuse` is the base color of the object's body. 
* `specular` and `shininess` are the parameters of the specular highlight: its color and its sharpness, respectively. 

The overal lighting formula now scales each color separately and then blends them all together: 

$$
\begin{aligned}
k_d &= I_L \cdot \max(\mathbf{l}\cdot\mathbf{n}, 0) \\
k_s &= I_L \cdot \max(\mathbf{n}\cdot\mathbf{h}, 0)^s \\
\mathbf{C}_{out} &= \mathbf{C}_{emissive} + k_d\,\mathbf{C}_{diffuse} + k_s\,\mathbf{C}_{specular}
\end{aligned}
$$

The same vector notation is used here: $\mathbf{l}$ toward light, $\mathbf{n}$ surface normal, $\mathbf{v}$ toward eye, and $\mathbf{h}$ as the Blinn half-vector.

## Discovering linear color space 

Color is a surprisingly complex subject when it comes to computer graphics. We're not diving into it yet since it doesn't apply to our project that much so far, but there's one very important detail we've discovered: _linear color space_ vs _sRGB color space_, and that discovery revealed to us that we've been doing ligthing calculations slightly wrong. Not in terms of mathematics, but in terms of input values. Let's look at the problem in detail. 

On one hand, we have the equations that define the lighting model (such as the ones above). All of them take _intensity_ of incoming light as the input, and then calcuate the intensity of the reflected light. When the color comes into play, we can represent is as a triplet of intensities `(r, g, b)`: one intensity value for each color channel. That implies that the color is a linear value: if I have a color $C_0$ and I divide it by 2, the result is a color $C_1$ with half of the intensity. Same rule applies for addition of color values. 

On the other hand, we have our beloved sRGB encoding for colors that is widely popular nowadays to specify color values. But the key word here is _encoding_: numeric values for `r`, `g` and `b` channels are used to represent specific intensities, but **this encoding is not linear**! In particular, darker intensities are over-represented: 

* The bottom half of sRGB values (0–127) maps to only about the bottom 5% of linear light
* The top half (128–255) covers the remaining 95%

What it means in practice is that you shouldn't use the sRGB values directly in the lighting equations because of that non-linearity. For example, if you have sRGB gray color $C_0$ encoded `(128, 128, 128)` and you halve it, you'll get the color code $C_1$ `(64, 64, 64)`. But the _actual_ color intensity that's represented by that triplet is going to be darker than if you took the intensity of $C_0$ and divided it by two. 

The bottom line is that arithmetic operations over sRGB values have no physical sense. In order to be able to perform mathematics with colors, we first need to convert sGRB values into _linear color space_, where components `r`, `g` and `b` represent actual intensity values. Luckily, the formula for such conversion is rather simple and [standardized][link-to-the-standard]: 

[insert here the formulas to convert sRGB into RGB]

Having discovered that important subject, we introduced a new type to represent the color in the linear space: [`Color`][link-to-code]. We've also moved the definition of arithmetic operations, such as `*` and `+`, to that data type, previously defined over values of `Rgb` data type. So now we have two data types to represent colors in our program: 

* `Color` represents the color in the linear space: we can do mathematic operations with these values; 
* `Rgb` represents the color in sRGB encoding; we use these values to represent color constants and to store rendered image as a binary array. 

There are also implementations for `From` and `Into` that allow us to convert between these two data types, that implement in code the formulas from above. 

A quick demonstration of what effect moving to linear color space has when it comes to rendering: 

<div class="still-compare">
<figure>
<img src="{{ "/assets/images/2026-06-17-materials-colors-and-the-stage/still-scene-rgb.webp" | relative_url }}" alt="Still scene rendered in non-linear RGB composition" />
<figcaption>Rendered with operations over sRGB</figcaption>
</figure>
<figure>
<img src="{{ "/assets/images/2026-06-17-materials-colors-and-the-stage/still-scene-linear.webp" | relative_url }}" alt="Still scene rendered in linear color space before output conversion" />
<figcaption>Rendered with operations over linear color space</figcaption>
</figure>
</div>

The main effect is that now we have the dark side of the sphere lighter, and the border between the lit and dark side is more pronounced. 

## What's next 

With the completion of this step, we have a more flexible `Material` data type and correct shading calculation. Our next step is going to be to add several directional light sources to the scene. 

[post-planning-phase-2]: {{site.baseurl}}/{% post_url 2026-06-17-planning-phase-2 %}
[post-first-shot-at-glossy-shapes]: {{site.baseurl}}/{% post_url 2026-06-03-first-shot-at-glossy-shapes %}
[post-first-shot-at-glossy-shapes-blinn-phong]: {{site.baseurl}}/{% post_url 2026-06-03-first-shot-at-glossy-shapes %}#blinnphong-reflection-model
[version-0-1-0]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.1.0
[source-material-0-0-13]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.13/src/lighting.rs#L5
[source-lighting]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/lighting.rs#L9
[source-lighting-color]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/lighting/color.rs#L5
[source-default-material]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/lib.rs#L46
[source-framebuffer-clear]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/framebuffer.rs#L43
[source-still-scene]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/bin/still-scene/main.rs#L52
[source-animated-scene]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/bin/animated-scene.rs#L31
