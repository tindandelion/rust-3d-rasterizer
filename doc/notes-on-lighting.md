# Notes on lighting

Findings from comparing **thorus-forge** export renders to the local Three.js reference (`threejs/index.html`) and the [official geometry browser TorusGeometry scene](https://threejs.org/docs/scenes/geometry-browser.html#TorusGeometry). Use this when tuning Phase 2 materials, light intensity, and export-bin parity — **not** as a substitute for `doc/planning/project-spec.md` or `doc/planning/project-breakdown.md`.

## Current policy — export-bin light intensity

**Export bins (`still-scene`, `animated-scene`) use `DirectionalLight::new(...)` — intensity `1.0` — until the multi-light milestone.** Do not raise single-light intensity to match the geometry browser’s per-light **`intensity` 3** without parity testing; even with linear color, one light at **`3.0`** is not equivalent to three summed browser lights. Use **`with_intensity`** only in unit tests or deliberate experiments. The browser’s **`intensity` 3** on **three** summed lights is deferred to **Multi-light** in `doc/planning/project-breakdown.md`.

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

**`Material::shade`** composes in **linear** space (`src/lighting/color.rs`), then encodes to sRGB **`Rgb`**:

```text
linear: emissive + diffuse × diffuse_contrib + specular × specular_contrib  →  Rgb
```

**`Rgb`** (`src/framebuffer/colors.rs`):

- **`u8`** channels are **sRGB display bytes** (framebuffer storage, **`from_hex`**, ASCII preview).
- Material inputs are **sRGB**; **`Material::new`** decodes to linear **`Color`** at construction.
- **No** per-channel **`Rgb + Rgb`** or **`Rgb × f32`** — lighting math lives in **`Color`** only.

There is **no tone mapping** beyond sRGB encode on fragment output.

---

## How Three.js differs (why “intensity 3” ≠ same brightness)

Three.js **`MeshPhongMaterial`** (r173 in our reference):

1. Treats material **`color`**, **`emissive`**, and **`specular`** as **sRGB** inputs.
2. Converts them to **linear** color for the lighting equation.
3. Applies white **`DirectionalLight`** color × **`intensity`** × Lambert / Blinn–Phong terms in linear space.
4. Sums emissive once per fragment (also handled in linear space).
5. Converts the result back to **sRGB** for the framebuffer (renderer **`outputColorSpace`**).

So **`intensity: 3`** in Three.js means “3× in **linear** lighting space, then encode for display.” thorus-forge uses the same linear lighting model; remaining gaps vs Three.js are mostly **multi-light**, **tone mapping**, and reference-scene wiring — not byte-space clipping.

### Worked example (one light, full Lambert, intensity 3)

Material palette diffuse **`Rgb(21, 98, 137)`**, emissive **`Rgb(7, 37, 52)`**, ignore specular for simplicity.

| | **thorus-forge (linear path)** | **Three.js (approx.)** |
|--|-------------------------------|------------------------|
| Emissive | `(7, 37, 52)` after linear → sRGB round-trip (similar) | `(7, 37, 52)` after linear → sRGB round-trip (similar) |
| Diffuse × 3 @ `N·L = 1` | ~`(59, 163, 219)` in sRGB after linear scale + encode | ~`(59, 158, 219)` in sRGB |
| **Diffuse + emissive** | **~(66, 163, 219)** — no early byte clip | **~(66, 158, 219)** |

**Takeaway:** matching Three.js “feel” still requires aligned light count, intensity, and eyeball/golden parity — not just copying hex colors and setting **`intensity = 3`** on one light.

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

When comparing **`still-scene`** (one light @ **`intensity` 1.0**) to the local HTML file with only one active Three.js light @ **`intensity` 3**, intensities still do not match one-for-one — and **multi-light** summation differs. That mismatch in the reference file should be fixed when we use it as a golden target for **multi-light**; it does **not** explain Rust looking brighter at equal nominal intensity (fewer active Three.js lights would make Three.js **darker**).

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

Both systems add **emissive once per fragment**, independent of **`N·L`**, in **linear** space before sRGB encode.

---

## Implications for Phase 2

From `doc/planning/project-breakdown.md`:

1. **Material — explicit Phong colors (single light):** palette and **`DirectionalLight::intensity`** are in place; **export bins stay at `intensity` 1.0** (`DirectionalLight::new`). **`FrameBuffer::clear(Rgb(68, 68, 68))`** still open.
2. **Multi-light:** sum per-light diffuse/specular in **`Shape::render`**; wire three browser directionals at **`intensity` 3** in export bins (first time **`intensity` 3** ships in bins). Expect approximate — not bit-identical — parity with Three.js until multi-light golden WebPs land.
3. **Parity goal:** reproduce the geometry browser **look** in export artifacts — eyeball + golden WebPs — not bit-identical Three.js output unless reference wiring and light count match.

### Practical guidance

- **Export bins:** **`DirectionalLight::new`** only (**`intensity` 1.0**) until multi-light lands.
- **Do not** raise single-light export-bin intensity to **`3.0`** expecting Three.js parity on one light vs three browser lights.
- **Tests / experiments:** use **`with_intensity`**; document any non-default values in commit messages or this file.
- **Fix** `threejs/index.html` so all three lights use intensity **3** when that file is the reference for the multi-light milestone.
- **Consider** (future): optional tone mapping only if exports need HDR headroom — out of scope until parity testing demands it.

---

## Related code and docs

- **`src/lighting.rs`** — **`Material`**, **`DirectionalLight`**, unit tests for intensity scaling.
- **`src/lighting/color.rs`** — linear **`Color`**, sRGB ↔ linear conversion, **`Add`** / **`Mul`** in linear space.
- **`src/framebuffer/colors.rs`** — display **`Rgb`** (**`from_hex`**, **`brightness`**).
- **`src/bin/still-scene/main.rs`** — export bin wiring and material constants.
- **`threejs/index.html`** — local orthographic Three.js reference scene.
- **`doc/planning/project-spec.md`** — Phase 2 lighting target and known gaps vs browser reference.
- **`doc/planning/project-breakdown.md`** — open milestones (**Material**, **Multi-light**, …).
