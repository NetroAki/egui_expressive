# Illustrator strict-code enhancements

## Implemented in this enhancement pass

- Added committed visual-fixture infrastructure under `tests/visual_diff/fixtures/`.
- Seeded a real Illustrator/PDF reference PNG at
  `tests/visual_diff/fixtures/illustrator/ui-assets-page1.png`.
- Added the matching validation-only vector source
  `tests/visual_diff/fixtures/egui/ui-assets-page1.svg`, rendered it to
  `tests/visual_diff/fixtures/egui/ui-assets-page1.png`, and made the
  `ui-assets-page1` manifest row required.
- Extended `tests/visual_diff_harness.rs` to read `manifest.tsv`, run any
  complete fixture pairs, and emit failure heatmaps into `test-results/visual-diff/`.
- Added `diff_image_paths_with_heatmap` for CI/debug artifacts.
- Kept PNG comparison strictly as a validation-only artifact. Generated UI code
  remains vector-only; raster Illustrator inputs must be traced into vector
  paths before export instead of emitted as image slots.
- Added the Rust-side raster-to-vector foundation (`src/vectorize.rs`) using
  vtracer/visioncortex so linked or extracted raster pixels can become
  `SceneNode` path geometry before code generation.
- Wired embedded `RasterItem` extraction through temporary tracing-only PNGs so
  extractable embedded rasters use the same vectorization path before strict
  parity checks.
- Added transform/effect-aware raster tracing for safe cases: raster rotation is
  calculated from source image dimensions and baked into traced vector child
  coordinates for linked rasters, transformed embedded extraction avoids
  double-rotation, and scene-supported effects remain on the traced vector group.
- Added non-dashed gradient/pattern stroke tessellation so preserved Illustrator
  stroke paints render as code-only vector scene output instead of strict-failing
  solely because the stroke is non-solid.
- Extended gradient/pattern stroke tessellation to dash-aware rendering so dashed
  gradient and dashed pattern strokes also render as code-only vector scene output
  instead of strict-failing.
- Promoted bounded text parity for justified paragraphs and small-caps transforms:
  `TextBlock` now supports justification variants and small-caps rendering without
  forcing strict exports to fail for those cases.
- **Nested vector clipping masks** support in the scene renderer and codegen.
  - Added `ClipFillRule` and `ClipMask` types to `src/draw/mod.rs` for compound
    even-odd / NonZero clip mask semantics with multiple contours (holes).
  - Updated `BlendLayer` with an optional `clip_mask: Option<ClipMask>` field,
    preserving legacy `clip_polygons` compatibility.
  - Added `clipped_layers_mask()` public function supporting compound clip masks
    in the GPU and CPU render paths.
  - Updated `rasterize_composited_layers` to apply both `clip_polygons` (AND
    semantics) and `clip_mask` (fill-rule-aware) per layer.
  - Added `geometry_to_contours()` in `src/scene.rs` to extract multi-contour
    clip geometry from `Path`, `CompoundPath`, `Group`, etc.
  - Updated `render_node` and `collect_node_layers` to construct `ClipMask`
    from path and compound-path clip geometries with correct fill-rule mapping.
- **Plugin codegen** for non-rect and compound clip group geometry:
  - `sceneNodeExpr` now emits `SceneNode::path(...).with_clip_children(true)` or
    `SceneNode::compound_path(...).with_clip_children(true)` when a clip-mask
    group has `pathPoints` or `subpaths`.
  - Rectangular clip groups fall back to `SceneNode::clip_group(...)`.
- **Strict parity promotion**:
  - Vector clipping/masking in strict mode changed from `unsupported` to
    `approximate` with wording referencing visual fixture coverage.
  - Compound clipping masks with even-odd fill rule and supported topology are
    now allowed (previously `unsupported`).
  - Mixed clip groups (text/images) remain `unsupported` (future Task 6).
- **Committed visual fixtures** for clip masking:
  - `vector-clip-nested`: nested rectangular clip mask with clipped content.
  - `compound-clip-hole`: even-odd compound clip mask (donut with hole).
  - Both fixtures have matching Illustrator reference and egui PNGs plus
    egui-side SVG vector sources; manifest rows are required.

## Completed gradient mesh promotion

- Added committed gradient mesh visual fixture (`gradient-mesh-quad`) under
  `tests/visual_diff/fixtures/` with both expected (illustrator/) and actual
  (egui/) PNGs and a vector source SVG.
- The fixture exercises four-corner mesh-style color variation and is required in
  the manifest, so CI fails if the PNG pair is missing or differs.
- Removed strict `unsupported` finding for `hasMeshPatches(el)` in the plugin
  parity system: gradient mesh with parsed patches is now always `approximate`
  in both strict and non-strict modes, with wording indicating code-rendered
  mesh patches are covered by visual fixtures.
- Gradient mesh without parsed patches remains `unsupported` (no geometry to
  emit).
- The `gradient-mesh-quad` fixture is a committed reference placeholder and can
  be replaced by an Illustrator-exported reference when available.

## Completed bounded OpenType fidelity (Task 4)

- Added `OpenTypeFeatures` struct to `src/typography/mod.rs` with booleans for
  common feature tags: `ligatures`, `contextual_ligatures`,
  `discretionary_ligatures`, `fractions`, `ordinals`, `swash`,
  `titling_alternates`, `stylistic_alternates`, `kerning`.
- Added `baseline_shift`, `horizontal_scale`, `vertical_scale` fields to
  `TypeSpec` with builder methods and `effective_size()`.
- Rendering/measurement honours these fields:
  - **baseline shift**: shifts rendered glyph baseline and measured rect.
  - **disabled standard ligatures**: forces char-by-char rendering so egui
    cannot form ligatures in the fast path.
  - **horizontal scale**: scales per-character advance/measurement (glyph
    geometry is not distorted; advance-level approximation).
  - **vertical scale**: applied to effective font size for approximate
    proportional scaling.
- Plugin `getOpenTypeFeatures()` / `getTextMetricsOverrides()` extract feature
  flags, `baselineShift`, `horizontalScale`, `verticalScale` from Illustrator
  `characterAttributes` on both whole-text and text-run level.
- `typeSpecExpr` emits `.open_type_features(...)`, `.baseline_shift(...)`,
  `.horizontal_scale(...)`, `.vertical_scale(...)` builder chains.
- Parity findings mark advanced OpenType as `approximate` with honest wording:
  feature flags preserved and bounded metrics applied; alternate glyph
  substitution remains approximate (needs full shaper).

## Completed bounded live/raster effects expansion (Task 5)

- Added code-only vector rendering for Illustrator noise/grain effects via
  deterministic `noise_rect(...)` scene effect layers.
- Added code-only vector bevel approximation via `bevel_rect(...)`, emitting
  highlight/shadow edge strips rather than raster filters or image slots.
- Promoted recognized bevel and noise/grain/mezzotint effects into the safe
  raster-vectorization path so traced rasters can preserve those effects as scene
  `EffectLayer`s in strict mode.
- Kept unknown/live opaque effects strict-unsupported; only recognized modeled
  effects are allowed.
- Parity sidecar marks code-rendered Illustrator effects as approximate with
  explicit shadow/glow/blur/feather/bevel/noise wording.

## Next high-impact enhancements

1. **Unsafe raster effect expansion**
   - Expand or model currently unrecognized Illustrator raster effects such as
     bevel and noise as
     parity-safe vector scene effects.
   - Keep generated code vector-only; never emit image assets or runtime raster
     slots.

2. **Real fixture expansion**
   - Add focused Illustrator fixtures for gradients, compound paths, vector
     clipping, blend modes, text, and parser-recovered opaque vectors.
   - Keep one visual concern per artboard/fixture for clear regression triage.

3. **Parser ordering confidence**
   - Add fixtures that verify parser `layer_name`, `z_order`, `depth`, and
     parent/child hierarchy match Illustrator visual stacking.

4. **Failure artifact ergonomics**
   - Upload `test-results/visual-diff/*.png` from CI on failure.
   - Keep any screenshot/PNG regeneration tooling outside the export path; use it
     only for fixture validation and intentional tolerance updates.

5. **Color/font environment control**
   - Pin fonts and color-management assumptions for visual fixtures to avoid
     CI-only flakes.

6. **Strict-support backlog**
   - Task index: `docs/plans/illustrator-full-parity-tasks.md` (covers B-1, B-3, B-4, B-5, B-6, B-12, B-13..B-20 with status + acceptance criteria).
   - Full font shaping/alternate glyph substitution via HarfBuzz/rustybuzz
     (current OpenType feature-flag preservation is bounded metrics + code-level
     fidelity; true glyph substitution needs a full shaper).
   - Code-rendered live effects beyond current safe subset.
   - Note: basic nested vector masks and compound even-odd clip masks are now
     implemented (see "Implemented in this enhancement pass" above). Mixed
     text/image clipping remains unsupported; see the task index for the
     scoped remainder and the broader parity gaps.
