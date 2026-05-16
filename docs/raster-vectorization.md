# Raster-to-vector export policy

Raster inputs are allowed only as **source material** for vector tracing. They are
not allowed in generated UI code as image assets, baked PNGs, texture slots, or
runtime rasterization dependencies.

## Pipeline

```text
raster pixels → vtracer/visioncortex → path/compound-path scene nodes → Rust code
```

The current Rust integration lives in `src/vectorize.rs`:

- `vectorize_rgba_to_scene_nodes(...)`
- `vectorize_image_file_to_scene_nodes(...)`
- `RasterVectorizeConfig`

The output is `scene::SceneNode` geometry using `Path`, `CompoundPath`, and
`PaintSource::Solid`, so normal codegen can emit pure vector Rust.

## Illustrator integration target

1. For linked `PlacedItem` rasters, pass the linked file path and Illustrator
   bounds into the bundled `ai-parser`/vectorizer helper.
2. For embedded `RasterItem`s, export a temporary tracing-only PNG through
   Illustrator when possible, then run the same vectorizer. The temp file is
   source material only and is not copied into generated UI output.
3. Replace the raster element with the generated vector scene/group before strict
   parity checks.
4. Keep strict export failing when pixels are unavailable or tracing fails.

Current Illustrator plugin wiring traces linked rasters and extractable embedded
rasters. Linked raster rotation is traced in unrotated local bounds computed from
the source image dimensions plus Illustrator transform scale metadata (or exact
orthogonal bbox inversion), then baked into traced vector child coordinates.
Non-orthogonal linked rotations without transform scale metadata remain
strict-unsupported instead of guessing. Embedded extraction traces Illustrator's
transformed temporary PNG and skips post-trace rotation to avoid double-rotation.
Scene-rendered `dropShadow`, `innerShadow`, `outerGlow`, `innerGlow`,
`gaussianBlur`, and `feather` effects are preserved on the traced vector group.
Motion Blur, Radial Blur, other non-Gaussian blur variants, bevel/noise, and
other unrecognized Illustrator effects still fail strict export until those
effects can be rendered or represented by a parity-safe vector wrapper.

## Non-goals

- No `paint_image_slot`.
- No `paint_image_from_path` in generated Illustrator exports.
- No copied raster assets in export bundles.
- No runtime vectorization; tracing is an export-time conversion step.
