---
layout: post
title: "Some Baseline Performance Benchmarks"
date: 2026-06-15 10:00:00 +0200
authors: Sergey and Cursor
---

Before moving on to the phase 2, we decided to measure and record some of the performance benchmarks of the rasterizer. We're not diving into the details yet: this post is mostly to establish the baseline to measure against in later work. 

## Rasterizer performance benchmark

At the first glance, the speed of rendering is predicated on two factors: 

* The level of details of the rendered shape: how many vertices and facets does it contain? 
* The pixel resolution of the resulting image. 

We've created a simple performance evaluation harness in `performance-eval` that measures _frame per second (FPS)_ rate of the rasterizer, when running a miniature version of the animation pipeline. 

### FPS per facet count 

Using a fixed 800x600 framebuffer and scaling the torus detail level, we get the following results: 

| Vertices | Facets | FPS |
|----------|--------|-----|
| 384 | 768 | 659.08 |
| 1536 | 3072 | 544.20 |
| 6144 | 12288 | 361.44 |
| 24576 | 49152 | 184.38 |
| 98304 | 196608 | 77.68 |

![FPS vs facet count at fixed 800×600 resolution]({{ "/assets/images/2026-06-15-some-baseline-performance-benchmarks/mesh-detail-fps.svg" | relative_url }})

### FPS per image resolution 

Using a fixed fixed coarse torus mesh (`torus(24, 16)`, 384 vertices), framebuffer swept from demo size up to 8K:

| Dimensions | Pixels | FPS |
|------------|--------|-----|
| 800×600 | 480000 | 704.76 |
| 1280×720 | 921600 | 502.97 |
| 1920×1080 | 2073600 | 244.07 |
| 3840×2160 | 8294400 | 65.38 |
| 7680×4320 | 33177600 | 16.65 |

![FPS vs pixel count with fixed coarse mesh]({{ "/assets/images/2026-06-15-some-baseline-performance-benchmarks/resolution-fps.svg" | relative_url }})

## We don't act on it yet 

Although both graphs show that the performance of the rasterizer worsens as we go to the higher level of details in both cases, we're not diving into the interpretations yet. At some point in the future, we'll have a closer look at improving its performance. For now, let's just take a note of the fact. 


[post-from-pixel-to-torus]: {{site.baseurl}}/{% post_url 2026-06-13-from-pixel-to-torus %}
[post-torus-takes-shape]: {{site.baseurl}}/{% post_url 2026-06-11-the-torus-takes-shape %}
[post-yet-another-shape-the-sphere]: {{site.baseurl}}/{% post_url 2026-05-27-yet-another-shape-the-sphere %}
[version-0-0-17]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.17
[benchmark]: https://en.wikipedia.org/wiki/Benchmark_(computing)
[fill-rate]: https://en.wikipedia.org/wiki/Glossary_of_computer_graphics#Fillrate
[flamegraph]: https://www.tindandelion.com/rust-text-compression/2025/01/12/profiling-with-flamegraphs.html
[source-performance-eval]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/bin/performance-eval.rs#L1
[source-stress-test]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/bin/stress-test.rs#L1
[source-animated-scene]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/bin/animated-scene.rs#L1
[source-framebuffer]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/framebuffer.rs#L6
[source-frame-count]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/lib.rs#L28
[source-shape-render]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/lib.rs#L48
[source-phong-shaded-triangle]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/framebuffer/phong_shaded_triangle.rs#L18
[source-torus]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/meshes/torus.rs#L20
