---
layout: post
title: "We Can Move the Camera Now!"
date: 2026-05-25 08:00:00 +0200
authors: Sergey and Cursor
---

Our 3D renderer becomes more potent: after the [exercise with the dodecahedron][post-dodecahedron], it acquired the ability to render an arbitrary shape, as long as we can represent it as a triangular mesh. It is quite restricted in other area, though: we can only render the scene from a single fixed point of view. Sergey wanted to add a bit more flexibility: what if we could position our camera at (almost) any point in the scene? This led us to an interesting quest in linear algebra: _transforming coordinate spaces_ between the world and the camera. 

[Version 0.0.10 on GitHub][version-0-0-10]{: .no-github-icon}

## What you will see

Having completed this exercise, we can now move the camera around the scene. Our animation now demonstrates this newly acquired ability. You'll still see the shaded dodecahedron, but notice that there is a change: 

1. The first half of the animation is the camera circling around the shape; 
2. The second half is our well known rotation of the shape, when viewed from the fixed PoV.

![Two-phase clip: camera orbit around a static dodecahedron, then model tumble with a fixed camera](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.10/doc/output/current.webp)

## World and view coordinates 

Until now, we had a fixed camera position that greatly simplified our life: we assumed that the camera was located at the point $(0, 0, -1)$, looking at the world's zero point. With this camera position, the math is very easy: camera's coordinate system basically aligns with the world's coordinates; the only difference is that $Z$ coordinate is shifted. But since in orthographic projection we discard Z coordinates anyways, that difference didn't affect anything. So with the camera in that position, we could safely assume that _world coordinates_ and _view coordinates_ are the same. 

Things start to get more interesting when we start moving the camera around. Let's imagine that we can place the camera in any arbitrary point in the scene, but with some restrictions on the camera orientation: 

* The camera still looks at the world's center; 
* The camera is oriented vertically: vertical lines in the scene stay vertical in the camera's view. 

![Different camera orientations]({{site.baseurl}}/assets/images/camera-orientations.svg)

In that case, the camera's coordinate system becomes both _translated_ and _rotated_ relative to the world's system. We can't use shape's vertex coordiantes as-is anymore. In order to build a correct projection from the camera's poit of view, we first need to transform all world (global) coordinates into the camera's (local) coordinate system, and then apply the projection transfromation that gives us the resulting screen coordinates in pixels. 

## Transforming coordinate systems

When it comes to transforming coordinate systems, linear algebra comes to help us a lot. Let's see how. 

First of all, the camera coordinate system can be specified as 3 _basis vectors_, one for each axis of the 3D space: 

$$
\mathbf{c_x} = \begin{pmatrix} c_{x1} \\ c_{x2} \\ c_{x3}\end{pmatrix}
\quad
\mathbf{c_y} = \begin{pmatrix} c_{y1} \\ c_{y2} \\ c_{y3}\end{pmatrix}
\quad
\mathbf{c_z} = \begin{pmatrix} c_{z1} \\ c_{z2} \\ c_{z3}\end{pmatrix}
$$

Notice that coordinates for these vectors are given in terms of *world coordinate system*; in other words, it's a description _how the camera's coordinate system is oriented in the world_.  

We can combine these vectors into 3x3 matrix $\mathbf{C} = \begin{pmatrix}\ \mathbf{c_x}; \mathbf{c_y}; \mathbf{c_z} \end{pmatrix}$. This becomes a _transformation matrix_ that relates points between camera and world spaces. In particular, given the point coordinates in the camera's system $\mathbf{p_c}$, we can find its coordinates in the world's space $\mathbf{p_w}$, and vice versa, using simple formulas:  

$$
\begin{gather}
\mathbf{p_w} = \mathbf{C}\,\mathbf{p_c} \\
\mathbf{p_c} = \mathbf{C}^{-1}\,\mathbf{p_w}
\end{gather}
$$

Let's apply now this knowledge to build the rotation matrix that will align camera and world spaces. 

## First step: rotation matrix 

Remember the conditions we've specified for the camera: they are sufficient to build the camera basis: 

1. The camera is placed at position $\mathbf{c}$ in the scene; 
2. The camera looks at the world's center $\mathbf{0}$;
3. The camera is oriented vertically. 

We'll start from finding the $\mathbf{c_z}$ vector. That's the easiest to find using camera position and the fact that it looks at the world's center: 

$$
\mathbf{c_z} = \frac{\mathbf{0} - \mathbf{c}}{\|\mathbf{0} - \mathbf{c}\|} = -\frac{\mathbf{c}}{\|\mathbf{c}\|}
$$

Once we have $\mathbf{c_z}$, we can use it to find $\mathbf{c_y}$. First of all, we know that it's going to be perpendicular to $\mathbf{c_z}$. Second, we know that it's going to lie in the plane spanned by $\mathbf{c_z}$ and world's Y axis (by condition 3 above). To find it, let's have a look at the picture: 

![Derivation of c_y]({{site.baseurl}}/assets/images/derivation-of-c_y.svg)

$$
\mathbf{c_y} = \frac{\mathbf{y} - \mathbf{u}}{\|\mathbf{y} - \mathbf{u}\|}
$$

We've expressed $\mathbf{c_y}$ in terms of world's Y axis and the auxillary vector $\mathbf{u}$. In its turn, $\mathbf{u}$ is a vector projection of Y onto $\mathbf{c_z}$: 

$$
\mathbf{u} = (\mathbf{y} \cdot \mathbf{c_z}) \mathbf{c_z}
$$

Finally, what's left is to find $\mathbf{c_x}$. Since we already have $\mathbf{c_z}$ and $\mathbf{c_y}$, it becomes a trivial cross product operation: 

$$
\mathbf{c_x} = \mathbf{c_y} \times \mathbf{c_z}
$$

Notice that the order of operands in the cross product matters: because we use left-hand coordinate system, we should use [left-hand rule][link-to-cross-product] to determine the direction of the X axis. 

## Combining rotation and translation 

TBD

## Implementation

The public surface is small and mirrors how exporters use it:

- [`Camera::for_viewport`][source-for-viewport] — default eye **`(0, 0, −1)`**, target at origin (replaces the old `Camera::new`).
- [`Camera::move_to`][source-move-to] — same viewport mapping, new eye; rebuilds view and stored forward direction.
- [`Camera::transform`][source-transform] — world **`Vec3` → `UVec2`** through the precomputed matrix.
- [`Camera::direction`][source-direction] — world-space into-scene unit vector for culling and lighting.

[`still-cube`][source-still-cube] calls **`move_to((0.1, 0.4, −1.0))`** so the filled cube still uses the π/4 tilt, but from a slightly raised, offset viewpoint. [`animated-scene`][source-animated-scene] drives **`camera_eye_orbit(angle)`** on the first **360** frames — **`(sin θ, 0.2, −cos θ)`** on a unit-radius **xz** circle — then pins **`(0, 0.2, −1)`** while the mesh tumbles. [`ANIMATED_SCENE_FRAME_COUNT`][source-frame-count] doubled to **720** to fit both halves.

## What comes next

The detour is landed; we can return to the [sphere milestone][project-breakdown-sphere] on the unified triangle path. Perspective projection later will reuse the same eye / target / world-up convention and only swap the projection half of the matrix stack. Depth buffer work still waits until self-overlap matters (torus and beyond).

[post-dodecahedron]: {{site.baseurl}}{% post_url 2026-05-23-meet-new-shape-dodecahedron %}
[project-breakdown-0-0-9]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/doc/planning/project-breakdown.md
[version-0-0-10]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.10
[project-breakdown-sphere]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.10/doc/planning/project-breakdown.md#--sphere-triangular-mesh--procedural-tessellation
[glam-crate]: https://docs.rs/glam
[source-ortho-camera]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.10/src/ortho_camera.rs
[source-world-to-camera]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.10/src/ortho_camera.rs#L124
[source-for-viewport]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.10/src/ortho_camera.rs#L72
[source-move-to]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.10/src/ortho_camera.rs#L78
[source-transform]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.10/src/ortho_camera.rs#L96
[source-direction]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.10/src/ortho_camera.rs#L89
[source-direction-0-0-9]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/src/ortho_camera.rs#L75
[source-transform-0-0-9]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.9/src/ortho_camera.rs#L80
[source-visible-facets]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.10/src/lib.rs#L52
[source-still-cube]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.10/src/bin/still-cube.rs
[source-animated-scene]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.10/src/bin/animated-scene.rs
[source-frame-count]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.10/src/lib.rs#L35
