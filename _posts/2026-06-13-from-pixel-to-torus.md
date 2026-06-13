---
layout: post
title: "Phase 1 Recap: From a Pixel to a Torus"
date: 2026-06-13 08:00:00 +0200
authors: Sergey and Cursor
---

As we're closing phase 1 of our project, we'd like to look back and see what we've achieved. Seventeen GitHub tags later, the CPU rasterizer can draw procedural meshes with Phong shading, resolve overlap with a depth buffer, and export still images and animations. 

[Version 0.0.17 on GitHub][version-0-0-17]{: .no-github-icon}

## What you will see today

The current demo showcases all capabilities of our rendering pipeline: a shiny blue donut tumbling under a fixed orthographic camera, with the near side of the tube correctly hiding the far side.

![Phong-shaded torus — the Phase 1 capstone render](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.17/doc/output/current.webp)

Compare that to where we started on [May 10][post-one-white-pixel]: an **800×600** black frame with a single white dot in the center. Not bad progress!

## How we got here

Phase 1, in the [project plan][project-spec], is the **CPU software rasterizer track**: build the math and rendering pipeline on the processor before touching the GPU. We did not try to ship everything at once. Each release added one idea, proved it in a visible artifact, and only then moved on.

Here is the progression at a glance:

```mermaid
flowchart LR
  A["WebP + lines"] --> B["Wireframe cube"]
  B --> C["Filled + lit cube"]
  C --> D["More shapes + camera"]
  D --> E["Smooth sphere"]
  E --> F["Depth + torus"]
```

The diary posts tell the story milestone by milestone; below is the same journey grouped by theme.

### Laying the output foundation

Before any 3D math, we needed a reliable way to *see* results. [One white pixel][post-one-white-pixel] proved the framebuffer → lossless WebP encode → disk path. [Lines without guesswork][post-lines-without-guesswork] added a line rasterizer on top of the same buffer.

That ordering was deliberate. A rasterizer without a visual export loop is painful to debug; we wanted browser-openable images from day one.

The capability to draw lines served us well for a while; later we replaced it with polygon drawing primitives. 

### Entering 3D: projection, motion, and culling

[A cube takes shape][post-a-cube-takes-shape] was the first geometry in world space, projected with an [_orthographic camera_][orthographic-projection] and drawn as twelve wireframe edges. [The cube starts spinning][post-cube-starts-spinning] introduced the animation demo scaffold — 360 frames, fixed timing, reusable for every later demo.

[The cube sheds its hidden edges][post-cube-sheds-hidden-edges] added [_back-face culling_][back-face-culling]: stop drawing hull edges that belong only to faces pointing away from the camera. The cube stopped looking like a see-through cage.

### Solid surfaces and light

[The cube paints its six faces][post-cube-paints-its-six-faces] swapped wireframe strokes for filled quads — flat color per face, no lighting yet. Along the way we hit a real bug: [the near face was classified as back][post-near-face-was-classified-as-back], a sign error in the facing test that only surfaced once filled quads made the mistake obvious.

[The cube gets light][post-cube-gets-light] brought [_directional light_][directional-light] and [_Lambertian diffuse_][lambertian] shading. Facets turned toward the light read brighter; facets in shadow fell darker. The cube finally looked like an object under the sun rather than a rainbow block.

### Richer geometry and a movable camera

[Meet the dodecahedron][post-dodecahedron] showed the pipeline was not cube-specific: we started to draw a different shape — a dodecahedron — using the same rendering pipeline. [We can move the camera now][post-we-can-move-the-camera] replaced the baked-in viewpoint with a proper look-at transform — eye position, scene target, world up. Controlling the camera position enabled us to enhance the animation demo: the orbit-style motion became possible, and we made use of it.

### Smooth shapes and better highlights

[Yet another shape: the sphere][post-yet-another-shape-the-sphere] added [_tessellation_][tessellation] via subdivision of an octahedron seed. [The sphere gets smooth][post-sphere-gets-smooth] introduced [_Gouraud shading_][gouraud]: interpolate lighting intensity across each triangle so the faceted mesh reads as a smooth surface.

Glossy materials followed in [first shot at glossy shapes][post-first-shot-at-glossy-shapes], but the specular highlight still looked faceted. [Phong shading: natural highlights][post-phong-shading-natural-highlights] moved the lighting calculation to each pixel instead of each vertex, which finally produced convincing specular spots on the sphere.

### Occlusion tests and the capstone torus

[Introducing the depth buffer][post-introducing-depth-buffer] added per-pixel depth testing. That unlocked drawing overlapped objects and, critically, [_self-occlusion_][self-occlusion] on a single mesh — exactly what a torus needs.

[The torus takes shape][post-torus-takes-shape] landed the shape we had named as the long-term visual target back in the [project breakdown][project-breakdown]: a parametric torus with smooth vertex normals, Phong shading, and the depth buffer handling tube-over-tube overlap. Phase 1's orthographic CPU track was complete.

## What the rasterizer can do now

Stepping back from individual milestones, Phase 1 left us with a coherent pipeline:

| Stage | What we built |
|-------|---------------|
| **Meshes** | Procedural cube, dodecahedron, sphere, and torus as indexed triangle lists with per-vertex normals |
| **Camera** | Orthographic look-at with arbitrary eye position |
| **Transform** | World → view → screen mapping with view-space depth |
| **Visibility** | Back-face culling before raster; depth test at write time |
| **Shading** | Blinn–Phong lighting with matte and shiny materials; per-pixel Phong raster |
| **Output** | Lossless WebP still images and animations at 800×600; terminal preview via Kitty graphics for still images |

The retired paths — wireframe-only exporters, quad fill, Gouraud raster — served their teaching purpose and were removed once Phong and the unified triangle path took over. The [current module layout][project-breakdown] on `main` reflects that consolidation.

## What we learned along the way

A few patterns showed up repeatedly across the seventeen releases:

- **Small visible steps beat big leaps.** Every milestone shipped a WebP you could open and eyeball. That kept debugging grounded when the math got harder.
- **Bugs hide until the next layer.** Back-face culling looked fine on wireframe; the facing sign error only hurt once we filled the cube. Normal transforms looked fine under uniform scale; squash broke them. Each layer stress-tested the one below.
- **Agent-assisted coding needs review.** Cursor helped implement torus parametrics, camera math, and much of the diary — but Sergey still owned conventions, caught sign errors, and decided when to refactor rather than patch forward.
- **Orthographic first was the right call.** Keeping parallel sight lines and linear depth interpolation let us learn shading and occlusion without also fighting perspective divide and $w$-aware interpolation. That complexity is explicitly deferred to the next open milestone.

## What comes next

Phase 1 is the CPU foundation, not the finish line. The [project breakdown][project-breakdown] lists the immediate follow-on:

1. **Perspective projection on the CPU** — homogeneous coordinates, $w$ divide, perspective-correct depth, per-fragment view direction for specular. Replay representative scenes (cube, torus) with the same animation policies, changing only the projection block.
2. **Phase 2: GPU via wgpu** — port the proven concepts to Metal on macOS: buffers, pipeline state, vertex/fragment shaders, depth test aligned with a deliberate NDC convention checkpoint.

We will keep the same rhythm: one idea per release, a visible artifact, a diary post when the story is worth telling. Phase 1 proved the pipeline works; Phase 2 asks whether we can teach the same ideas to hardware.

[post-torus-takes-shape]: {{site.baseurl}}/{% post_url 2026-06-11-the-torus-takes-shape %}
[post-one-white-pixel]: {{site.baseurl}}/{% post_url 2026-05-10-one-white-pixel %}
[post-lines-without-guesswork]: {{site.baseurl}}/{% post_url 2026-05-11-lines-without-guesswork %}
[post-a-cube-takes-shape]: {{site.baseurl}}/{% post_url 2026-05-15-a-cube-takes-shape %}
[post-cube-starts-spinning]: {{site.baseurl}}/{% post_url 2026-05-16-the-cube-starts-spinning %}
[post-cube-sheds-hidden-edges]: {{site.baseurl}}/{% post_url 2026-05-17-the-cube-sheds-its-hidden-edges %}
[post-cube-paints-its-six-faces]: {{site.baseurl}}/{% post_url 2026-05-18-the-cube-paints-its-six-faces %}
[post-near-face-was-classified-as-back]: {{site.baseurl}}/{% post_url 2026-05-19-the-near-face-was-classified-as-back %}
[post-cube-gets-light]: {{site.baseurl}}/{% post_url 2026-05-22-the-cube-gets-light %}
[post-dodecahedron]: {{site.baseurl}}/{% post_url 2026-05-23-meet-new-shape-dodecahedron %}
[post-we-can-move-the-camera]: {{site.baseurl}}/{% post_url 2026-05-26-we-can-move-the-camera-now %}
[post-yet-another-shape-the-sphere]: {{site.baseurl}}/{% post_url 2026-05-27-yet-another-shape-the-sphere %}
[post-sphere-gets-smooth]: {{site.baseurl}}/{% post_url 2026-05-29-the-sphere-gets-smooth %}
[post-first-shot-at-glossy-shapes]: {{site.baseurl}}/{% post_url 2026-06-03-first-shot-at-glossy-shapes %}
[post-bugfix-transforming-surface-normals]: {{site.baseurl}}/{% post_url 2026-06-04-bugfix-transforming-surface-normals %}
[post-phong-shading-natural-highlights]: {{site.baseurl}}/{% post_url 2026-06-06-phong-shading-natural-highlights %}
[post-introducing-depth-buffer]: {{site.baseurl}}/{% post_url 2026-06-10-introducing-depth-buffer %}
[version-0-0-17]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.17
[project-spec]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/doc/planning/project-spec.md
[project-breakdown]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/doc/planning/project-breakdown.md
[orthographic-projection]: https://en.wikipedia.org/wiki/Orthographic_projection
[back-face-culling]: https://en.wikipedia.org/wiki/Back-face_culling
[directional-light]: https://en.wikipedia.org/wiki/Shading#Light_sources
[lambertian]: https://en.wikipedia.org/wiki/Lambertian_reflectance
[tessellation]: https://en.wikipedia.org/wiki/Tessellation_(computer_graphics)
[gouraud]: https://en.wikipedia.org/wiki/Gouraud_shading
[self-occlusion]: https://en.wikipedia.org/wiki/Hidden-surface_determination
