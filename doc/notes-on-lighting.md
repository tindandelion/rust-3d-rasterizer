# Notes on lighting

Findings from comparing **thorus-forge** export renders to the local Three.js reference (`threejs/index.html`) and the [official geometry browser TorusGeometry scene](https://threejs.org/docs/scenes/geometry-browser.html#TorusGeometry). Use this when tuning Phase 2 materials, light intensity, and export-bin parity — **not** as a substitute for `doc/planning/project-spec.md` or `doc/planning/project-breakdown.md`.

## Current policy — export-bin light intensity

**Export bins (`still-scene`, `animated-scene`) use `DirectionalLight::new(...)` — intensity `1.0` — until the multi-light milestone.** Do not raise single-light intensity to match the geometry browser’s per-light **`intensity` 3**; our byte-space Phong path clips much earlier than Three.js linear lighting (see below). Use **`with_intensity`** only in unit tests or deliberate experiments. The browser’s **`intensity` 3** on **three** summed lights is deferred to **Multi-light** in `doc/planning/project-breakdown.md`.

---

## What we compare

| | **thorus-forge (`still-scene`)** | **Local reference (`threejs/index.html`)** |
|--|----------------------------------|--------------------------------------------|
| Material | `Material::new(Rgb(7, 37, 52), Rgb(21, 98, 137), Rgb(17, 17, 17), Some(30))` | `MeshPhongMaterial`: color `0x156289`, emissive `0x072534`, specular `0x111111`, shininess `30` |
| Shading | Per-pixel Phong via `Material::shade` + `PhongShadedTriangle` | `MeshPhongMaterial` (smooth Phong) |
| Camera | Orthographic, eye `(0, 0.5, −1)` | Orthographic, eye `(0, 0.5, 30)` (same look-at target) |
| Torus | `meshes::torus(48, 32)` — default radii `0.7` / `0.3`, ring in **XZ** | `TorusGeometry(0.7, 0.3, 48, 32)` + `rotation.x = π/2` |
| Background | Black (`FrameBuffer` default clear) | `0x444444` |
| Lights | One `DirectionalLight` at **`intensity` 1.0** (`DirectionalLight::new`) | Three directionals (see **Reference light setup** below) |

Palette hex values match; **pixel values often do not**, even if export-bin intensity were raised (e.g. experiments with **`with_intensity(..., 3.0)`**).

---

## How our lighting works today

**`DirectionalLight`** (`src/lighting.rs`):

- Stores **`toward_light`** (unit vector from surface toward the light) and **`intensity`** (`f32`).
- **`new(toward_light)`** defaults intensity to **`1.0`**; **`with_intensity(toward_light, intensity)`** for explicit values.
- **`diffuse_contrib`** and **`specular_contrib`** multiply Lambert / Blinn–Phong terms by **`intensity`** inside the light — not in **`Material::shade`**.

**`Material::shade`** composes:

```text
emissive + diffuse × diffuse_contrib + specular × specular_contrib
```

**`Rgb`** (`src/framebuffer/colors.rs`):

- **`u8`** channels are used **directly** in lighting math.
- **`Rgb * f32`** scales each channel, rounds, then clamps to **`[0, 255]`**.
- **`Rgb + Rgb`** saturates per channel at **`255`**.

There is **no sRGB ↔ linear conversion** and **no tone mapping**. What you encode in **`Material`** is what gets multiplied and added in “display byte” space.

---

## How Three.js differs (why “intensity 3” ≠ same brightness)

Three.js **`MeshPhongMaterial`** (r173 in our reference):

1. Treats material **`color`**, **`emissive`**, and **`specular`** as **sRGB** inputs.
2. Converts them to **linear** color for the lighting equation.
3. Applies white **`DirectionalLight`** color × **`intensity`** × Lambert / Blinn–Phong terms in linear space.
4. Sums emissive once per fragment (also handled in linear space).
5. Converts the result back to **sRGB** for the framebuffer (renderer **`outputColorSpace`**).

So **`intensity: 3`** in Three.js means “3× in **linear** lighting space, then encode for display.” In thorus-forge it means “3× the **raw byte** values, then clamp.” Those are **not equivalent**, especially for mid-tone palette colors.

### Worked example (one light, full Lambert, intensity 3)

Material palette diffuse **`Rgb(21, 98, 137)`**, emissive **`Rgb(7, 37, 52)`**, ignore specular for simplicity.

| | **thorus-forge** | **Three.js (approx.)** |
|--|------------------|------------------------|
| Emissive | `(7, 37, 52)` | `(7, 37, 52)` after linear → sRGB round-trip (similar) |
| Diffuse × 3 @ `N·L = 1` | `(63, 294→255, 411→255)` | ~`(59, 293, 411)` in linear → sRGB ~`(59, 158, 219)` |
| **Diffuse + emissive** | **`(70, 255, 255)`** — G/B **saturated** | **`~(66, 158, 219)`** — no channel pegged at 255 |

At intensity **3**, our pipeline **clips green and blue early** on well-lit fragments; Three.js stays darker and less neon because lighting runs in linear space first.

**Takeaway:** matching Three.js “feel” requires more than copying hex colors and setting **`intensity = 3`**. A linear color path (or deliberate tuning scalars) is needed for numeric parity.

---

## Reference light setup (`threejs/index.html`)

The HUD claims three white directionals at intensity **3**, but the current script is:

```javascript
const lights = [
  new DirectionalLight(0xffffff, 0),
  new DirectionalLight(0xffffff, 0),
  new DirectionalLight(0xffffff, 3),
]
```

**Only the third light contributes.** The first two have intensity **0**.

The [official geometry browser](https://threejs.org/docs/scenes/geometry-browser.html#TorusGeometry) uses **three** active lights, all at intensity **3**:

```javascript
lights[0] = new DirectionalLight(0xffffff, 3)
lights[1] = new DirectionalLight(0xffffff, 3)
lights[2] = new DirectionalLight(0xffffff, 3)
```

Positions match our Phase 2 breakdown: **`(0, 200, 0)`**, **`(100, 200, 100)`**, **`(-100, -200, -100)`**.

When comparing **`still-scene`** (one light @ **`intensity` 1.0**) to the local HTML file with only one active Three.js light @ **`intensity` 3**, intensities still do not match — and Rust would look **brighter still** at **`intensity` 3** because of byte-space vs linear color math. That mismatch in the reference file should be fixed when we use it as a golden target for **multi-light**; it does **not** explain Rust looking brighter at equal nominal intensity (fewer active Three.js lights would make Three.js **darker**).

---

## Other comparison caveats

These affect highlight placement and side-by-side viewing; they are secondary to the color-space issue but worth keeping aligned.

| Topic | thorus-forge | Three.js reference |
|-------|--------------|-------------------|
| **Light direction** | e.g. `normalize(1, 0.5, −1)` in `still-scene` | From light **position** toward the scene; e.g. `(-100, −200, −100)` → ~`(0.44, 0.87, 0.22)` |
| **Torus orientation** | Default mesh: major ring in **XZ** | `torus.rotation.x = π/2` |
| **Background** | Black clear | `0x444444` — torus on black can **look** brighter in a split view even when torus pixels are unchanged |
| **Multi-light** | Single light until multi-light milestone | Official browser sums **three** contributions (each scaled by intensity in linear space) |

---

## Emissive

Both systems add **emissive once per fragment**, independent of **`N·L`**. In our pipeline emissive is full **`u8`** magnitude before lit terms are added; at high **`intensity`**, lit diffuse/specular saturate on top. Three.js adds emissive in linear space, which keeps the base glow comparatively subdued before display encoding.

---

## Implications for Phase 2

From `doc/planning/project-breakdown.md`:

1. **Material — explicit Phong colors (single light):** palette and **`DirectionalLight::intensity`** are in place; **export bins stay at `intensity` 1.0** (`DirectionalLight::new`). **`FrameBuffer::clear(Rgb(68, 68, 68))`** still open.
2. **Multi-light:** sum per-light diffuse/specular in **`Shape::render`**; wire three browser directionals at **`intensity` 3** in export bins (first time **`intensity` 3** ships in bins). Even then, do **not** expect pixel match without a linear color path.
3. **Parity goal:** reproduce the geometry browser **look** in export artifacts — eyeball + golden WebPs — not bit-identical Three.js output unless we add sRGB/linear handling.

### Practical guidance

- **Export bins:** **`DirectionalLight::new`** only (**`intensity` 1.0**) until multi-light lands.
- **Do not** raise single-light export-bin intensity to **`3.0`** expecting Three.js parity — Rust clips earlier in byte space (see worked example above).
- **Tests / experiments:** use **`with_intensity`**; document any non-default values in commit messages or this file.
- **Fix** `threejs/index.html` so all three lights use intensity **3** when that file is the reference for the multi-light milestone.
- **Consider** (future): sRGB ↔ linear on material inputs and fragment output; optional tone mapping only if exports need it — out of scope until parity testing demands it.

---

## Related code and docs

- **`src/lighting.rs`** — **`Material`**, **`DirectionalLight`**, unit tests for intensity scaling.
- **`src/framebuffer/colors.rs`** — **`Rgb`** arithmetic (saturating add, clamped multiply).
- **`src/bin/still-scene/main.rs`** — export bin wiring and material constants.
- **`threejs/index.html`** — local orthographic Three.js reference scene.
- **`doc/planning/project-spec.md`** — Phase 2 lighting target and known gaps vs browser reference.
- **`doc/planning/project-breakdown.md`** — open milestones (**Material**, **Multi-light**, …).
