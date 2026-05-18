---
layout: post
title: "The Cube Sheds Its Hidden Edges"
date: 2026-05-17 08:00:00 +0200
authors: Sergey and Cursor
---

[Version 0.0.4][post-cube-starts-spinning] gave us a wireframe cube that tumbles smoothly — and that was real progress. A wireframe is an honest way to learn: twelve edges, a line rasterizer, no fills yet. But every frame still drew **all twelve** hull segments, including the ones that belong to faces on the **far side** of the box. The cube looked like a cage — you could see edges that a solid object would hide.

Version 0.0.5 fixes that with the simplest trick that works on a convex cube: **[back-face culling][back-face-culling]** on the wireframe. We still raster only lines; we just stop drawing edges that both adjacent faces agree are hidden.

[Version 0.0.5 on GitHub][version-0-0-5]{: .no-github-icon}

## What you will see

Side by side with 0.0.4, the difference is obvious. The tumble and the cornflower blue stroke are the same; what changes is **which** edges survive. Back faces no longer leak through — it finally looks like a cube you’re looking at from the outside, not a glass box you can see straight through.

![Animated cornflower-blue wireframe cube with back faces culled](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.5/doc/output/current.webp)

Fewer lines per frame also shrinks the animated WebP on disk (**~300 KB** vs **~387 KB** at 0.0.4) — a side effect, not the goal.

## Wireframe is fine — hidden faces are not

We are not abandoning wireframe. Orthographic projection, the Euler tumble, **360** frames at **50 fps**, lossless WebP — all of that stays. The raster path is still [`draw_line`][source-draw-line] end to end.

What felt wrong was **semantic**: when you watch the cube spin, edges on the rear faces are still there. Your brain knows those faces are turned away; drawing them anyway makes the object feel **see-through in the wrong way** — not artistic x-ray wireframe, just incomplete occlusion. For a convex cube, that is the easiest visual lie to fix before we add perspective, triangle fills, or a depth buffer.

The usual name for that fix is **[back-face culling][back-face-culling]** — here is how we apply it on a line-only cube.

## What is back-face culling?

In real-time graphics, the idea is simple: **do not spend work on geometry that faces away from the camera.**

Each flat face of a mesh has an **outward normal** — a unit vector perpendicular to the surface, pointing out of the solid. Compare that normal to the **view direction** (from the surface toward the eye, or equivalently the direction you treat as “into the scene”). The dot product $\mathbf{n} \cdot \mathbf{v}$ tells you which way the face points relative to the viewer:

- **Front-facing:** $\mathbf{n} \cdot \mathbf{v} > 0$ — the face points somewhat toward the camera; draw it (or, for us, allow its edges).
- **Back-facing:** $\mathbf{n} \cdot \mathbf{v} < 0$ — the face points away; **cull** it.
- **Edge-on (grazing):** $\mathbf{n} \cdot \mathbf{v} = 0$ — the face is sideways to the view. We treat that as **not** back-facing so silhouette edges do not vanish awkwardly.

For **filled** rasterization you usually cull whole **triangles** before shading. We are not filling yet; we cull at the level of **faces vs edges**:

> On each hull edge shared by two faces, **draw the edge unless both faces are strictly back-facing.**

In practice that means keeping silhouette and front hull wiring while dropping edges that exist only on the back of a convex solid. It is the same facing test as triangle culling, applied to **which line segments we emit**.

A concrete check: after a π/4 tilt on **X** and **Y** with **0.5** uniform scale (camera down **+Z**), [`visible_edges`][source-visible-edges] emits **nine** segments instead of twelve. The three dropped edges are hull segments where **both** incident faces are back-facing. A silhouette edge between a front face and a back face still draws — only one side is culled.

Our fixed orthographic camera looks down **+Z**; [`Camera::direction`][source-camera-direction] returns that axis so [`draw_edges`][source-draw-edges] and the cube use one consistent $\mathbf{v}$.

## Implementation: why we introduced _CubeFace_

Until 0.0.4, the cube was enough to describe as **vertices and edges**: eight corners, twelve undirected segments, pose them with a matrix, hand them to the line rasterizer. That model was fine while we drew every edge every frame.

Back-face culling adds a requirement edges alone do not carry: **which way each side of the box faces**. You need a **face** — not as a filled polygon yet, but as a record that owns an outward **normal** and knows which corners bound that side. Normals do not live naturally on a bare edge list; they belong to facets.

We extracted [`CubeFace`][source-cube-face] into its own module so that logic stays in one place and the rest of the code reads in terms of “a face” instead of scattered dot products and adjacency tables. The goal was simpler reasoning, not a bigger public API.

#### What CubeFace is responsible for

Each `CubeFace` is one quad of the hull. It stores:

- an outward **unit normal** (updated when the cube is posed), and  
- four **vertex indices** into the parent cube’s corner array (which corners form this side).

Its methods mirror those responsibilities:

- [`transform`][source-face-transform] — rotate the normal with the same pose matrix as the cube; leave indices unchanged (**0 … 7** always refer to the same corners).
- [`is_back`][source-face-is-back] — run the facing test ($\mathbf{n} \cdot \mathbf{v} < 0$).
- [`edges`][source-face-edges] — list the four boundary segments of this quad (index pairs along the winding).

[`Cube`][source-cube-vertices] itself changed shape: it now stores **`vertices` and `faces`**, not **`vertices` and `edges`**.

[`Cube::default`][source-cube-default] builds a **unit cube** — edge length **1**, centered at the origin **(0, 0, 0)** — as eight corners and six `CubeFace` records (outward normals along $\pm X$, $\pm Y$, $\pm Z$). That default is the template; exporters customize the pose by calling [`Cube::transform`][source-cube-transform], which returns a new cube with moved corners and updated face normals (the animation’s Euler tumble and the **0.5** uniform scale go through here).

We still need **edges** to draw the wireframe. They are no longer stored on the cube — we **derive** them from the faces. `visible_edges` skips back-facing faces, walks the boundary of each remaining face, and emits [`Edge`][source-edge] pairs for the rasterizer. Each interior hull edge belongs to two faces, so without deduplication we would emit it twice; canonical $(\min,\max)$ index keys collapse those duplicates.

`wireframe::draw_edges` is unchanged in spirit: project through [`Camera::transform`][source-camera-transform], call `draw_line`. It now sources segments from `visible_edges`, passing `camera.direction()`, instead of the old always-twelve [`edges()`][post-cube-starts-spinning] path from 0.0.4.

## Our next move

With orthographic projection, animation, and back-face–aware edges in place, the wireframe line of milestones feels complete for now.

The next chapter is **solid geometry**: a **filled cube** whose **faces have color** — six sides, six deliberate RGB values, rasterized as flat facets rather than strokes. Same pose and camera; the framebuffer gets triangles instead of segment lists. That is not lighting yet (no diffuse term, no light direction), but it is the bridge from “edges only” to surfaces you can later shade.

After the colored solid reads correctly, **shading and lighting** are the natural follow-on — faces that change brightness with orientation and a simple light model, still on the CPU before we tackle spheres, depth buffers, or perspective. The wireframe era taught projection and facing; the fill era is where the cube starts to look like an object. That solid, faceted cube is **[live in 0.0.6][post-cube-paints-faces]**.

[version-0-0-5]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.5
[post-cube-starts-spinning]: {{site.baseurl}}/{% post_url 2026-05-16-the-cube-starts-spinning %}
[back-face-culling]: https://en.wikipedia.org/wiki/Back-face_culling
[source-draw-line]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.5/src/framebuffer.rs#L51
[source-cube-face]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.5/src/scene/cube/face.rs#L9
[source-cube-vertices]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.5/src/scene/cube.rs#L28
[source-face-transform]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.5/src/scene/cube/face.rs#L24
[source-camera-direction]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.5/src/ortho_camera.rs#L75
[source-draw-edges]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.5/src/wireframe.rs#L9
[source-visible-edges]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.5/src/scene/cube.rs#L49
[source-cube-transform]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.5/src/scene/cube.rs#L36
[source-cube-default]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.5/src/scene/cube.rs#L94
[source-edge]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.5/src/scene/cube.rs#L21
[source-face-is-back]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.5/src/scene/cube/face.rs#L41
[source-face-edges]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.5/src/scene/cube/face.rs#L35
[source-camera-transform]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.5/src/ortho_camera.rs#L79
[post-cube-paints-faces]: {{site.baseurl}}/{% post_url 2026-05-18-the-cube-paints-its-six-faces %}
