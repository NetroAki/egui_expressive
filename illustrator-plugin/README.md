# egui_expressive Illustrator Exporter

This is a **CEP extension** (not UXP) that exports Illustrator artboards to egui Rust code. It installs as a `.zxp` file.
This plugin uses CEP (Common Extensibility Platform). The `manifest.json` file is included for reference but the active runtime is CEP via the `.zxp` installer.
The packaged extension includes the Rust `ai-parser` binary under `bin/<platform>/`; the CEP panel probes that bundled binary and uses it to enrich DOM extraction with project-file transforms, paths, artboards, and appearance data when the host permits local process execution.

## Installation

1. Build the platform-specific `.zxp` file by running `cd installer && bash build_zxp.sh` (Linux) or `cd installer && build_zxp.bat` (Windows). The canonical package names are `egui_expressive_export-1.0.0-linux.zxp` and `egui_expressive_export-1.0.0-win32.zxp`.
   - **Note**: You can set the `ZXP_SIGN_PASSWORD` environment variable to specify a custom password for the self-signed certificate. If not set, an ephemeral password is used.
2. Install the generated `.zxp` file:
   - **Windows**: Double-click `install.bat` from the Windows installer bundle (requires `egui_expressive_export-1.0.0-win32.zxp` next to the script or in `..\dist\`)
   - **Linux CEP test host**: Install the `*-linux.zxp` with your CEP extension manager; helper scripts intentionally refuse cross-platform ZXP installs.
3. Restart Adobe Illustrator.
4. Go to **Window → Extensions → egui_expressive Export** to open the panel.

## Usage

1. Open your Illustrator document
2. Open the plugin panel: **Window → Extensions → egui_expressive Export**
3. Select the artboards you want to export
4. Configure options:
   - **Semantic color names** — Use extracted Illustrator color names when generating design tokens
   - **Include JSON sidecar** — Export element data for manual inspection
   - **Strict Code-Only** — Enabled by default; hard-fails unsupported opaque features instead of emitting runtime fallbacks
5. Click **Export Selected Artboards**
6. Copy the generated code or save to your project

Linked raster/images are traced into vector paths when the bundled parser/vectorizer can read the source pixels. Embedded rasters are first exported to a temporary tracing-only PNG when Illustrator allows extraction, then traced into vector paths before strict checks. Linked raster rotation uses source image dimensions plus Illustrator transform scale metadata (or exact orthogonal bbox inversion) to trace unrotated local bounds, then bakes rotation into vector coordinates; embedded raster extraction traces Illustrator’s transformed appearance to avoid double-rotation. Only `dropShadow`, `innerShadow`, `outerGlow`, `innerGlow`, `gaussianBlur`, and `feather` are preserved on the traced vector group; Motion Blur, Radial Blur, other non-Gaussian blur variants, unavailable extraction/tracing/transform metadata, or unmapped effects remain strict-unsupported. No generated export uses image slots or copied raster assets.

Gradient strokes (dashed or non-dashed) are exported as tessellated vector scene strokes. Pattern fills/strokes try to sample simple foreground/background colors from Illustrator pattern swatch artwork (`pattern.patternItem.pageItems`) and preserve those colors in generated scene patterns. Strict Code-Only still fails when swatch artwork is inaccessible rather than emitting procedural placeholder colors as exact output.

If project-file analysis is unavailable, **Strict Code-Only** export fails with an `ai-parser` diagnostic instead of silently falling back to DOM-only extraction. Disable Strict Code-Only only when you intentionally want an incomplete diagnostic export.

## Naming

The current scene-code exporter preserves Illustrator layer names in comments/metadata and uses extracted swatch/color names for token names when **Semantic color names** is enabled. Layout structure is emitted from Illustrator geometry and retained scene nodes; layer-name prefixes like `row-*`/`btn-*` are not interpreted as widget/layout commands in this code-only path.

## Output format

Each artboard generates a `draw_<name>(ui: &mut Ui, state: &mut <Name>State) -> Option<<Name>Action>` function. Add it to your project and call it from your egui update loop.
