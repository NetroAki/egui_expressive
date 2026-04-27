# spec-Temp-AI.md — Illustrator Parity & Per-Artboard Export

## Goal

Converting Illustrator projects into editable `egui_expressive` Rust code is the active implementation target. This working spec records the concrete parser, exporter, codegen, preview, and GPU-rendering behaviors used to satisfy that target. Current exports cover an incremental subset of features (vector/primitive export target). Limitations include embedded rasters, unsupported live effects, and complex blend/mask cases.

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
2. Add `parse_ctm` function that:
   - Scans for the `cm` operator pattern: `<a> <b> <c> <d> <e> <f> cm`
   - Regex: `r"(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s+(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s+(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s+(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s+(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s+(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s+cm\b"`
   - Note: Case-insensitive exponent for PDF robustness.
   - Returns `(rotation_deg, scale_x, scale_y, translate_x, translate_y)`
3. Call `parse_ctm` in `parse_aip_private_stream` and populate element fields.
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

1. Add `pub fn generate_artboard_file(artboard_name: &str, artboard_w: f32, artboard_h: f32, elements: &[LayoutElement], token_map: &HashMap<String, Color32>) -> String` function that:
   - Generates a complete `.rs` file with `pub fn draw_<artboard_name>(ui: &mut egui::Ui, state: &mut crate::generated::state::<ArtboardName>State)` function
   - Includes standard imports header
   - Groups elements by artboard
2. Add `pub fn generate_all_artboards<T: ArtboardDef>(all_elements: &[LayoutElement], artboards: &[T], token_map: &HashMap<String, Color32>) -> Vec<(String, String)>` that returns `(filename, content)` pairs.

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

Current public APIs:

- `clipped_shape_approx(ui, clip_polygon, is_convex, content)` clips to the polygon bounding box and paints background-colored cover geometry. It is only correct on uniform backgrounds.
- `clipped_shape_cpu(ui, clip_polygon, content)` uses tiny-skia to build an inverted mask overlay. It is a background-dependent approximation, not true arbitrary clipping on layered/non-flat scenes.

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

Implement and verify each parser, exporter, codegen, preview, and GPU-rendering behavior in this document as part of the vector/primitive export target.

---

## Acceptance Criteria

### Feature 1 (Rotation)
- [x] `Element.rotation_deg` populated from `cm` operator in `.ai` streams
- [x] `LayoutElement.rotation_deg` populated from `Element.rotation_deg` in codegen pipeline
- [x] Generated code uses `Transform2D::rotate_around(deg, rect.center())` correctly
- [x] Test: `parse_ctm("1 0 0 1 0 0 cm")` → `rotation_deg = 0.0`
- [x] Test: `parse_ctm("0 1 -1 0 0 0 cm")` → `rotation_deg = 90.0`

### Feature 2 (Corner Radius)
- [x] `Element.corner_radius` populated from Bezier handle geometry
- [x] `detect_corner_radius` returns correct radius for 8-point rounded rect paths
- [x] `detect_corner_radius` returns `0.0` for non-rounded paths
- [x] Test: 8-point path with handle distance 27.6 → radius ≈ 50.0

### Feature 3 (Per-Artboard Export)
- [x] `--per-artboard` flag produces one JSON entry per artboard
- [x] Each entry has `artboard`, `filename`, `code` fields
- [x] `generate_artboard_file` produces valid compilable Rust
- [x] Artboard name is sanitized to valid Rust identifier

### Feature 4 (Clipping Masks)
- [x] `clipped_shape_approx` function exists in `src/draw/mod.rs`
- [x] Clips content to convex polygon using bounding rect + corner approximation
- [x] No panic on empty clip path or degenerate shapes

### Feature 5 (Blend Modes)
- [x] `BlendLayer` struct exists in `src/draw/mod.rs`
- [x] `composite_layers` function exists (CPU path)
- [x] All 16 blend modes from `BlendMode` enum are handled
- [x] No panic on empty layer list
