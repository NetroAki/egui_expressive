# Visual parity fixtures

This directory is the committed fixture entry point for Illustrator-vs-egui
visual parity tests.

## Layout

- `illustrator/` — reference PNGs exported from Illustrator/PDF.
- `egui/` — matching PNGs rendered from generated `egui_expressive` code.
- `current-render/` — committed baselines generated from the in-crate draw pipeline for a small current-code proof subset.
- `manifest.tsv` — tab-separated fixture cases consumed by
  `tests/visual_diff_harness.rs`.

## Manifest columns

```text
case    expected    actual    max_channel_delta    max_mean_delta    max_bad_pixel_ratio    compare_alpha    required
```

Paths are relative to this directory. Optional rows (`required=false`) are
skipped until both PNGs exist. Required rows fail CI if either side is missing
or the visual diff exceeds the configured tolerance.

Required rows also need immediately preceding metadata comments naming the case:

```text
# fixture-intent: <case> ...
# fixture-source: <case> ...
# fixture-backend: <case> ...
```

Use `fixture-intent` for the behavior being proven, `fixture-source` for the
reference/artifact origin, and `fixture-backend` for the renderer path or
bounded contract. Broad tolerances additionally need a preceding
`tolerance-justification: <case>` comment.

Rows may also carry optional crop metadata:

```text
# crop-rect: <case> x y width height
```

Crop rows compare only that rectangle from the full expected/actual PNG pair and
write heatmaps using the cropped content. Exact crop rows are slice evidence for
the named region; they do not make a broad full-page parity claim.

When a case fails, the harness writes a red-channel heatmap to
`test-results/visual-diff/<case>-heatmap.png`.

## Current-code proof subset

`tests/visual_diff_harness.rs` remains the committed-pair regression corpus. It
does not regenerate every visual row from current renderer code.

`src/draw/current_render_visual_proof_tests.rs` adds a narrow deterministic
current-code proof subset that reconstructs these exact cases through the active
draw/raster/composite helpers and diffs the generated pixels against
`current-render/` baselines at zero tolerance. The R100-009B blend-boundary row
uses the helper path plus a bounded blue-mask green-channel quantization
correction that is asserted before application:

- `phase5-supported-gradient`
- `phase6-supported-gradient-angle`
- `phase6-supported-rounded-stroke`
- `vector-clip-nested`
- `compositing-blend-boundary`
- `phase7-supported-compound-hole-fill`
- `phase7-supported-polygon-clip-gradient`
- `phase7-supported-multiply-stack`
- `compound-clip-hole`

This narrows the replay-only gap for base gradients, angled gradients, rounded
rect/stroke drawing, a nested vector clip row that now passes green content
through `BlendLayer::clip_polygon`, the R100-009B `compositing-blend-boundary`
row, the `phase7-supported-compound-hole-fill` row, gradient+clip, multiply
compositing, and compound-hole masking. It does not upgrade
bounded/plumbing/full-page rows or the rest of the manifest to current-render
proof.

## Current real reference

`illustrator/ui-assets-page1.png` is seeded from the existing Illustrator/PDF
reference render. `egui/ui-assets-page1.svg` is the validation-only vector
source for the matching generated-output side, rendered to
`egui/ui-assets-page1.png`; the manifest row is required, so CI fails when this
fixture is missing or drifts beyond the recorded tolerance.

The first required fixture keeps a tight mean-delta gate while allowing sparse
high-channel edge outliers from renderer antialiasing and Illustrator/PDF color
management. Tighten the tolerance whenever the render pipeline becomes more
pixel-identical.

Phase 8 adds exact crop-slice rows that reuse this page (`ui-assets-page1-el3-fill`
and `ui-assets-page1-el4-fill`) for interior filled polygon regions that compare
at zero tolerance. The original full-page row remains bounded.

Phase 9A adds exact headless source-layer effect rows (`scene-supported-gaussian-blur`
and `scene-supported-feather`) for the optional WGPU two-pass blur callback over
initialized, library-owned, solid rectangular RGBA layers. They do not promote `tailwind-soft-shadow`,
`tailwind-backdrop-layered`, or host-framebuffer backdrop capture to exact parity.

Phase 9B adds exact headless source-layer effect rows (`scene-supported-drop-shadow`
and `scene-supported-outer-glow`) for initialized optional WGPU callbacks over
padded, library-owned, solid rectangular RGBA layers with requested blur/radius
at least `1.0`. They do not promote Tailwind `drop_shadow`, Tailwind
`backdrop_blur`, host-framebuffer backdrop capture, rounded/non-rect scene
effects, zero/sub-pixel shadow blur requests, or broad CSS shadow parity to exact.

R100-003A adds exact headless shaped scene source-layer rows:
`scene-supported-rounded-rect-blur`, `scene-supported-ellipse-drop-shadow`,
`scene-supported-path-feather`, and `scene-supported-rotated-rect-drop-shadow`.
They prove initialized WGPU callbacks over library-owned rounded-rect, ellipse,
closed-path, and rotated-rect-as-closed-path RGBA source rasterization. Shaped
shadow/glow rows use spread `0.0`. They do not promote group or mesh effects,
non-normal blend modes, zero/sub-pixel shadow blur, oversized tiling, inner
effects, Tailwind, codegen, browser, native, or host-framebuffer paths to exact.

R100-001A adds `backdrop-supported-app-snapshot-blur`, an exact headless artifact
row for an app-provided 96×64 RGBA backdrop snapshot blurred through the
initialized egui-wgpu callback path. The row includes a traceability source PNG
and exact expected/actual blurred PNG pair. It does not promote host/native
framebuffer capture, default Tailwind `backdrop_blur`, browser `backdrop-filter`,
or broad current-render proof.

R100-002 adds exact headless Tailwind effect evidence for two source-qualified
subsets: `tailwind-supported-drop-shadow-wgpu` covers a solid non-rounded
rectangular `Tw::drop_shadow` rendered through the initialized egui-wgpu
source-layer callback path, and `tailwind-supported-backdrop-snapshot-blur`
covers the explicit `Tw::backdrop_blur_app_provided` selector over an
app-provided 96×64 RGBA snapshot. The backdrop row includes a source PNG and
exact expected/actual blurred output pair. These rows do not promote default
`Tw::backdrop_blur`, rounded/CSS-complete shadow parity, codegen effect output,
browser `backdrop-filter`, native/host framebuffer capture, or app-owned WGPU
Tailwind token integration.

## Gradient mesh fixture

`gradient-mesh-quad` (`illustrator/gradient-mesh-quad.png` vs
`egui/gradient-mesh-quad.png`) exercises four-corner mesh-style color variation:
red, green, blue, and yellow at the four corners. The egui side has a
validation-only vector source at `egui/gradient-mesh-quad.svg`. This fixture is
required in the manifest and gates strict-code export of gradient mesh patches.

The current 2×2 PNG is a committed reference placeholder that validates the
fixture pipeline end-to-end. Replace it with a real Illustrator-exported
gradient mesh reference when Illustrator is available in the CI environment —
generate from a .ai file containing a single four-corner mesh patch, export as
PNG at the same dimensions, and update the egui-side render to match.

Visual fixture PNGs are validation artifacts only. Exported Illustrator raster
items must be converted into vector geometry before code generation; never add
runtime image slots or baked PNG dependencies to generated UI code.

## Stage 12 headless fidelity fixtures

The `headless/` rows added by Stage 12 are deterministic contract fixtures for
egui-native visual behavior, not browser/CSS pixel-parity claims:

- `tailwind-layout-bounds` — bounded sizing/flex/gap/position visual intent.
- `tailwind-soft-shadow` — deterministic soft-shadow output expectation.
- `tailwind-backdrop-layered` — backdrop blur remains overlay/tint behavior over layered content.
- `editor-canvas-selection-states` — grid, selected rect, handles, and marquee/focus visual state.
- `clip-layered-background` — clipping approximation behavior over non-flat backgrounds.
- `typography-token-panel` — token-driven type scale and text-state packaging.
- `m3-component-states` — representative Material 3 component visual states.
- `icon-button-token-states` — icon/button token states and hit-target packaging.
- `animation-transition-frames` — deterministic transition-keyframe visual proof.
- `form-data-widget-polish` — form/data widget state composition.
- `compositing-blend-boundary` — compositing boundary behavior for supported vector layers.
- `design-tool-stroke-boundary` — explicitly bounded gradient/pattern stroke parity status.
- `figma-token-placeholder-boundary` — Figma REST placeholder-token handoff status.

These rows use exact tolerance. If a future fixture requires broad tolerance, add
a `tolerance-justification: <case>` comment immediately above the row and explain
why the tolerance is still release-meaningful. Approximate fixtures prove bounded
behavior only and must not be counted as exact browser/design-tool parity.
