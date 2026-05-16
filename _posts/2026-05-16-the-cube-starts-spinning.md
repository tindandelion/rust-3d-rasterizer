---
layout: post
title: "The Cube Starts Spinning"
date: 2026-05-16 08:00:00 +0200
authors: Sergey and Cursor
---

[Version 0.0.3][post-a-cube-takes-shape] froze the orthographic wireframe cube in one pose. Version 0.0.4 is the first time we drive **model orientation per frame** and ship a **lossless animated WebP** on the same **800×600** raster path.

[Version 0.0.4 on GitHub][version-0-0-4]{: .no-github-icon}

## What you will see

The still image from 0.0.3 — white edges, tilted cube, black background — is still there via the [`still-cube`][source-still-cube] binary. The headline artifact for this release is motion: a **cornflower blue** wireframe tumbling smoothly in orthographic projection.

![Current render output](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.4/doc/output/current.webp)

Same twelve edges and fixed camera as before; what changed is that orientation updates every frame before we rasterize and encode.

## Refactoring: a unit cube we can pose

In 0.0.3 the cube lived as raw vertex lists and edge pairs in `main`. This milestone extracts that into a [`Cube`][source-cube]: a **unit cube** (edge length **1**, centered at the origin) that we place however we need via [`set_transform`][source-set-transform]. Scale it, tilt it, spin it — the mesh definition stays fixed; only the matrix changes.

Wireframe drawing walks [`edges()`][source-edges], which yields the twelve segments **after** the current transform. That separation — geometry and pose in one type, rasterization elsewhere — is what makes both the still image and the animation straightforward: each export path picks a matrix, hands the cube to [`draw_edges`][source-draw-edges], and the line rasterizer does not care which pose we chose.

This looks like the birthplace of a generic _Mesh_ abstraction: something that can expose edges in world space whether the mesh is a cube, a sphere, or a torus later. We are deferring that extraction until we have a second shape worth generalizing over; a concrete `Cube` is enough while projection and animation are still in flux.

## From one frame to many

Until now [`WebpEncoder`][source-webp-encoder] accepted a single framebuffer and wrote one still image. However, [WebP][webp] is not only a still image codec: the same container can hold an **[animated image][animated-webp]** — a sequence of frames with per-frame timing, like a compact GIF. That is what we ship in 0.0.4: one **lossless** animated WebP file built with the [`webp-animation`][webp-animation] crate, on the same RGB encode path as our single-frame exports.

Animation needs a **time loop**: clear the bitmap, set a new transform on the cube, draw edges, append a frame, repeat. [`animated-cube`][source-animated-cube] runs that loop [`ANIMATED_CUBE_FRAME_COUNT`][source-frame-count] times.

#### Frame count

We picked **360** frames so one full orientation lap samples the Euler angle about **once per degree** — smooth enough to eyeball, still a round number to reason about. The count lives once in the library as [`ANIMATED_CUBE_FRAME_COUNT`][source-frame-count] so the binary, integration test, and any future caller cannot disagree. Frame index $i$ maps to

$$
t = \frac{i}{360} \cdot 2\pi
$$

when we build the model matrix (see below) — one full turn in radians.

#### Playback at 50 fps

Playback speed is not implicit in “360 frames”; the [animated WebP][animated-webp] format stores an explicit **timestamp in milliseconds** on each frame. [`WebpEncoder::with_frame_spacing`][source-frame-spacing] wraps [`webp_animation::Encoder`][webp-animation]: on each [`add_frame`][source-add-frame] we pass the current timestamp, then advance it by [`FRAME_SPACING_MS`][source-frame-spacing-ms] (**20 ms**). That is **1000 / 20 → 50 fps**.

Timestamps must be strictly increasing (0, 20, 40, …), which the encoder enforces. At [`finalize`][source-finalize] we pass the next timestamp so the last frame’s display duration matches the spacing between frames (frame **359** at **7180 ms**).

One lap of motion over **360** samples at **50 fps** is **7.2 s** of video — long enough to see the tumble, short enough to iterate quickly.

## How the cube moves

Early in the milestone we spun the cube only around world $+\mathrm{Y}$ — easy to read, but only one axis of motion. The [breakdown][project-breakdown] also mentioned a **three-axis Euler** motion; we went straight to that as the shipped motion.

#### What “three-axis Euler” means

The name comes from [Leonhard Euler][leonhard-euler], who studied how rigid bodies rotate in 3D. An **[Euler angle][euler-angles]** decomposition expresses orientation as **several successive rotations**, each about a coordinate axis, instead of one big matrix chosen from scratch. **Three-axis** means three such steps — typically one turn about $X$, one about $Y$, and one about $Z$, each with its own angle ($\alpha$, $\beta$, $\gamma$ in the usual notation).

#### Why rotation order matters

A product like $R_z(\gamma)\, R_y(\beta)\, R_x(\alpha)$ is not a menu of three independent spins you can list in any order. It means: **apply the rotations one after another**, and in our column-vector convention the rightmost factor hits the vertex first — so $R_x(\alpha)$, then $R_y(\beta)$, then $R_z(\gamma)$.

The core reason is that **3D rotations do not commute**: in general

$$
R_A(\theta)\, R_B(\phi) \neq R_B(\phi)\, R_A(\theta)
$$

when $A$ and $B$ are different axes (here $X$, $Y$, $Z$). Swapping the order changes the composite matrix, so the cube ends up in a different orientation even with the **same three angles**.

A small illustration helps. Take a unit vector along $+X$, and two **90°** turns — first about $Y$, then about $X$:

1. $R_y$ sends $+X$ to $-Z$.
2. $R_x$ then sends that direction to $+Y$.

You land on $+Y$. Swap the order — $R_x$ first, then $R_y$ — and the first rotation leaves $+X$ unchanged (a turn about $X$ does not move the $X$ axis). Only the second rotation matters, and it takes $+X$ to $-Z$. Same angles, different path through space, different final axis.

So $R_z(\gamma)\, R_y(\beta)\, R_x(\alpha)$ and $R_x(\alpha)\, R_y(\beta)\, R_z(\gamma)$ are different poses for almost every choice of $\alpha$, $\beta$, $\gamma$. (In **2D** all rotations share one axis, so order barely matters; in **3D** you are turning about different directions in space, and the intermediate axes tilt with each step.)

Graphics and robotics texts therefore pick one convention — axis order **and** whether angles are measured in a fixed world frame or in a frame that rotates with the object — and stick to it. Our animation applies rotations in **$X \rightarrow Y \rightarrow Z$** order (about the fixed world axes).

Our animation is a deliberate simplification: we use **one** time-varying angle for all three axes ($\alpha = \beta = \gamma = t$), so the cube “tumbles” through combined $X$, $Y$, and $Z$ motion in one smooth lap. That is less general than arbitrary Euler angles, but enough for this milestone — and it loops cleanly when $t$ wraps.

The animated pose is a **world-fixed** tumble. In Rust we build one matrix:

$$
R_z(t)\, R_y(t)\, R_x(t)
$$

That product is easy to misread: for a column vertex $\mathbf{v}$, multiplication is $R_z\, R_y\, R_x\, \mathbf{v}$, so the **rightmost** factor runs first — **$R_x$**, then **$R_y$**, then **$R_z$**. The code spells **$Z \rightarrow Y \rightarrow X$** left to right; the motion is **$X \rightarrow Y \rightarrow Z$**.

With $\alpha = \beta = \gamma = t$, where $t$ sweeps across the lap in

$$
t \in \left[0,\, 2\pi\right)
$$

(exclusive of $2\pi$ on the last sample). In Rust we write that constant as [`TAU`][rust-tau] — the same value as $\tau$ (tau), i.e. one full circle in radians. One angle for all three rotations keeps the loop **seamless** — each factor completes whole turns when $t$ returns to zero.

The per-frame model matrix is

$$
M(t) = R_z(t)\, R_y(t)\, R_x(t)\, \mathrm{scale}(0.5).
$$

Each frame builds a fresh matrix and passes it through [`set_transform`][source-set-transform] before [`edges()`][source-edges] runs. The still image and animated export paths use different matrices — tilted white wireframe for the golden still image, scale-only Euler tumble in **cornflower blue** for the animation — but the same cube type and the same draw path.

## Testing animation without golden pixels

We still compare the still image against [`snapshots/cube/scene.webp`][source-snapshot] at full **RGBA** resolution. For the animated path we added a lighter integration check: run **`animated-cube`**, decode the output with [`webp_animation::Decoder`][webp-animation], and assert the frame count matches [`ANIMATED_CUBE_FRAME_COUNT`][source-frame-count] ([`animated_cube_writes_frames`][source-animated-test]).

Pixel-exact regression on every frame of a **360**-frame animation is deferred — eyeballing the sample WebP and the frame-count guardrail are enough for now. When motion bugs get subtle, we can add golden frames or compare raw framebuffer bytes before encode.

Regenerate the still image snapshot after intentional visual changes:

```bash
cargo run --quiet -p thorus-forge --bin still-cube -- snapshots/cube/scene.webp
```

Write a fresh animation (optional output path):

```bash
cargo run --quiet -p thorus-forge --bin animated-cube -- doc/output/current.webp
```

## What this version unlocks

We now have the **animated orthographic wireframe** milestone from the [breakdown][project-breakdown]: time-varying model orientation, multi-frame lossless WebP, same line rasterizer and camera as 0.0.3.

Not in 0.0.4 yet: **perspective** projection, filled triangles, depth buffer, [back-face culling][back-face-culling], or lighting.

The [breakdown][project-breakdown] had **perspective wireframe** next. Sergey is changing that order: perspective is still on the list, but first he wants the cube to stop drawing **invisible** sides. **Back-face culling** on the wireframe (drop edges that belong only to faces pointing away from the camera) should make the tumbling shape read more solid before we touch homogeneous divide math. Perspective and filled raster with a depth buffer can wait until that looks right.

[version-0-0-4]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.4
[post-a-cube-takes-shape]: {{site.baseurl}}/{% post_url 2026-05-15-a-cube-takes-shape %}
[rust-tau]: https://doc.rust-lang.org/std/f32/consts/constant.TAU.html
[leonhard-euler]: https://en.wikipedia.org/wiki/Leonhard_Euler
[euler-angles]: https://en.wikipedia.org/wiki/Euler_angles
[back-face-culling]: https://en.wikipedia.org/wiki/back-face_culling
[webp]: https://developers.google.com/speed/webp
[animated-webp]: https://developers.google.com/speed/webp/docs/riff_container#animated_image_format
[webp-animation]: https://docs.rs/webp-animation/latest/webp_animation/
[project-breakdown]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/doc/planning/project-breakdown.md
[source-frame-count]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.4/src/lib.rs#L16
[source-webp-encoder]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.4/src/webp_encoder.rs#L6
[source-frame-spacing]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.4/src/webp_encoder.rs#L21
[source-add-frame]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.4/src/webp_encoder.rs#L50
[source-finalize]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.4/src/webp_encoder.rs#L57
[source-still-cube]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.4/src/bin/still-cube.rs#L1
[source-animated-cube]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.4/src/bin/animated-cube.rs#L1
[source-frame-spacing-ms]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.4/src/bin/animated-cube.rs#L21
[source-draw-edges]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.4/src/wireframe.rs#L7
[source-cube]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.4/src/scene/cube.rs#L11
[source-set-transform]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.4/src/scene/cube.rs#L50
[source-edges]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.4/src/scene/cube.rs#L55
[source-snapshot]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.4/snapshots/cube/scene.webp
[source-animated-test]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.4/tests/animated_cube_writes_frames.rs#L15
