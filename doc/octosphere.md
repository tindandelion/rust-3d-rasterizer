# Octasphere — procedural sphere from octahedron subdivision

This note captures the **octahedron + iterative midpoint subdivision → unit sphere** approach for building a triangular mesh sphere. It aligns with the open milestone **Sphere: triangular mesh — procedural tessellation** in `doc/planning/project-breakdown.md` and fits the unified **`TriMesh` / triangle** raster path used after the **Dodecahedron** milestone.

**Coordinate intuition** elsewhere in this project: Unity-style left-handed world, **+Y up**, **+Z forward** (`doc/planning/project-spec.md`). The octasphere geometry is **agnostic** until you attach model/world transforms; winding must match whichever **facet-facing / back-face** convention the CPU path uses (`draw_facets`, `FillTriangle`).

---

## Why this approach

- **Small seed:** Six vertices and eight triangles at level zero—easy to validate and regress.
- **Even triangles:** After a subdivision or two, faces stay **reasonably uniform** compared to latitude–longitude (no pole pinch).
- **Smooth shading hook:** On a sphere of radius \(R\), vertex positions on the sphere already define **outward normals**: \(\mathbf{n}_i \approx \mathbf{p}_i / \|\mathbf{p}_i\|\) (for the unit sphere, \(\mathbf{n}_i = \mathbf{p}_i\)).
- **LOD:** Each subdivision step multiplies triangle count by about **four**; level is a natural quality knob.

**Trade-off:** Correct implementation depends on **reusing midpoint vertices along shared edges** (see midpoint cache below). Compared to **subdivide cube → normalize**, the octasphere avoids six face grids and cube-seam bookkeeping; compared to **icosphere**, the base is simpler but the lowest subdivision levels look less “hexagonal/geodesic” until \(L\) increases.

---

## Level 0: regular octahedron on the unit sphere

Place **six** vertices at axis crossings. They lie on the **unit sphere** already (no normalization required at \(L = 0\)):

| Vertex role   | Positions                              |
|---------------|----------------------------------------|
| ±X on sphere  | \((\pm 1, 0, 0)\)                      |
| ±Y on sphere  | \((0, \pm 1, 0)\)                      |
| ±Z on sphere  | \((0, 0, \pm 1)\)                      |

A regular octahedron has **eight triangular faces**—one per octant—with each triangle using **one vertex from each axis**, choosing the **sign pattern** that matches that octant (e.g. \((+,+,+)\) uses vertices \((1,0,0)\), \((0,1,0)\), \((0,0,1)\) with consistent **winding from outside**).

**Counts:** \(V_0 = 6\), **12** undirected edges, \(F_0 = 8\) faces.

Represent the mesh as:

- **`positions[vertex_id]`** — `Vec3` on sphere (eventually scaled by model matrix if \(R \neq 1\)).
- **`indices`** — triplets `(i₀, i₁, i₂)` per triangle; **outside** winding must match raster / culling expectations (typically **counter-clockwise when viewed from outside** the closed solid—a **diagram** avoids sign mistakes).

Hardcode vertices and the eight triangles in a tiny table first; assert \(\|\mathbf{p}\| \approx 1\) and sanity-check **`visible_facets` / winding** against a cube or existing mesh.

---

## One subdivision step (topology + geometry)

For each triangle **(A, B, C)** (indices `iA`, `iB`, `iC`, positions \(\mathbf{A}, \mathbf{B}, \mathbf{C}\)):

1. **Edge midpoints on the sphere.**  
   The new vertex halfway along the **great circle** arc between \(\mathbf{A}\) and \(\mathbf{B}\) is obtained from the Euclidean bisector direction:
   \[
   \mathbf{M}_{AB} = \frac{\mathbf{A} + \mathbf{B}}{\|\mathbf{A} + \mathbf{B}\|}
   \]
   Likewise \(\mathbf{M}_{BC}\), \(\mathbf{M}_{CA}\).  
   (**Degeneracy:** \(\mathbf{A} + \mathbf{B} = \mathbf{0}\) only if \(\mathbf{B} = -\mathbf{A}\); that does not occur on octahedron edges at \(L=0\), but keep in mind for exotic inputs.)

2. **Split into four triangles.**  
   Using midpoint **indices** (after cache resolution, below), one standard pattern is:
   - **(A, M_AB, M_CA)**
   - **(B, M_BC, M_AB)**
   - **(C, M_CA, M_BC)**
   - **(M_AB, M_BC, M_CA)**  

   Preserve **outside** winding for **all four** triangles when ordering corners (verify once with a sketch: **A → B → C** CCW from outside; mids lie on edges **AB**, **BC**, **CA**).

3. **Output:** Append new vertex positions when creating new mids; emit **four** triplets replacing the original face. Repeat for every triangle of the **current** mesh to produce **next** mesh.

---

## Midpoint caching (mandatory)

Neighboring triangles share an edge. Without deduplication you create **distinct** midpoint vertices at the **same geometric location** ⇒ **holes**, **normal seams**, or **incorrect** shaded boundaries.

Maintain a map from **undirected edge** → **midpoint vertex index**:
- Key: `(min(iA, iB), max(iA, iB))` for edge **AB**.
- On first sight of that pair: compute \(\mathbf{M}_{AB}\), push `positions`, record index.
- On later sight: **reuse** stored index.

Build all mids for the current subdiv pass using this cache, then assemble the four triangles per face using **only** those indices.

---

## Growth and LOD

Let \(L\) be subdivision depth (\(L = 0\): octahedron).

- **Faces:** \(F_L = F_0 \cdot 4^L = 8 \cdot 4^L\).
- **Vertices:** grow with each level; after the first subdivision the canonical octasphere has **18** vertices (original **6** plus **12** edge mids). Higher levels: easiest to count after each pass or derive from mesh statistics.

Each **subdivision** is an independent “replace whole index buffer + extend position buffer” pass; keep previous level only if you need **multiple LODs** for debugging or future streaming.

---

## Normals: faceted vs smooth

- **Faceted (per-face):** Reuse the same **flat** normal model as the shaded cube / dodecahedron (one normal per triangle, duplicate at vertices if the pipeline requires it).
- **Smooth (later milestone):** For a **sphere**, use **interpolated vertex normals** matching **position direction** on the unit sphere: \(\mathbf{n}_i = \mathbf{p}_i\) after every projection step. Subdivision keeps vertices on the sphere, so this stays consistent without tangent-frame construction.

---

## Implementation checklist (order of work)

1. **Seed mesh:** Six positions, eight triangles, correct **winding** and **face count** vs `TriMesh` expectations.
2. **One subdivision** with **edge midpoint cache**; verify **shared edge** uses one midpoint index (spot-check on paper or with a small test).
3. **Loop** \(L\) for density; cap by triangle budget or export frame time.
4. **Optional:** scale radius via model matrix or `R * p̂` after normalization; world placement matches other procedural shapes.

---

## Reference comparison (same project context)

| Aspect            | Octasphere (this doc)     | Cube grid + normalize      |
|------------------|---------------------------|----------------------------|
| Seed size        | 6 verts, 8 faces          | 6 face grids               |
| Refinement       | ×4 faces per level        | Increase grid **N**        |
| Pitfall          | Midpoint **cache**        | **Welding** at cube seams  |
| Uniformity       | Good after a few levels   | Good; not as even as icosphere |

For an **icosphere** (icosahedron seed), the idea is the same **midpoint + project + four triangles + edge cache**; only the **level-0** vertex and face tables change.

---

## See also

- `doc/planning/project-breakdown.md` — **Sphere** milestones and ordering vs **depth buffer** / **torus**.
- `doc/planning/project-spec.md` — world / camera conventions.
