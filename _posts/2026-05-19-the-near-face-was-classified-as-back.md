---
layout: post
title: "The Bug: The Near Face Was Classified as Back"
date: 2026-05-19 
authors: Sergey and Cursor
---

Version 0.0.6 shipped the filled, six-color cube — a real milestone. The facing bug hid in plain sight until we started the lighting milestone on top of it. The quad rasterizer was fine; we were classifying which faces are visible with the dot-product sign backwards relative to our camera convention.

This is a small patch release, not a feature line item. We are documenting it because the mistake is easy to repeat and instructive if you are learning facing tests.

[Version 0.0.7 on GitHub][version-0-0-7]{: .no-github-icon}

## What you will see

The overall picture didn't change much. The bug was very hard to spot at this stage - only when we started to [work on lighting]   (#how-we-spotted-the-bug), did we notice things going wrong. 

The tumbling animation still culls back facets; they are just the facets that _truly point away now_.

![Faceted cube with corrected front-face culling](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.7/doc/output/current.webp)

## How we spotted the bug

At first, everything went according to the plan. Right after the filled-cube release we started on the next item in the [project breakdown][project-breakdown]: Cube: basic shading — simple lighting on the faceted cube.

While placing a directional light in the scene, Sergey got confused and then suspicious. No matter how he aimed it, he could not get the cube to read as lit from the front the way he expected. That felt wrong long before we had a precise diagnosis; it was the nudge to stop tuning light vectors and ask whether the pipeline was showing the faces he thought it was.

We went back and forth for a while trying to put the light in the right place. But no matter how we tried, it looked like the light was lighting up the wrong side of the cube.

Eventually we stepped back from lighting and investigated our back-face culling. To make the picture clear, we added [`still-unit-cube`][source-still-unit-cube] — a small export binary that renders the default unit cube at identity pose, square-on through the same orthographic camera as the main exporters, with no tumble and no extra transform. We also changed [`CUBE_FACE_PALETTE`][source-cube-palette] so the caps toward and away from the camera are impossible to confuse: deep blue on the −Z face (slot 0, the near cap) and red on the +Z face (slot 1, the back cap).

We ran the still and opened the WebP. The square in the middle of the frame was **red** — we were painting the back of the box.

Only then did we go hunting in [`CubeFace::is_back`][source-is-back-0-0-6] and the tests that had been feeding the wrong view vector into the facing check. The sections below are that chase.

## The convention we already agreed on

Our orthographic camera looks into the scene along +Z, as per[`Camera::direction`][source-camera-direction]. 

Each [`CubeFace`][source-cube-face] stores an outward unit normal $\mathbf{n}$. For the cap closest to the eye (the −Z side of a unit cube centered at the origin), $\mathbf{n}$ points toward −Z — opposite to $\mathbf{v}$.

![Outward normal on the near cap points opposite to into-scene view (+Z)]({{site.baseurl}}/assets/images/facing-convention-n-v.svg)

So for a facet that should be drawn when you look down +Z:

$$
\mathbf{n} \cdot \mathbf{v} < 0
$$

The outward normal and the into-scene view direction point against each other. That is front-facing in our setup.

## What went wrong

[`CubeFace::is_back`][source-is-back-0-0-6] had the sign convention backwards. It returned true when $\mathbf{n} \cdot \mathbf{v} < 0$ — but that is exactly the condition we just agreed means front-facing. In other words, **`is_back` labeled front-facing facets as back-facing** and treated genuinely back-facing facets as “not back.”

[`Cube::visible_faces`][source-visible-faces-0-0-6] then kept every face with `!is_back`, i.e. $\mathbf{n} \cdot \mathbf{v} \geq 0$. So the pipeline culled the cap toward the camera and kept the cap on the far side of the box.

For the default camera:

| Cap | Outward $\mathbf{n}$ | $\mathbf{n} \cdot \mathbf{v}$ with $\mathbf{v} = +Z$ | Actually | `is_back` said | `!is_back` kept? |
|-----|----------------------|------------------------------------------------------|----------|----------------|------------------|
| Near (−Z) | −Z | < 0 | front | back | no — culled |
| Far (+Z) | +Z | > 0 | back | not back | yes — filled |

The helper was not merely misnamed: its predicate was the inverse of back-facing for outward normals and an into-scene view vector. We were filling the back of the box and dropping the side facing the viewer — exactly what the blue/red still had shown.

## Why the tests did not catch it sooner

Several unit tests used `Vec3::NEG_Z` as the view direction and asserted counts like five visible faces from the front. That vector is the opposite of [`Camera::direction`][source-camera-direction]. With −Z as $\mathbf{v}$, the wrong inequality accidentally labels the near cap as visible and the far cap as back — so the tests passed while production code (which uses +Z) painted the wrong side.

An older check only required that some visible quad matched the −Z corner layout, using the same inverted view vector. It never required that only the near cap survived culling with the real camera axis.

Sergey added [`looking_at_cube_from_front`][source-looking-from-front]: default cube, `look_along_z_axis = Vec3::Z`, exactly one visible face, and that face’s corners are the −Z quad. That test failed immediately and pinned the bug.

## The fix

We replaced the inverted rule with an explicit [`is_front_facing`][source-is-front-facing] on 0.0.7:

$$
\text{front-facing} \iff \mathbf{n} \cdot \mathbf{v} < 0
$$

[`Cube::visible_faces`][source-visible-faces] filters on that predicate only. Grazing facets ($\mathbf{n} \cdot \mathbf{v} = 0$) are excluded as well, which matters for fills: from a cardinal view you should see one cap, not five side faces edge-on.

[`Cube::visible_edges`][source-visible-edges] now uses the same front-facing rule so edge and fill culling stay aligned with `Camera::direction`. The old `is_back` helper is gone.

Tests that describe “from the front” now take `Vec3::Z`, matching the camera. An integration test renders the default unit cube through [`draw_faces`][source-draw-faces] and compares the framebuffer to a hand-built golden image — one blue square for palette slot 0 (−Z cap), no WebP snapshot required.

## Next steps

This release was a small detour: correct facing, then back to the roadmap. The [project breakdown][project-breakdown] still has Cube: basic shading next — keep the quad fill path for a while, add a light direction and a simple diffuse term so facets read as lit planes rather than flat palette colors. Same orthographic camera, still no depth buffer. After that comes the sphere milestone and the triangle refactor; for now we pick up where the filled-cube milestone left off.

[project-breakdown]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/doc/planning/project-breakdown.md
[version-0-0-7]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.7
[source-still-unit-cube]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.7/src/bin/still-unit-cube.rs#L1
[source-cube-palette]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.7/src/lib.rs#L36
[source-camera-direction]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.7/src/ortho_camera.rs#L75
[source-cube-face]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.7/src/scene/cube/face.rs#L9
[source-is-back-0-0-6]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.6/src/scene/cube/face.rs#L45
[source-visible-faces-0-0-6]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.6/src/scene/cube.rs#L67
[source-is-front-facing]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.7/src/scene/cube/face.rs#L47
[source-visible-faces]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.7/src/scene/cube.rs#L72
[source-visible-edges]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.7/src/scene/cube.rs#L54
[source-looking-from-front]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.7/src/scene/cube.rs#L190
[source-draw-faces]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.7/src/lib.rs#L65
