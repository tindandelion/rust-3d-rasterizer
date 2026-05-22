---
layout: post
title: "The Bug: The Near Face Was Classified as Back"
date: 2026-05-19 10:00:00 +0200
authors: Sergey and Cursor
---

[Version 0.0.6][post-cube-paints-its-six-faces] shipped the filled, six-color cube — a real milestone. Back-face culling arrived one release earlier, in [Version 0.0.5][post-cube-sheds-hidden-edges], with [`CubeFace::is_back`][source-is-back-0-0-5] driving [`Cube::visible_edges`][source-visible-edges-0-0-5]. Little did we know the sign in that helper was backwards relative to [`Camera::direction`][source-camera-direction] — we had shipped the wrong facing test without noticing. The mistake stayed hidden until we started the lighting milestone on top of the filled cube.

We had to come up with a small patch release. We are documenting it because the mistake is easy to repeat and instructive if you are learning facing tests.

[Version 0.0.7 on GitHub][version-0-0-7]{: .no-github-icon}

## What you will see

The tumbling animation looks almost the same at a glance — still a faceted cube, still six colors, still back faces culled. The difference is which facets count as “back.” Believe it or not, **we were culling wrong faces up until now!**  

![Faceted cube with corrected front-face culling](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.7/doc/output/current.webp)

The bug is really subtle to spot on a simple scene like this. Only when we started to work on lighting did we spot oddities between what we expected to see and what was actually rendered. 

## How we spotted the bug

At first, everything went according to plan. Right after the filled-cube release we started on the next item in the [project breakdown][project-breakdown]: Cube: basic shading — simple lighting on the faceted cube.

While placing a directional light in the scene, Sergey got confused and then suspicious. No matter how he aimed it, the cube would not read as lit from the front the way he expected. That felt wrong long before we had a precise diagnosis — the nudge to stop tuning light vectors and ask whether the pipeline was showing the faces he thought it was.

Eventually we stepped back from lighting and investigated our back-face culling. To make the picture clear, we added [`still-unit-cube`][source-still-unit-cube] — a small export binary that renders the default unit cube in the identity pose, square-on through the same orthographic camera as the main exporters, with no tumble and no extra transform. We also changed [`CUBE_FACE_PALETTE`][source-cube-palette] so the faces toward and away from the camera are impossible to confuse: deep blue on the −Z face (slot 0, the near face) and red on the +Z face (slot 1, the back face).

We ran the binary and opened the WebP. We expected a **blue** square in the middle of the frame; it was **red** — we were painting the back of the box!

Only then did we go hunting in the [`is_back`][source-is-back-0-0-5] helper we added in 0.0.5 and the tests that had been feeding the wrong view vector into the facing check. The sections below are that chase.

## Our convention for face culling

Our orthographic camera looks into the scene along +Z, matching `Camera::direction`.

Each [`CubeFace`][source-cube-face] stores an outward unit normal $\mathbf{n}$. For the cap closest to the eye (the −Z side of a unit cube centered at the origin), $\mathbf{n}$ points toward −Z — opposite to $\mathbf{v}$.

![Outward normal on the near cap points opposite to into-scene view (+Z)]({{site.baseurl}}/assets/images/facing-convention-n-v.svg)

So for a facet that should be drawn when you look down +Z:

$$
\mathbf{n} \cdot \mathbf{v} < 0
$$

The outward normal and the into-scene view direction point against each other. That is front-facing in our setup.

## What went wrong

[`CubeFace::is_back`][source-is-back-0-0-5] had the sign convention backwards from the day we introduced it in 0.0.5. It returned true when $\mathbf{n} \cdot \mathbf{v} < 0$ — but that is exactly the condition we just agreed means front-facing. In other words, `is_back` labeled front-facing facets as back-facing and treated genuinely back-facing facets as “not back.”

[`Cube::visible_edges`][source-visible-edges-0-0-5] skipped edges on faces where `is_back` was true. When we added fills in 0.0.6, [`Cube::visible_faces`][source-visible-faces-0-0-6] kept every face with `!is_back`, i.e. $\mathbf{n} \cdot \mathbf{v} \geq 0$. Either way, the pipeline culled the faces toward the camera and kept the faces on the far side of the box.

With $\mathbf{v} = +Z$ as in production:

| Cap | Outward $\mathbf{n}$ | $\mathbf{n} \cdot \mathbf{v}$ | Actually | `is_back` said | `!is_back` kept? |
|-----|----------------------|-------------------------------|----------|----------------|------------------|
| Near (−Z) | −Z | < 0 | front | back | no — culled |
| Far (+Z) | +Z | > 0 | back | not back | yes — filled |

The helper was not merely misnamed: its predicate was the inverse of back-facing for outward normals and an into-scene view vector. We were filling the back of the box and dropping the side facing the viewer — exactly what the blue/red still had shown.

## Why the tests did not catch it sooner

The tests and production disagreed because they used opposite view vectors: several unit tests passed `Vec3::NEG_Z` as $\mathbf{v}$ and asserted counts like five visible faces from the front, while the camera supplies +Z through `Camera::direction`. With −Z as $\mathbf{v}$, the wrong inequality accidentally labels the near cap as visible and the far cap as back — so the tests passed while production painted the wrong side.

An older check only required that some visible quad matched the −Z corner layout, using the same inverted view vector. It never required that only the near cap survived culling with the real camera axis.

Sergey added [`looking_at_cube_from_front`][source-looking-from-front]: default cube, `look_along_z_axis = Vec3::Z`, exactly one visible face, and that face’s corners are the −Z quad. That test failed immediately and pinned the bug.

## The fix

We replaced the inverted rule with an explicit [`is_front_facing`][source-is-front-facing] on 0.0.7:

$$
\text{front-facing} \iff \mathbf{n} \cdot \mathbf{v} < 0
$$

[`Cube::visible_faces`][source-visible-faces] filters on that predicate only. Grazing facets ($\mathbf{n} \cdot \mathbf{v} = 0$) are excluded as well, which matters for fills: from a cardinal view you should see one cap, not five side faces edge-on.

`Cube::visible_edges` now uses the same front-facing rule as fills, so wireframe and face culling finally match `Camera::direction`. We first noticed the mistake in the filled exporters; the old `is_back` helper is gone.

Tests that describe “from the front” now take `Vec3::Z`, matching the camera. An integration test renders the default unit cube through [`draw_faces`][source-draw-faces] and compares the framebuffer to a hand-built golden image — one blue square for palette slot 0 (−Z cap), no WebP snapshot required.

## What comes next

This release was a small detour: correct facing, then back to basic shading on the [project breakdown][project-breakdown]. That landed in [Version 0.0.8][post-the-cube-gets-light] — a directional diffuse term on the faceted cube, still with quads and no depth buffer.

[project-breakdown]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/doc/planning/project-breakdown.md
[version-0-0-7]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.7
[post-cube-paints-its-six-faces]: {{site.baseurl}}/{% post_url 2026-05-18-the-cube-paints-its-six-faces %}
[post-cube-sheds-hidden-edges]: {{site.baseurl}}/{% post_url 2026-05-17-the-cube-sheds-its-hidden-edges %}
[source-still-unit-cube]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.7/src/bin/still-unit-cube.rs#L1
[source-cube-palette]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.7/src/lib.rs#L36
[source-camera-direction]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.7/src/ortho_camera.rs#L75
[source-cube-face]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.7/src/scene/cube/face.rs#L9
[source-is-back-0-0-5]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.5/src/scene/cube/face.rs#L41
[source-visible-edges-0-0-5]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.5/src/scene/cube.rs#L49
[source-visible-faces-0-0-6]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.6/src/scene/cube.rs#L67
[source-is-front-facing]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.7/src/scene/cube/face.rs#L47
[source-visible-faces]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.7/src/scene/cube.rs#L72
[source-looking-from-front]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.7/src/scene/cube.rs#L190
[source-draw-faces]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.7/src/lib.rs#L65
[post-the-cube-gets-light]: {{site.baseurl}}/{% post_url 2026-05-22-the-cube-gets-light %}
