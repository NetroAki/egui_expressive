# UI Framework Module Map

Use this map to find the right file without spelunking through the crate.

## Crate Root / Foundation

- `src/lib.rs` — public crate re-exports and top-level module wiring.
- `src/gpu.rs` — optional GPU bridge / acceleration-facing types.
- `src/draw/blend_shader.wgsl` — shader source consumed by `src/gpu.rs` via `include_str!`.
- `src/vectorize.rs` — raster/vector conversion helpers.
- `src/visual_diff.rs` — visual comparison and heatmap helpers.

## Codegen

- `src/codegen/mod.rs` — codegen docs, module wiring, and re-exports.
- `src/codegen/types.rs` — shared codegen data types and public sidecar model values.
- `src/codegen/dims.rs` — layout-node size estimation helpers.
- `src/codegen/generate.rs` — single-artboard Rust generation entrypoints.
- `src/codegen/inference.rs` — layout inference entrypoints.
- `src/codegen/multi_file.rs` — multi-artboard file generation.
- `src/codegen/naming.rs` — naming hints and label extraction.
- `src/codegen/node_emit.rs` — active per-node Rust emission dispatch.
- `src/codegen/effect_emit.rs` — R100-004A direct-shape effect emission helper; emits bounded or unsupported fidelity diagnostics for active `node_emit.rs` generated output, without exact WGPU callback claims.
- `src/codegen/node_emit_layout.rs` — row/column node emission helpers.
- `src/codegen/render.rs` — generated Rust rendering helpers and node emission body assembly.
- `src/codegen/render_shape.rs` — inactive legacy shape-specific emission evidence; R100-004A keeps it read-only and routes active direct effect emission through `node_emit.rs` plus `effect_emit.rs`.
- `src/codegen/render_utils.rs` — render/codegen utility functions.
- `src/codegen/scaffold.rs` — artboard scaffold/public export API.
- `src/codegen/scene_codegen.rs` — rich scene / shaped glyph emission.
- `src/codegen/sidecar.rs` — sidecar parser and diff input model.
- `src/codegen/sidecar_appearance.rs` — sidecar appearance parsing.
- `src/codegen/sidecar_appearance_fallback.rs` — fallback appearance parsing.
- `src/codegen/sidecar_diff.rs` — sidecar diff helpers.
- `src/codegen/sidecar_pattern.rs` — sidecar pattern parsing helpers.
- `src/codegen/sidecar_values.rs` — sidecar scalar/path/effects/text/appearance parsing helpers.
- `src/codegen/svg_helpers.rs` — SVG parser helper functions.
- `src/codegen/svg_parser.rs` — SVG parsing entrypoints.
- `src/codegen/svg_parser_helpers.rs` — SVG parser helper routines.
- `src/codegen/tests/` — split codegen tests by feature area.
- `src/codegen/tests/mod.rs` — codegen test module wiring.
- `src/codegen/tests/basic.rs` — basic codegen tests.
- `src/codegen/tests/codegen.rs` — inactive/unwired legacy test file; compiled codegen tests are wired by `src/codegen/tests/mod.rs` through `basic`, `multifile`, and `sidecar`.
- `src/codegen/tests/multifile.rs` — multi-file output tests.
- `src/codegen/tests/sidecar.rs` — sidecar parsing/codegen tests.

## CLI / Parser Tools

- `src/bin/ai_parser.rs` — Illustrator/AI parser CLI entrypoint and tests.
- `src/bin/ai_parser/` — focused AI parser implementation modules.
- `src/bin/ai_parser/types.rs` — parser data types.
- `src/bin/ai_parser/parsing.rs` — parser orchestration/helpers.
- `src/bin/ai_parser/pdf.rs` — PDF extraction/parsing helpers.
- `src/bin/ai_parser/parse_file.rs` — file parsing entrypoints.
- `src/bin/ai_parser/convert.rs` — conversion from parsed data to scene/codegen models.
- `src/bin/ai_parser/output.rs` — parser output serialization.
- `src/bin/ai_parser/entry.rs` — parser CLI dispatch.
- `src/bin/ai_parser/tests.rs` — parser tests.
- Stage 9 removed the orphan `src/bin/ai_parser_parts/*` split remnants after reference audit; active parser code is under `src/bin/ai_parser/` only.
- `src/bin/figma_export.rs` — Figma export CLI entrypoint.

## Import / Interop

- `src/figma/mod.rs` — Figma import docs and public re-exports.
- `src/figma/parse.rs` — Figma token/color parsing helpers.
- `src/figma/runtime.rs` — runtime data structures for Figma-derived UI.
- `src/figma/codegen.rs` — Figma-to-codegen bridge.
- `src/figma/tests.rs` — Figma parser/runtime tests.
- `src/svg/mod.rs` — SVG/ASE docs and public re-exports.
- `src/svg/path.rs` — SVG path parser and path geometry helpers.
- `src/svg/document.rs` — SVG document model/parser helpers.
- `src/svg/ase.rs` — Adobe Swatch Exchange parsing helpers.
- `src/compat/mod.rs` — cross-framework compatibility docs and re-exports.
- `src/compat/html.rs` — HTML-like alias/vocabulary support.
- `src/compat/kivy.rs` — Kivy-like alias/vocabulary support.
- `src/compat/qt.rs` — Qt-like alias/vocabulary support.
- `src/compat/swiftui.rs` — SwiftUI-like alias/vocabulary support.
- `src/compat/tkinter.rs` — Tkinter-like alias/vocabulary support.
- `src/swiftui/mod.rs` — SwiftUI-specific compatibility entrypoint.

## Styling

- `src/typography/mod.rs` — typography docs and public re-exports.
- `src/typography/core.rs` — text enums, shaper data, `TypeSpec`, and `TypeScale`; R100-005A owns bounded `TypeSpec::to_rich_text` weak / normal / strong weight emphasis while `to_font_id()` remains weight-agnostic.
- `src/typography/shaping.rs` — rustybuzz shaping helpers.
- `src/typography/render.rs` — shaped-glyph rendering helpers.
- `src/typography/text.rs` — text layout, text blocks, and `TypeLabel`.
- `src/tailwind/mod.rs` — module docs and public re-exports.
- `src/tailwind/builder.rs` — `Tw` data structure, `new`, stable ids.
- `src/tailwind/render.rs` — `Tw::to_frame` and `Tw::show` render contract, including R100-002 exact-first selection for eligible Tailwind drop shadows and explicit app-provided backdrop blur with bounded fallback.
- `src/tailwind/exact_effects.rs` — R100-002 WGPU-gated exact Tailwind source-layer helper for the solid non-rounded rectangular drop-shadow subset.
- `src/tailwind/responsive.rs` — `ResponsiveTw` style resolution and show helpers.
- `src/tailwind/theme_tokens.rs` — theme-token style helpers.
- `src/tailwind/box_model.rs` — margin, padding, frame aliases.
- `src/tailwind/color.rs` — raw colors and opacity.
- `src/tailwind/sizing.rs` — width/height/min/max/viewport sizes.
- `src/tailwind/display.rs` — block/flex/grid/hidden/overflow.
- `src/tailwind/flex_child.rs` — gap, wrapping, flex convenience sizing.
- `src/tailwind/grid_intent.rs` — grid columns/rows/spans.
- `src/tailwind/interaction.rs` — cursor and pointer-events utilities.
- `src/tailwind/effects.rs` — elevation/shadow entrypoint plus default bounded and explicit app-provided backdrop blur selectors.
- `src/tailwind/variants.rs` — state variant entrypoints.
- `src/tailwind/spacing.rs` — spacing constants and edge values.
- `src/tailwind/shadow.rs` — elevation/shadow conversion.
- `src/tailwind/types.rs` — small value enums such as size, font weight, and `TwBackdropSource`.
- `src/tailwind/typography.rs` — text size, weight, and tracking utilities; R100-005A propagates Tailwind numeric weight intent into `TypeSpec` without claiming weight-specific font-face selection.
- `src/tailwind/border.rs` — uniform/directional border and corner radius utilities.
- `src/tailwind/position.rs` — position, inset, translate, and z-index utilities.
- `src/tailwind/state.rs` — `hover:`/`focus:`/`disabled:`-style variant resolver and bounded transition interpolation.
- `src/style/mod.rs` — style docs and public re-exports.
- `src/style/visual.rs` — `VisualVariant`, `VisualState`, interpolation.
- `src/style/tokens.rs` — widget theme, surface/accent/spacing tokens.
- `src/style/text.rs` — text styles, styled text, scrollbar styling.
- `src/theme/mod.rs` — semantic colors, theme presets, elevation, borders.
- `src/icons/mod.rs` — icon families, glyph lookup, and icon text helpers.
- `docs/ui-framework/tw-render-contract.md` — canonical Tailwind/CSS-style utility support matrix and bounded-effect contract, including R100-005A typography weight-intent propagation and bounded RichText emphasis.
- `docs/ui-framework/tokens.md` — Stage 7 guide for tokens, typography, fonts, icons, M3 visuals, and visual recipes; keeps R100-005A numeric weight intent separate from future font registry/fallback/font-face work.

## Animation / Effects

- `src/animation/mod.rs` — animation docs and public re-exports.
- `src/animation/easing.rs` — easing functions and cubic-bezier helpers.
- `src/animation/tween.rs` — value tween helpers.
- `src/animation/spring.rs` — spring animation helpers.
- `src/animation/transition.rs` — transition state helpers.
- `src/animation/sequence.rs` — sequence/keyframe helpers.
- `src/animation/animated_state.rs` — animated state wrappers.
- `src/animation/memory.rs` — animation memory/state storage helpers.
- `src/animation/tests.rs` — animation tests.
- `src/blur/mod.rs` — blur/backdrop blur helper surface.
- `src/platform/backdrop.rs` — R100-001A app-provided backdrop snapshot provider contract, request/snapshot validation, and context-scoped provider install/load. R100-001B also owns the WGPU-gated app-owned offscreen backdrop source metadata and context-scoped install/load contract. These surfaces are single egui context/surface only and do not own OS/native capture.
- `src/platform/native_backdrop/mod.rs` — R100-001B feature-gated common native backdrop adapter substrate. It owns reserved native feature names, platform labels, and initialization errors only; platform-specific capture providers require later child blockers.
- `src/backdrop.rs` — R100-001A runtime helper layer that preflights app-provided snapshot backdrop blur reports and builds exact `GpuSourceLayerEffectCallback` shapes after provider capture and snapshot validation. R100-001B B3 also preflights app-owned WGPU source metadata, current installed source allocation, and renderer-bound sidecars before building app-owned callback shapes after exact source-qualified report success.
- `src/draw/app_owned_backdrop_blur_shader.wgsl` — R100-001B B3 first-pass shader that maps a validated app-owned source subrect into callback-local UV space before the existing vertical blur and present passes.

## Draw / Rendering

- `src/draw/mod.rs` — draw docs, active module declarations, and public re-exports.
- `src/draw/builders.rs` — general draw builder helpers.
- `src/draw/color.rs` — color conversion and color utility helpers.
- `src/draw/composite.rs` — layer compositing helpers.
- `src/draw/effects.rs` — visual effect helpers.
- `src/draw/geometry.rs` — geometry helper types/functions.
- `src/draw/images.rs` — image drawing helpers.
- `src/draw/layout.rs` — draw-time layout helpers.
- `src/draw/painter.rs` — painter extension helpers.
- `src/draw/painter_builders.rs` — layered painters and rect/circle/path shape builders.
- `src/draw/shadows_images.rs` — shadows, glow, bevels, runtime image loading, and fallbacks.
- `src/draw/gradients.rs` — linear and path gradient meshes.
- `src/draw/patterns.rs` — mesh/pattern/noise fill helpers.
- `src/draw/color_icons.rs` — blend modes, icons, radial gradients, overlays.
- `src/draw/strokes.rs` — rich strokes, dashed paths, transforms.
- `src/draw/transform_clip_layout.rs` — clip masks, clip scopes, opacity, transforms, z-stacks.
- `src/draw/composite_core.rs` — blend-layer compositing entrypoints.
- `src/draw/composite_masks.rs` — polygon/clip mask helpers.
- `src/draw/composite_hash.rs` — compositing hash/cache helpers.
- `src/draw/rasterize.rs` — compositing rasterization helpers.
- `src/draw/raster_pixels.rs` — raster pixel blend helpers.
- `src/draw/clipping.rs` — polygon clip approximations and fallback clipping.
- `src/draw/stack_tests.rs` — draw/stack regression tests.

## Material 3

- `src/m3/mod.rs` — module docs and public re-exports.
- `src/m3/color.rs` — Material 3 color roles and palettes.
- `src/m3/elevation.rs` — elevation tokens and tint helpers.
- `src/m3/theme.rs` — theme loading and semantic color access.
- `src/m3/typography.rs` — type scale and text styling; R100-005A exposes M3 Regular/Medium/Bold as numeric `TypeSpec` weight intent while `to_font_id()` remains size/family-only.
- `src/m3/components.rs` — component-family docs and public re-exports.
- `src/m3/components/button.rs` — button widget family.
- `src/m3/components/inputs.rs` — switch, checkbox, radio, chip, slider.
- `src/m3/components/feedback.rs` — progress, badge, divider, tooltip.
- `src/m3/components/surfaces.rs` — card surface shell.
- `src/m3/tier2.rs` — mid-tier docs and public re-exports.
- `src/m3/tier2/text_field.rs` — text field widget.
- `src/m3/tier2/navigation.rs` — nav bar, nav rail, nav item.
- `src/m3/tier2/app_bar.rs` — top app bar widget.
- `src/m3/tier2/list_item.rs` — list item widget.
- `src/m3/tier3.rs` — high-tier docs and public re-exports.
- `src/m3/tier3/dialog.rs` — dialog and snackbar widgets.
- `src/m3/tier3/floating.rs` — FAB and dropdown menu widgets.

## Accessibility / Interaction Semantics

- `src/accessibility/mod.rs` — docs and public re-exports.
- `src/accessibility/metadata.rs` — semantic roles, labels, descriptions, values, disabled state.
- `src/accessibility/live_region.rs` — live-region politeness, atomicity, relevant-change, and metadata descriptors.
- `src/accessibility/focus.rs` — focus ring, modal focus-trap helpers, and roving-focus group resolution.
- `src/accessibility/motion.rs` — reduced-motion flag and animation timing policy.
- `src/platform/mod.rs` — dependency-free platform descriptor docs and re-exports.
- `src/platform/clipboard.rs` — pure clipboard-write intent descriptors.
- `src/platform/file_drop.rs` — dropped-file summaries and editor-drop request adapters.
- `src/platform/system.rs` — system-theme preference and high-DPI display scale descriptors.
- `src/interaction/mod.rs` — docs and public re-exports.
- `src/interaction/drag.rs` — drag/pan helpers and value conversion.
- `src/interaction/actions.rs` — action/shortcut registry, accessibility meta, message bridge.
- `src/interaction/shortcuts.rs` — scoped shortcut hierarchy, conflict detection, and deterministic shortcut resolution.
- `src/interaction/shortcuts_tests.rs` — scoped shortcut conflict/priority/unknown-action tests split out to keep `shortcuts.rs` below the size target.
- `src/interaction/gestures.rs` — tap/long-press/swipe gestures.
- `src/interaction/focus.rs` — focus scope and tab navigation.
- `src/interaction/history.rs` — unbounded snapshot-based undo/redo stack and history entries.
- `src/interaction/feedback.rs` — runtime feedback queue/dispatcher policy for modal, snackbar, toast, progress, and notification-center events.
- `src/interaction/feedback_tests.rs` — feedback queue/live-region tests split from `feedback.rs` to keep the implementation file below the size target.
- `src/state/mod.rs` — persistence registry, audio bridge, and shared primitive state helpers.
- `docs/ui-framework/accessibility.md` — keyboard, metadata, live-region, screen-reader audit, and i18n/RTL guide.
- `docs/ui-framework/platform.md` — clipboard, file dialog/drop, system theme, high-DPI, dependency-review guide, and R100-001B app-owned WGPU backdrop support matrix/manual smoke checklist.

## Developer Tooling / Diagnostics

- `src/debug/mod.rs` — debugging helpers.
- `src/devtools/mod.rs` — devtools docs and public re-exports.
- `src/devtools/macros.rs` — devtools registration macros.
- `src/devtools/panel.rs` — devtools panel UI.
- `src/devtools/registry.rs` — devtools registry state.
- `src/daw/mod.rs` — DAW-named compatibility namespace pending Stage 6 rename/extract/generic-retention decision.

## Responsive UI

- `src/responsive/mod.rs` — docs and re-exports.
- `src/responsive/breakpoints.rs` — `BreakpointName`, `Breakpoints`.
- `src/responsive/value.rs` — `Responsive<T>`.
- `src/responsive/context.rs` — viewport/container helpers.

## Layout

- `src/layout/mod.rs` — docs and public re-exports only.
- `src/layout/helpers.rs` — small frame/divider/aspect-ratio helpers.
- `src/layout/stack.rs` — stack/spacer/divider macros.
- `src/layout/flex.rs` — `FlexContainer`, `FlexSize`, alignment, justification.
- `src/layout/grid.rs` — `GridLayout` and `GridSpan` values.
- `src/layout/position.rs` — `PositionMode`, `Insets`, `PositionStyle` values.

## Forms

- `src/forms/mod.rs` — form docs and public re-exports.
- `src/forms/field.rs` — labels, help/error text, validation shell.
- `src/forms/text.rs` — `TextField`, `TextArea` wrappers around egui `TextEdit`.
- `src/forms/select.rs` — select/dropdown wrappers around egui combo boxes.
- `src/forms/check.rs` — checkbox/radio/switch wrappers around egui native widgets.
- `src/forms/schema.rs` — schema field definitions, form values, dependency rules, focus/action metadata.
- `src/forms/validation.rs` — validation messages, sync rules, deferred validation descriptors, summaries.
- `src/forms/input.rs` — masks, numeric constraints, selection metadata, text-input platform contract.
- `src/forms/rich_inputs.rs` — autocomplete, multi-select, date/time, file request, color value descriptors.
- `src/forms/editing.rs` — inline edit targets, sessions, commits, cancels, and controller.

Forms wrap egui's input widgets rather than reimplement low-level input handling.

## Editor / Canvas

- `src/editor/mod.rs` — docs and public re-exports only.
- `src/editor/interaction.rs` — generic stateful canvas interaction controller and keyboard nudge helpers.
- `src/editor/alignment.rs` — pure alignment and distribution helpers for selected item rects.
- `src/editor/drop.rs` — pure file/object/text drop descriptors with no native side effects.
- `src/editor/inspector.rs` — generic inspector metadata/update descriptors backed by Forms v2 values.
- `src/editor/snap.rs` — coordinate snapping.
- `src/editor/axis.rs` — axis definitions and tick generation.
- `src/editor/selection.rs` — selected-ID model.
- `src/editor/item_interaction.rs` — canvas item hit-testing/move/resize.
- `src/editor/marquee.rs` — rubber-band selection.
- `src/editor/canvas.rs` — `EditorCanvas` adapter.
- `src/editor/lane_stack.rs` — generic lane-stack composition.
- `src/editor/value_lane.rs` — automation/value range mapping.
- `src/editor/persistence.rs` — view-state and interaction snapshots for undo integration.
- `src/surface/mod.rs` — `LargeCanvas`, `ViewportCuller`.
- `docs/ui-framework/editor-canvas.md` — Stage 6 editor/canvas recipes, decisions, and deferrals.

## Stage 2 Examples

Scope note: these are the Stage 2 app-shell proofs referenced by this plan, not a full crate example inventory.

- `examples/responsive_dashboard.rs` — canonical Stage 2 app-shell proof using persistent layout state, dock/split layout, sidebar collapse, breadcrumbs, tabs, and status bar composition.
- `examples/data_explorer_dashboard.rs` — Stage 3 proof combining Stage 2 app-shell chrome with data table, tree-table, property grid, and state demos.
- `examples/command_center_shell.rs` — Stage 4 proof combining action registry, menu/palette construction, scoped shortcuts, pure focus traversal, undo/redo, and feedback dispatch.
- `examples/forms_gallery.rs` — Stage 5 proof for Forms v2 schema, validation, input correctness, rich inputs, and inline edit descriptors.
- `examples/generic_editor_canvas.rs` — Stage 6 proof for DAW-free editor interactions, inspector hooks, drop descriptors, and undo snapshots.
- `examples/tailwind_style_gallery.rs` — Stage 7 proof for Tailwind-style utilities, responsive values, theme tokens, effects, and bounded transitions.
- `examples/accessibility_platform_gallery.rs` — Stage 8 proof for roving focus, live-region feedback, input/i18n contracts, and platform descriptors.
- `examples/neutraudio_shell.rs` — existing domain-flavored shell proof that must keep building across Stage 2 app-shell changes.

## Scene / Render Plan

- `src/render/mod.rs` — Phase 1A render fidelity vocabulary (`RenderQuality`, `RenderCapabilities`, `RenderReport`, `RenderIssue`) used to separate exact, approximate, and unsupported visual paths without changing egui's immediate-mode authoring model.
- `src/scene.rs` — active scene implementation and public re-exports, pinned by `src/lib.rs`.
- `src/scene/effects_geom.rs`, `src/scene/fill.rs`, `src/scene/model.rs`, `src/scene/render.rs`, `src/scene/stroke.rs` — active scene submodules wired by `src/scene.rs`. `effects_geom.rs` owns source-layer effect selection and, after R100-003A, routes approved rounded-rect, ellipse, closed-path, and rotated-rect-as-closed-path scene sources through rasterized RGBA plus the existing WGPU callback path while preserving fallback for excluded shapes/effects.
- `src/scene/tests.rs` — active scene tests wired by `src/scene.rs` under `cfg(test)`.
- Stage 9 removed inactive scene split leftovers (`mod.rs`, `appearance.rs`, `geom.rs`, `geom_extra.rs`, `stroke_extra.rs`, `tests_a.rs`, `tests_b.rs`, `tests_c.rs`) after verifying active scene wiring remains `src/scene.rs` + the six routed submodules above.
- `src/draw/composite_core.rs` plus `src/draw/rasterize.rs` — active compositor/raster group implementation. Report-returning APIs such as `composite_layers_report` and `clipped_layers_gpu_report` are the Phase 1A exactness seam; compatibility wrappers remain for older callers.
- `src/draw/transform_clip_layout.rs` — active clip/layout helpers plus Phase 2 `ClipMask` / `ClipFillRule` model for CPU offscreen polygon, rect, rounded-rect, compound even-odd/non-zero, and alpha-mask clipping.
- `src/draw/raster_pixels.rs` — active per-pixel mask application for polygon and `ClipMask` CPU compositing.
- `src/draw/composite_core.rs::clipped_layers_mask_report` — Phase 2 CPU offscreen compound/alpha mask entry point; compatibility wrapper `clipped_layers_mask` preserves immediate-mode call style.
- `src/gpu.rs` — Phase 3/3B optional `wgpu` backend resources and `GpuCompositeCallback` for egui-wgpu callback presentation of CPU-composited textures. Phase 5 also owns `GpuSourceLayerEffectCallback` and `wgpu_source_layer_effect_report` for bounded source-layer blur/backdrop reporting on library-owned RGBA pixels. Phase 9A hardens `GpuSourceLayerEffectCallback` into a two-pass separable blur path for exact context-marked initialized, solid-rect source-layer `GaussianBlur`/`Feather` scene evidence. Phase 9B extends the same callback/report path to exact context-marked initialized, padded solid-rect source-layer `DropShadow`/`OuterGlow` scene evidence only when requested blur/radius is at least `1.0`. R100-001A uses `GpuEffectSource::AppProvidedBackdropSnapshot` for source-qualified exact snapshot backdrop blur reports without making backend-global native capture claims. R100-001B B2 adds `GpuEffectSource::AppOwnedOffscreenBackdrop` as WGPU-first app-owned source vocabulary; B3 keeps direct generic reports non-exact but adds renderer-bound sidecar binding plus app-owned callback sampling for explicit helper paths. Native adapters still feed the snapshot path and do not promote `HostFramebufferBackdrop`. `init_gpu_effects_for_context` marks a specific egui context exact-ready so one renderer cannot globally enable scene callbacks for unrelated contexts. The caches are bounded and keyed by source/request/shader inputs; callback paths use callback-owned offscreen texture passes before main-pass presentation.
- `src/draw/blend_shader.wgsl` and `src/draw/blur_shader.wgsl` — Phase 3/3B blend/offscreen/presentation shader contract plus Phase 9A/9B source-layer blur/shadow shader contract. CPU-composited or source-layer pixels render into callback-owned offscreen targets with callback uniforms and no fixed-function alpha blending, then present with normal/1.0 uniforms to avoid double-applying blend state. This is not framebuffer/backdrop capture.
- Historical duplicate compositor/effects files that are not wired by `src/draw/mod.rs` are not active ownership surfaces. Wire or remove them in a later cleanup stage before using them for fidelity claims.

## Release Validation

- `tests/interaction_smoke.rs` — GUI-free release smoke coverage for data grid, undo, focus, feedback, Forms schema flows, and editor/canvas interaction descriptors.
- `tests/performance_smoke.rs` — deterministic large-input/performance-smoke proxies for data, focus, undo, viewport culling, and editor/canvas many-item operations without wall-clock thresholds.
- `tests/visual_diff_harness.rs` — manifest-driven visual regression/parity harness. Phase 7 fixture checks keep exact external vector/compositing assets under 300 KiB and exact headless Tailwind/typography/M3 rows dimension-pinned. Phase 9A pins exact source-layer blur/feather rows to 96×64 PNG pairs; Phase 9B adds exact source-layer shadow/glow rows with the same dimension/score-class governance. R100-002 adds exact Tailwind drop-shadow and app-provided backdrop rows, including source-vs-output guards for the app-provided snapshot row. R100-003A adds exact shaped scene source-layer rows for rounded-rect blur, ellipse drop shadow, path feather, and rotated-rect drop shadow.
- `src/draw/current_render_visual_proof_tests.rs` — unit-test-only current-code proof subset that regenerates nine exact draw/composite/clip cases through active draw/raster helpers and diffs them at zero tolerance against `tests/visual_diff/fixtures/current-render/` baselines. R100-009A adds `phase7-supported-compound-hole-fill` and hardens `vector-clip-nested` so green content passes through `BlendLayer::clip_polygon`; R100-009B adds `compositing-blend-boundary` with decoded-RGBA equality against the committed headless pair, using the helper path plus an asserted blue-mask green-channel quantization correction for `egui::Color32` alpha storage. This narrows current-render proof for named rows only; the manifest corpus remains committed-pair regression evidence unless another row gets its own generator.
- `tests/raster_vectorization.rs` — raster-to-scene-node vectorization regression proof included by the all-targets release gate.
- `docs/ui-framework/index.md` — release-facing docs index.
- `docs/release-checklist.md` — release gate checklist.
- `docs/versioning-policy.md` — SemVer and compatibility policy.
- `docs/migration-guide.md` — downstream migration guidance.
- `docs/exec-plans/file-size-exceptions.md` — Stage 9 >400-line remeasurement and justified exceptions.

## Widgets

- `src/widgets/mod.rs` — widget-family module declarations and public re-exports only.
- `src/widgets/app_shell/mod.rs` — app-shell docs and public re-exports.
- `src/widgets/app_shell/layout_state.rs` — serializable app-shell layout state and persistence slot registration.
- `src/widgets/app_shell/sidebar.rs` — generic sidebar/nav rail item model and widget.
- `src/widgets/app_shell/status_bar.rs` — status bar item model and widget.
- `src/widgets/app_shell/breadcrumbs.rs` — breadcrumb item model and widget.
- `src/widgets/channel_strip.rs` — channel strip composition widget.
- `src/widgets/daw_editors/mod.rs` — legacy creative-editor primitive path retained for compatibility.
- `src/widgets/editor_tools.rs` — DAW-neutral re-export path for existing creative-editor primitives.
- `src/widgets/daw_editors/piano_roll.rs` — view-only `PianoRollView` plus `PianoRoll` compatibility alias.
- `src/widgets/daw_editors/plugin_manager.rs` — searchable plugin/preset manager list.
- `src/widgets/daw_editors/system_monitor.rs` — CPU/RAM/disk/audio load metrics panel.
- `src/widgets/daw_editors/color_wheel.rs` — HSV-style color wheel/slider primitive.
- `src/widgets/daw_editors/controller_link.rs` — controller-link overlay state and UI.
- `src/widgets/daw_editors/generator_overlay.rs` — generator slot overlay primitive.
- `src/widgets/daw_editors/mixer_designer.rs` — mixer strip designer layout primitive.
- `src/widgets/designer.rs` — routing cable and designer-canvas primitives.
- `src/widgets/menus.rs` — menu definitions and top menu bar.
- `src/widgets/tabs.rs` — tab bar widget and `TabSetState` selected-tab fallback state.
- `src/widgets/toolbar.rs` — toolbar item data and toolbar strip.
- `src/widgets/transport.rs` — transport button kinds and button widget.
- `src/widgets/tree.rs` — tree node data and tree view widget.
- `src/widgets/controls/mod.rs` — small-control docs and public re-exports.
- `src/widgets/controls/dot_state.rs` — toggle-dot visual state.
- `src/widgets/controls/toggle_dot.rs` — toggle-dot widget.
- `src/widgets/controls/collapse_panel.rs` — collapsible panel state and shell.
- `src/widgets/controls/color_swatch.rs` — color swatch/button primitive.
- `src/widgets/controls/search_field.rs` — dense search/filter field.
- `src/widgets/controls/tool_button.rs` — action-id preserving tool button.
- `src/widgets/controls/control_group.rs` — pro-audio control-group/card shell.
- `src/widgets/data/mod.rs` — data-widget docs and public re-exports.
- `src/widgets/data/state.rs` — data-grid sort/filter/selection/view-state values.
- `src/widgets/data/model.rs` — data-grid row/column/cell descriptors and row provider model.
- `src/widgets/data/data_table.rs` — virtualized read-only data table surface.
- `src/widgets/data/editing.rs` — additive Forms v2 data-cell/property edit descriptors.
- `src/widgets/data/virtual_window.rs` — pure visible-range helper for large row sets.
- `src/widgets/data/tree_table.rs` — tree-table row model, flattening, and read-only display contract.
- `src/widgets/data/property_grid.rs` — read-only property-grid model with categories/groups.
- `src/widgets/displays/mod.rs` — display-widget docs and public re-exports.
- `src/widgets/displays/waveform.rs` — waveform display and compatibility alias.
- `src/widgets/displays/spectrum.rs` — spectrum bar display.
- `src/widgets/displays/spectrogram.rs` — spectrogram raster display.
- `src/widgets/displays/mini_bar_graph.rs` — compact bar graph display.
- `src/widgets/dock/mod.rs` — dock/split docs and public re-exports.
- `src/widgets/dock/split.rs` — resizable split-axis layout widget.
- `src/widgets/dock/panel.rs` — dock panel IDs, placement, and metadata value types.
- `src/widgets/dock/overlay.rs` — dock-zone hit regions and overlay paint.
- `src/widgets/drag/mod.rs` — drag-control docs and public re-exports.
- `src/widgets/drag/drag_reorder.rs` — drag-reorder list behavior.
- `src/widgets/drag/vertical_drag.rs` — vertical drag control.
- `src/widgets/drag/drag_number.rs` — numeric drag editor.
- `src/widgets/faders/mod.rs` — fader/slider docs and public re-exports.
- `src/widgets/faders/fader.rs` — fader widget.
- `src/widgets/faders/slider.rs` — slider widget.
- `src/widgets/faders/range_slider.rs` — range slider widget and normalization.
- `src/widgets/faders/xy_pad.rs` — XY pad widget.
- `src/widgets/faders/render.rs` — shared fader/slider paint helpers.
- `src/widgets/grid/mod.rs` — grid-family docs and public re-exports.
- `src/widgets/grid/step_grid.rs` — boolean step sequencer grid.
- `src/widgets/grid/cell.rs` — step-cell data and grid widget.
- `src/widgets/grid/canvas.rs` — beat/bar grid canvas helper.
- `src/widgets/grid/note_rect.rs` — note/ranged-event rectangle primitive.
- `src/widgets/knobs/mod.rs` — knob-family docs and public re-exports only.
- `src/widgets/knobs/style.rs` — shared knob/fader style enums (`Orientation`, `KnobStyle`, `KnobSize`, `ResetGesture`).
- `src/widgets/knobs/continuous.rs` — continuous-value drag/wheel/reset behavior.
- `src/widgets/knobs/knob.rs` — `Knob` widget state and egui widget integration.
- `src/widgets/knobs/render.rs` — private knob paint helpers.
- `src/widgets/meters/mod.rs` — meter-family docs and public re-exports.
- `src/widgets/meters/mode.rs` — meter mode enum.
- `src/widgets/meters/ballistics.rs` — meter ballistics configuration.
- `src/widgets/meters/meter.rs` — meter widget.
- `src/widgets/overlays/mod.rs` — overlay-family docs and public re-exports.
- `src/widgets/overlays/context_menu.rs` — context-menu builder.
- `src/widgets/overlays/floating_panel.rs` — floating panel/window wrapper.
- `src/widgets/overlays/modal_overlay.rs` — modal backdrop/window primitive.
- `src/widgets/overlays/toast.rs` — toast data, feedback-toast adapter, and visible toast layer.
- `src/widgets/overlays/progress_overlay.rs` — progress/result overlay.
- `src/widgets/overlays/command_palette.rs` — command palette data and filtering UI.
- `src/widgets/timeline/mod.rs` — timeline-family docs and public re-exports.
- `src/widgets/timeline/ruler.rs` — timeline ruler configuration.
- `src/widgets/timeline/clip.rs` — timeline clip metadata.
- `src/widgets/timeline/ruler_widget.rs` — interactive timeline ruler widget.
- `src/widgets/timeline/clip_widget.rs` — timeline clip widget and clip kind.
- `src/widgets/timeline/automation.rs` — automation points, segments, and curve interpolation.
- `src/widgets/timeline/loop_region.rs` — loop-region geometry.
- `src/widgets/timeline/fade.rs` — clip fade handles.
