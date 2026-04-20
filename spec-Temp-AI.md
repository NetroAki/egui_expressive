# spec-Temp-AI.md — Illustrator Parity & Per-Artboard Export

## Goal

Achieve maximum fidelity when exporting Adobe Illustrator `.ai` files to egui Rust code, including:

1. **Rotation extraction** from the `.ai` PDF content stream CTM (`cm` operator)
2. **Corner radius detection** from Bezier control-point geometry in path data
3. **Per-artboard file export** — each artboard generates its own `.rs` file
4. **Clipping mask support** in egui via `PaintCallback` + scissor approximation
5. **Blend mode compositing** in egui via `PaintCallback` + CPU-side blend math

---

## Feature 1: Rotation Extraction from `.ai` CTM

### Background

Adobe Illustrator `.ai` files are PDF wrappers. Object transformations are encoded as the `cm` (current transformation matrix) PDF operator in content streams:

```
a b c d e f cm
```

This sets the CTM to the 3×3 matrix:
```
[ a  b  0 ]
[ c  d  0 ]
[ e  f  1 ]
```

Rotation angle (degrees) = `atan2(b, a) * 180 / π`
Scale X = `sqrt(a² + b²)`
Scale Y = `sqrt(c² + d²)`
Translation = `(e, f)`

### Implementation

**File: `src/bin/ai_parser.rs`**

1. Add `rotation_deg: f64` and `scale_x: f64`, `scale_y: f64`, `translate_x: f64`, `translate_y: f64` fields to `Element`.
2. Add `parse_ctm_from_stream(content: &str) -> Option<(f64, f64, f64, f64, f64)>` function that:
   - Scans for the `cm` operator pattern: `<a> <b> <c> <d> <e> <f> cm`
   - Regex: `r"(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s+(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s+(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s+(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s+(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s+(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s+cm\b"`
   - Note: Case-insensitive exponent for PDF robustness.
   - Returns `(rotation_deg, scale_x, scale_y, translate_x, translate_y)`
3. Call `parse_ctm_from_stream` in `parse_aip_private_stream` and populate element fields.
4. Also scan the main PDF content stream (not just AIPrivateData) for `cm` operators associated with each object.

### Math

```rust
fn ctm_to_rotation(a: f64, b: f64, c: f64, d: f64) -> f64 {
    b.atan2(a).to_degrees()
}
fn ctm_to_scale(a: f64, b: f64, c: f64, d: f64) -> (f64, f64) {
    ((a*a + b*b).sqrt(), (c*c + d*d).sqrt())
}
```

---

## Feature 2: Corner Radius Detection from Bezier Geometry

### Background

Illustrator represents rounded rectangles as closed cubic Bezier paths with 8 anchor points (4 corners × 2 control points each). The corner radius `r` relates to the control handle distance `h` by:

```
h = r × 0.5522847498  (cubic Bezier approximation of quarter circle)
```

So: `r = h / 0.5522847498`

A rounded rect has this structure (clockwise from top-left):
- Anchor at `(x + r, y)` with handles going left and right
- Anchor at `(x + w - r, y)` with handles going left and right
- ... (4 corners, each with 2 control points)

### Implementation

**File: `src/bin/ai_parser.rs`**

1. Add `path_points: Vec<PathPoint>` and `corner_radius: f64` to `Element`.
2. Add `PathPoint` struct: `{ anchor: [f64; 2], left_ctrl: [f64; 2], right_ctrl: [f64; 2] }`.
3. Add `parse_path_geometry(content: &str) -> (Vec<PathPoint>, bool)` that:
   - Parses `m` (moveto), `l` (lineto), `c` (curveto), `C` (curveto abs), `z`/`Z` (closepath) operators
   - Preserves control points (not flattened)
   - Returns `(points, is_closed)`
4. Add `detect_corner_radius(points: &[PathPoint]) -> f64` that:
   - Checks if path has exactly 8 points (rounded rect signature)
   - Computes handle distance at each corner
   - Returns `handle_dist / 0.5522847498` if consistent (±5% tolerance), else `0.0`
5. Populate `element.corner_radius` and `element.path_points` in `parse_aip_private_stream`.

### PostScript Path Operators in .ai

```
x y m          → moveto
x y l          → lineto  
x1 y1 x2 y2 x3 y3 c  → curveto (control1, control2, endpoint)
f              → fill
S              → stroke
b              → fill + stroke
h              → closepath
```

---

## Feature 3: Per-Artboard File Export

### Background

Currently `ai_parser` outputs a single JSON with all elements. The codegen produces a single `.rs` file. We need each artboard to produce its own `.rs` file.

### Implementation

**File: `src/bin/ai_parser.rs`**

1. Add `artboard_name: Option<String>` to `Element` — populated from `%%Layer:` markers or artboard bounding box containment.
2. Enhance artboard parsing to extract artboard names from `%AI9_Artboard` markers.

**File: `src/codegen/mod.rs`**

1. Add `generate_artboard_file(artboard_name: &str, elements: &[LayoutElement], token_map: &HashMap<String, Color32>) -> String` function that:
   - Generates a complete `.rs` file with `pub fn draw_<artboard_name>(ui: &mut egui::Ui)` function
   - Includes standard imports header
   - Groups elements by artboard
2. Add `generate_all_artboards(layout: &[LayoutElement], artboards: &[(&str, egui::Rect)], token_map: &HashMap<String, Color32>) -> Vec<(String, String)>` that returns `(filename, content)` pairs.

**File: `src/bin/ai_parser.rs` main()**

Add `--per-artboard` flag: when set, output a JSON array of `{ artboard: name, filename: "artboard_name.rs", code: "..." }` objects instead of a single JSON.

---

## Feature 4: Clipping Mask Support

### Background

egui only supports rectangular scissor clipping. True arbitrary-shape clipping requires either:
- CPU-side masking (render to image, apply mask, upload texture)
- GPU stencil buffer (requires `PaintCallback` + custom wgpu pipeline)

### Implementation Strategy: CPU-side mask approximation

For the common case (clipping to a rounded rect or simple polygon), we implement CPU-side masking:

**File: `src/draw/mod.rs`**

Add `clipped_shape(ui: &mut egui::Ui, clip_path: &[egui::Pos2], clip_closed: bool, content: impl FnOnce(&mut egui::Ui))`:
1. Allocate a `ColorImage` matching the clip bounding box
2. Render content shapes into the image (CPU rasterization via `tiny-skia` or manual scanline)
3. Apply the clip mask (set alpha=0 outside the clip polygon)
4. Upload as texture and paint

**Alternative: Polygon approximation**

For convex clip shapes, approximate with the largest inscribed rectangle + corner fade:
- Use `Painter::with_clip_rect` for the bounding box
- Paint corner-covering shapes in the background color to simulate rounding

**Dependency addition** (`Cargo.toml`):
```toml
tiny-skia = { version = "0.11", optional = true }
```
Feature flag: `clip-mask`

### egui PaintCallback approach (GPU stencil)

For full accuracy, use `egui::PaintCallback` with a custom wgpu render pass:
1. First pass: render clip shape to stencil buffer
2. Second pass: render content with stencil test enabled

This requires `eframe` with wgpu backend and access to `RenderState`.

---

## Feature 5: Blend Mode Compositing

### Background

egui uses premultiplied alpha blending only. Photoshop/Illustrator blend modes (Multiply, Screen, Overlay, etc.) require per-pixel math that cannot be expressed as GPU blend state in wgpu (which only supports Add/Subtract/Min/Max operations).

### Implementation Strategy: CPU-side compositing

**File: `src/draw/mod.rs`**

Enhance `blend_color` (already exists) to be used in a layer compositing system:

Add `BlendLayer` struct:
```rust
pub struct BlendLayer {
    pub shapes: Vec<egui::Shape>,
    pub blend_mode: BlendMode,
    pub opacity: f32,
}
```

Add `composite_layers(ui: &mut egui::Ui, layers: Vec<BlendLayer>)`:
1. For each layer, tessellate shapes to a `ColorImage` (CPU rasterization)
2. Composite layers using `blend_color` per pixel
3. Upload final composite as texture

**File: `src/blur/mod.rs`**

Reuse existing `blur_image` infrastructure for the CPU rasterization pipeline.

### PaintCallback approach (GPU shader)

For real-time performance, implement blend modes as wgpu fragment shaders:

**File: `src/draw/blend_shader.wgsl`** (new file):
```wgsl
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base = textureSample(base_texture, base_sampler, in.uv);
    let blend = textureSample(blend_texture, blend_sampler, in.uv);
    // Multiply: base * blend
    let result = base * blend;
    return mix(base, result, blend.a);
}
```

This requires a custom `egui_wgpu::CallbackFn` that:
1. Renders the base layer to an offscreen texture
2. Renders the blend layer to a second texture
3. Composites with the blend shader

---

## Implementation Order

1. **Phase 1** (this sprint): Features 1, 2, 3 — pure Rust, no new dependencies
2. **Phase 2** (next sprint): Feature 4 with CPU approximation (polygon clip)
3. **Phase 3** (future): Features 4 (GPU stencil) and 5 (GPU shader) — requires wgpu integration

---

## Acceptance Criteria

### Feature 1 (Rotation)
- [ ] `Element.rotation_deg` populated from `cm` operator in `.ai` streams
- [ ] `LayoutElement.rotation_deg` populated from `Element.rotation_deg` in codegen pipeline
- [ ] Generated code uses `Transform2D::rotate_around(deg, rect.center())` correctly
- [ ] Test: `parse_ctm_from_stream("1 0 0 1 0 0 cm")` → `rotation_deg = 0.0`
- [ ] Test: `parse_ctm_from_stream("0 1 -1 0 0 0 cm")` → `rotation_deg = 90.0`

### Feature 2 (Corner Radius)
- [ ] `Element.corner_radius` populated from Bezier handle geometry
- [ ] `detect_corner_radius` returns correct radius for 8-point rounded rect paths
- [ ] `detect_corner_radius` returns `0.0` for non-rounded paths
- [ ] Test: 8-point path with handle distance 27.6 → radius ≈ 50.0

### Feature 3 (Per-Artboard Export)
- [ ] `--per-artboard` flag produces one JSON entry per artboard
- [ ] Each entry has `artboard`, `filename`, `code` fields
- [ ] `generate_artboard_file` produces valid compilable Rust
- [ ] Artboard name is sanitized to valid Rust identifier

### Feature 4 (Clipping Masks)
- [ ] `clipped_shape` function exists in `src/draw/mod.rs`
- [ ] Clips content to convex polygon using bounding rect + corner approximation
- [ ] No panic on empty clip path or degenerate shapes

### Feature 5 (Blend Modes)
- [ ] `BlendLayer` struct exists in `src/draw/mod.rs`
- [ ] `composite_layers` function exists (CPU path)
- [ ] All 16 blend modes from `BlendMode` enum are handled
- [ ] No panic on empty layer list

---

## Files Modified

- `src/bin/ai_parser.rs` — CTM parsing, path geometry, corner radius, per-artboard
- `src/codegen/mod.rs` — per-artboard generation, artboard-aware layout inference
- `src/draw/mod.rs` — clipping mask, blend layer compositing
- `Cargo.toml` — optional `tiny-skia` dependency (Phase 2)
- `spec-Temp-AI.md` — this file
