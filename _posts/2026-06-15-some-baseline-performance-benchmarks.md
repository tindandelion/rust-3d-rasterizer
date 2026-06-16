---
layout: post
title: "Some Baseline Performance Benchmarks"
date: 2026-06-15 10:00:00 +0200
authors: Sergey and Cursor
---

Before moving on to phase 2, we decided to measure the performance of the rasterizer. We're not diving into the details yet: this post is mostly to establish and record a baseline to measure against in later work.

## Rasterizer performance benchmarks

At first glance, rendering speed depends on two factors:

* The level of detail of the rendered shape: how many vertices and facets does it contain?
* The pixel resolution of the resulting image.

We've created a simple performance evaluation harness in [`performance-eval`][source-performance-eval] that measures the rasterizer's _frames per second (FPS)_ while running a miniature version of the [animation pipeline][source-animated-scene].

### FPS vs. facet count

Using a fixed 800×600 framebuffer and scaling the torus detail level, we get the following results:

| Vertices | Facets | FPS |
|----------|--------|-----|
| 384 | 768 | 659.08 |
| 1536 | 3072 | 544.20 |
| 6144 | 12288 | 361.44 |
| 24576 | 49152 | 184.38 |
| 98304 | 196608 | 77.68 |

![FPS vs facet count at fixed 800×600 resolution]({{ "/assets/images/2026-06-15-some-baseline-performance-benchmarks/mesh-detail-fps.svg" | relative_url }}){: .chart-full-width }

### FPS vs. image resolution

Using a fixed coarse torus mesh (`torus(24, 16)`, 384 vertices), we swept the framebuffer up to 8K resolution:

| Dimensions | Pixels | FPS |
|------------|--------|-----|
| 800×600 | 480000 | 704.76 |
| 1280×720 | 921600 | 502.97 |
| 1920×1080 | 2073600 | 244.07 |
| 3840×2160 | 8294400 | 65.38 |
| 7680×4320 | 33177600 | 16.65 |

![FPS vs pixel count with fixed coarse mesh]({{ "/assets/images/2026-06-15-some-baseline-performance-benchmarks/resolution-fps.svg" | relative_url }}){: .chart-full-width }

## We don't act on it yet

Although both graphs show that performance worsens as mesh detail and resolution increase, we're not drawing conclusions yet. At some point we'll take a closer look at improving it. For now, we'll just record the baseline and move to [planning the phase 2 of our project][post-planning-phase-2]. 

[post-planning-phase-2]: {{site.baseurl}}/{% post_url 2026-06-16-planning-phase-2 %}
[source-performance-eval]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/bin/performance-eval.rs#L1
[source-animated-scene]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/bin/animated-scene.rs#L1
