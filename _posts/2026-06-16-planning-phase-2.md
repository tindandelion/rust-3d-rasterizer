---
layout: post
title: "Planning Phase 2"
date: 2026-06-16 08:00:00 +0200
authors: Sergey and Cursor
---

[Phase 1][post-from-pixel-to-torus] laid the basic groundwork of the 3D rasteriser pipeline. Now we're ready to make a plan for the next phase. By our intention, we're not going to introduce major new concepts during phase 2. Instead, we'd like to focus on the concepts we've already implemented, revisit them and deepen our undestanding of certain subects. 

## Evolving the rasteriser 

During the phase 1, we've built something that's called a _Minimal Viable Product (MVP)_: a first version of a product that exercises the idea end-to-end, but oversimplifies things at times and cuts some corners in favor of the development speed. 

In phase 2, we don't plan to introduce any major new features, such as textures or shadows. Instead, we'd like to use this phase as a driver to explore certain topics in more depth. If phase 1 was like building a walking skeleton, in this phase we'll add more meat to it. 

The state of the codebase is also going to inform a few areas for improvement. In some parts, our current design decisions feel a little awkward, but we don't have enough knowledge yet to make meaningful improvements. So sometimes we'll pick the topics for exploration that will force us to revisit the current design and think how to make it more flexible to support new behaviour.  

## Picking the north star 

> Good artists copy, great artists steal — Pablo Picasso

There are plenty of existing 3D libraries to steal ideas from, and we would like to use this opportunity to learn how things are implemented by more knowledgeable developers. Our choice is [`three.js`][threejs] — a JavaScript library for building 3D graphics applications in the browser.

Very helpfully, `three.js` includes an interactive [Geometry Browser][geometry-browser]: the demo application that demonstrates its capabilities. Not surprisingly, one of the demos is a [rotating torus][torus-geometry], similar to what we've been building all along.

<figure class="demo-embed">
  <iframe
    src="{{ '/assets/demos/geometry-browser-torus.html' | relative_url }}"
    title="three.js Geometry Browser torus reference"
    loading="lazy"
  ></iframe>
  <figcaption>Our north-star scene — drag to orbit. Trimmed from the official <a href="https://threejs.org/docs/scenes/geometry-browser.html#TorusGeometry">Geometry Browser</a> torus demo.</figcaption>
</figure>

This demo is going to be our visual north star: at the end of phase 2 we would like to be able to render something similar, shamelessly stealing ideas for the scene setup and the color palette. We don't aim to replicate it exactly, though: only borrow the parts that seem interesting to explore. 

At first glance, the [following topics][project-breakdown] look worth touching:

* **More flexible materials**. Our implementation right now doesn't give us enough flexibility to set the material parameters; we'd like to extend the `Material` struct to make it more flexible; 
* **Multiple lights per scene**. Torus demo from `three.js` uses three directional lights in the scene. Our pipeline doesn't support multiple lights. 
* **Point lights**. Although they are not used in the torus demo from `three.js`, it's interesting to explore this type of light and see how it changes both the code and the render result. 
* **Perspective projection**. That's a tentative goal at the moment. Though it's definitely worth pursuing, we might postpone it to later time. 

## What comes next

We'll start from the most visual part: borrowing the color palette from `three.js` and extending our `Material` data type and shading equations to match. 


[post-from-pixel-to-torus]: {{site.baseurl}}/{% post_url 2026-06-13-from-pixel-to-torus %}
[post-baseline-benchmarks]: {{site.baseurl}}/{% post_url 2026-06-15-some-baseline-performance-benchmarks %}
[threejs]: https://threejs.org/
[geometry-browser]: https://threejs.org/docs/scenes/geometry-browser.html#TorusGeometry
[torus-geometry]: https://threejs.org/docs/#api/en/geometries/TorusGeometry
[perspective-projection]: https://en.wikipedia.org/wiki/3D_projection#Perspective_projection
[orthographic-projection]: https://en.wikipedia.org/wiki/Orthographic_projection
[project-breakdown]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/doc/planning/project-breakdown.md
[project-spec]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/doc/planning/project-spec.md
