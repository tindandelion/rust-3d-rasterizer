---
layout: post
title: "We Can Move the Camera Now!"
date: 2026-05-26 08:00:00 +0200
authors: Sergey and Cursor
---

After the [exercise with the dodecahedron][post-dodecahedron], the renderer can draw any shape we can express as a triangular mesh — but only from one baked-in viewpoint. Sergey wanted more flexibility: place the camera at (almost) any point in the scene, while it still points at the center. This challenge led us to a short quest in linear algebra: transforming coordinate spaces between _world_ and _view_ coordinate systems.

[Version 0.0.10 on GitHub][version-0-0-10]{: .no-github-icon}

## What you will see

This milestone divides our demo animation clip into two parts. You still see the shaded dodecahedron, but now the animation showcases different kinds of scene transformation:

1. The first half of the clip orbits the camera around the dodecahedron on a circle in the XZ plane;
2. The second half pins the camera and brings back the familiar Euler tumble of the model, viewed from a fixed eye.

![Two-phase clip: camera orbit around a static dodecahedron, then model tumble with a fixed camera](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.10/doc/output/current.webp)

## World and view coordinates

Until now we assumed the camera sat at $(0, 0, -1)$, looking at the world's origin. With that pose the math to convert between world and camera coordinates stayed easy: the camera's axes line up with the world's, and the only offset is along $Z$. Because [orthographic projection][orthographic-projection] drops $Z$ coordinates anyway, that $Z$ shift never affected positions — so we could treat world space and view space as the same.

Things get more interesting when the camera starts moving. To simplify the camera setup slightly, let's consider the following ability to place the camera:

* We can place the camera at any arbitrary position in the scene; 
* The camera always looks at the scene center (world origin).
* The camera's _up_ stays aligned with the $+Y$ world axis: vertical lines in the scene stay vertical on screen (in other words, there's no roll).

![Different camera orientations]({{site.baseurl}}/assets/images/camera-orientations.svg)

With those rules the camera frame is both _translated_ and _rotated_ relative to the world. It means that we can no longer use mesh vertex coordinates straight away. First, we need to transform all world coordinates from $XYZ$ into coordinates in the local camera coordinate system $X'Y'Z'$. That transformation expresses the scene from the camera's point of view. 

It leads to two distinct steps in the projection pipeline to render the shape: 

* First, we convert the shape's vertex coordinates from world to the camera's coordinate space; 
* Second, we apply the viewport transform that maps 3D coordinates into the 2D pixel space. 

## Transforming coordinate systems

To learn how to transform the coordinates between different spaces, we need a bit of linear algebra first. 

As we know, the 3D space can be specified by three [_basis vectors_][basis-vectors] that span the coordinate space. For the world coordinates, we assume the [standard basis][standard-basis], specified by unit vectors:

$$
\mathbf{e_x} = \begin{pmatrix} 1 \\ 0 \\ 0 \end{pmatrix}
\quad
\mathbf{e_y} = \begin{pmatrix} 0 \\ 1 \\ 0 \end{pmatrix}
\quad
\mathbf{e_z} = \begin{pmatrix} 0 \\ 0 \\ 1 \end{pmatrix}
$$

The camera frame is specified by its own basis vectors, each of which can be expressed in the *world coordinate system*:

$$
\mathbf{c_x} = \begin{pmatrix} c_{x1} \\ c_{x2} \\ c_{x3}\end{pmatrix}
\quad
\mathbf{c_y} = \begin{pmatrix} c_{y1} \\ c_{y2} \\ c_{y3}\end{pmatrix}
\quad
\mathbf{c_z} = \begin{pmatrix} c_{z1} \\ c_{z2} \\ c_{z3}\end{pmatrix}
$$

Those vectors give us a description of how the camera's axes sit inside the world.

We can stack them as columns of a $3 \times 3$ transformation matrix $\mathbf{C} = \begin{pmatrix}\ \mathbf{c_x} & \mathbf{c_y} & \mathbf{c_z}\end{pmatrix}$. That matrix relates camera-space $\mathbf{p_c}$ and world-space $\mathbf{p_w}$ points:

$$
\begin{gather}
\mathbf{p_w} = \mathbf{C}\,\mathbf{p_c} \\
\mathbf{p_c} = \mathbf{C}^{-1}\,\mathbf{p_w}
\end{gather}
$$

The second formula is the one we need: take a mesh vertex in the world, multiply by $\mathbf{C}^{-1}$, and you have coordinates in the camera space.

### Rotation matrix

The question then becomes: _how can we build that transformation matrix for the camera?_ Let's have a look at the constraints we set earlier: 

1. The camera sits at position $\mathbf{c}$ in the scene.
2. It looks at the world's center $\mathbf{0}$.
3. It stays upright relative to the world's $+Y$ axis.

They give us enough information to build $\mathbf{C}$, one basis vector at a time.

We start with the forward vector $\mathbf{c_z}$ ($Z'$ axis). It goes into the scene from the camera position and toward the target:

$$
\mathbf{c_z} = \frac{\mathbf{0} - \mathbf{c}}{\|\mathbf{0} - \mathbf{c}\|} = -\frac{\mathbf{c}}{\|\mathbf{c}\|}
$$

Having $\mathbf{c_z}$, we can now derive the up vector $\mathbf{c_y}$ ($Y'$ axis). We want the camera up as close to world $+Y$ as possible while staying perpendicular to $\mathbf{c_z}$. Let's take a look at this picture to see how we can derive $\mathbf{c_y}$ from the world's $\mathbf{y} = (0,1,0)$ and $\mathbf{c_z}$, using the auxiliary vector $\mathbf{u}$:

![Derivation of c_y]({{site.baseurl}}/assets/images/derivation-of-c_y.svg)

$$
\mathbf{u} = (\mathbf{y} \cdot \mathbf{c_z})\,\mathbf{c_z}
\qquad
\mathbf{c_y} = \frac{\mathbf{y} - \mathbf{u}}{\|\mathbf{y} - \mathbf{u}\|}
$$

For reference: this step is part of the [Gram–Schmidt process][gram-schmidt], which builds an orthonormal basis from a set of linearly independent vectors.

With $\mathbf{c_y}$ and $\mathbf{c_z}$ in hand, we can now calculate the right basis vector $\mathbf{c_x}$ to complete the frame. It becomes a [cross product][cross-product] (operand order matters in our left-handed scene):

$$
\mathbf{c_x} = \mathbf{c_y} \times \mathbf{c_z}
$$

Those three unit vectors are exactly the columns of $\mathbf{C}$.

### Combining rotation and translation

Rotation alone would leave the camera's origin sitting on top of the world's origin. We also need to account for the translation component of our view transform, which places the camera origin at $\mathbf{c}$. However, translation cannot be expressed as a matrix in ordinary 3D space: in order to combine it with rotation, we need to move to 4D [_homogeneous coordinates_][homogeneous-coordinates]. Luckily, we can reuse our 3D rotation matrix $\mathbf{C}$, adjusted for homogeneous coordinates.

In homogeneous coordinates we build a $4 \times 4$ matrix that first applies the rotation $\mathbf{C}$, then translates by $\mathbf{c}$: that composition maps a point from the camera space into the world space. Inverting it gives us the _view matrix_ that maps world points into the camera frame:

$$
\mathbf{V} = \bigl(\mathbf{T}(\mathbf{c})\,\mathbf{C}\bigr)^{-1}
$$

where $\mathbf{T}(\mathbf{c})$ is translation by the camera position. Intuitively that means "subtract the camera offset and rotate into camera axes". We could write it by hand in 3D space: 

$$\mathbf{p_c} = \mathbf{C}^{-1}(\mathbf{p_w} - \mathbf{c})$$

though in homogeneous coordinates it is more compact and aligns well with other kinds of transforms.  

It's also worth noting that the basis matrix $\mathbf{C}$ is [_orthonormal_][orthonormal-basis], which means that $\mathbf{C}^{-1} = \mathbf{C}^T$, and also $\mathbf{T}^{-1}(\mathbf{c}) = \mathbf{T}(\mathbf{-c})$. Using these properties, we can write the view matrix without an explicit inverse: 

$$
\mathbf{V} = \mathbf{C}^T\,\mathbf{T}(\mathbf{-c})
$$

### Some camera positions are forbidden 

With the math above, we can place the camera at _almost_ any arbitrary point in the scene. There are a few exceptions, though: 

* We can't place the camera at the world's origin. That would yield a zero `forward` vector in our current implementation; 
* We can't place the camera on the $\pm Y$ line through the origin: that would result in the zero-length `up` vector.

For now, Sergey decided not to bother coming up with fallbacks for those cases: they will `panic!` today. Just place your cameras correctly, folks! 

## Implementation details

All that calculation goes into the [`Camera`][source-ortho-camera] data type. The API of this type remains small; we only add the ability to move the camera around the scene:

- [`Camera::for_viewport`][source-for-viewport] — default eye $(0, 0, -1)$, target at the origin (replaces `Camera::new`).
- [`Camera::move_to`][source-move-to] — new eye, same target and world-up policy; rebuilds view and stored forward.
- [`Camera::transform`][source-transform] — world `Vec3` → pixel `UVec2` through the precomputed matrix.
- [`Camera::direction`][source-direction] — world-space into-scene unit vector for culling and lighting.

Our [`animated-scene`][source-animated-scene] binary uses `Camera::move_to` to make a full circle around the dodecahedron, showcasing this new capability. 

## What comes next

This exercise was a bit of a detour into the world of linear algebra and space transformations. 

Now we can return to the [sphere milestone][project-breakdown-sphere]. Perspective projection later will reuse the same eye / target / world-up convention and only swap the projection half of the matrix stack. Depth buffer work still waits until self-overlap matters (torus and beyond).

[post-dodecahedron]: {{site.baseurl}}{% post_url 2026-05-23-meet-new-shape-dodecahedron %}
[version-0-0-10]: https://github.com/tindandelion/rust-3d-rasterizer/tree/0.0.10
[project-breakdown-sphere]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.10/doc/planning/project-breakdown.md#--sphere-triangular-mesh--procedural-tessellation
[orthographic-projection]: {{site.baseurl}}{% post_url 2026-05-15-a-cube-takes-shape %}
[basis-vectors]: https://en.wikipedia.org/wiki/Basis_(linear_algebra)
[standard-basis]: https://en.wikipedia.org/wiki/Standard_basis
[homogeneous-coordinates]: https://en.wikipedia.org/wiki/Homogeneous_coordinates
[cross-product]: https://en.wikipedia.org/wiki/Cross_product
[gram-schmidt]: https://en.wikipedia.org/wiki/Gram%E2%80%93Schmidt_process
[orthonormal-basis]: https://en.wikipedia.org/wiki/Orthonormal_basis
[source-ortho-camera]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.10/src/ortho_camera.rs
[source-for-viewport]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.10/src/ortho_camera.rs#L72
[source-move-to]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.10/src/ortho_camera.rs#L78
[source-transform]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.10/src/ortho_camera.rs#L96
[source-direction]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.10/src/ortho_camera.rs#L89
[source-animated-scene]: https://github.com/tindandelion/rust-3d-rasterizer/blob/0.0.10/src/bin/animated-scene.rs
