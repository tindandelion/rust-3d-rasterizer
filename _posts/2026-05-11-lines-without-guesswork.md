---
layout: post
title: "Lines Without Guesswork"
date: 2026-05-11 08:50:00 +0200
authors: Sergey and Cursor
---

We shipped the second milestone of the rasterizer. This one feels like a real step forward: instead of writing one known pixel, we now draw full line segments across the framebuffer and export the result as a valid lossless WebP.

[Current version (0.0.2) on GitHub][version-0-0-2]{: .no-github-icon}

## What Changed In Practice

Version 0.0.1 proved the output pipeline. Version 0.0.2 turns that pipeline into an actual drawing loop.

The key change is a line primitive, [`draw_line`][source-draw-line], implemented on top of the existing framebuffer and guarded [`set_pixel`][source-set-pixel]. We also introduced a tiny [`Point`][source-point] type and kept color usage explicit with `Rgb::BLACK` and `Rgb::WHITE`.

That combination gives us a cleaner mental model for upcoming milestones:

1. choose geometry in scene space,
2. convert to integer pixel endpoints,
3. draw endpoint-inclusive segments,
4. let out-of-bounds writes fall through safely.

The result is still intentionally simple and easy to inspect:

![Current render output](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/main/doc/output/current.webp)

## Why DDA, Not Something Fancier

A notable decision in this milestone was algorithm choice. We briefly had an integer Bresenham path while iterating, then switched to a DDA-style implementation because it better matches the project plan and is easier to reason about while we are still building fundamentals.

In code, that means:

- `steps = max(|dx|, |dy|)`,
- walk `t` from `0` to `1`,
- interpolate floating-point coordinates,
- round to nearest integer pixel.

For this stage, that trade-off is exactly what we wanted: prioritize readability and predictable behavior over micro-optimizing the inner loop before profiling says it matters.

## Testing The Shape, Not Just The Function

This milestone also improved tests in a useful way. Instead of only checking individual bytes, we added several line-focused unit tests and a tiny ASCII view helper so expected pixel patterns are readable at a glance.

That gave us confidence in the cases that matter right now:

- horizontal, vertical, and diagonal segments,
- endpoint inclusivity (forward and reverse),
- clipping behavior when endpoints land outside the framebuffer.

The broader integration test still verifies that the binary writes a decodable WebP, so we keep both levels of feedback: local geometry correctness and end-to-end artifact validity.

## Why The New Output Looks Like A Flower

The rendered scene is now a radial pattern: many spokes from the image center to a circle. In code, that happens in [`draw_flower`][source-draw-flower] by iterating angles over [`TAU`][tau-docs] and issuing one `draw_line` per spoke.

We originally discussed a crossed square for this milestone. That remains a good regression-style scene, but the radial image turned out to be better for quick visual checks:

- missing pixels are obvious,
- endpoint handling is easier to spot,
- small directional asymmetries stand out immediately.

Because of that, the milestone checklist in the planning doc was updated to mark Drawing lines complete with the radial-segment artifact description.

## What 0.0.2 Unlocks

This release does not yet include projection, meshes, or triangle filling. But it gives us a dependable primitive that all of that will rely on.

The next milestone is orthographic cube projection, where this line path becomes the first wireframe backbone instead of a standalone demo.

[version-0-0-2]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.2
[source-draw-line]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.2/src/framebuffer.rs#L47-L70
[source-set-pixel]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.2/src/framebuffer.rs#L37-L45
[source-point]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.2/src/framebuffer.rs#L4
[source-draw-flower]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.2/src/main.rs#L38-L59
[tau-docs]: https://doc.rust-lang.org/std/f64/consts/constant.TAU.html
