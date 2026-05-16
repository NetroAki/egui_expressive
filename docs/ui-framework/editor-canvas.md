# Editor / Canvas Guide

Stage 6 makes editor surfaces generic: the same primitives can drive designer canvases, timelines, piano-roll-like views, layout inspectors, or object editors without DAW-only coupling.

## Coordinate Model

- `LargeCanvas` owns viewport allocation, pan/zoom memory, and `ViewportCuller`.
- `EditorCanvas` wraps `LargeCanvas` with axes, snap-grid drawing, and logical-to-screen helpers.
- `PanZoom` remains the only pan/zoom model. Do not create a parallel transform stack for editor widgets.
- `SnapGrid` snaps logical positions and rectangle minima; resize helpers snap resized edges.

## Interaction Controller

`CanvasInteraction<K>` is the generic stateful bridge between pointer/keyboard input and pure editor mutations:

- `begin` chooses the topmost `CanvasItem` hit, applies `SelectionMode`, and starts move, resize, or marquee state.
- `drag` returns `CanvasInteractionEvent::Move`, `Resize`, or `Marquee` with logical rect mutations or selected ids.
- `keyboard_nudge` moves selected items by a logical delta through the same snap rules.
- `finish` clears active drag/marquee state.

Apps own their domain storage and apply `CanvasRectMutation<K>` to notes, clips, shapes, cards, layers, or any other item model.

## Selection, Resize, Snap, Alignment

- `SelectionModel<K>` stores selected ids and supports replace/add/toggle semantics.
- `CanvasItem<K>` owns hit testing, move, resize, min-size, locks, and resize-edge policy.
- `MarqueeSelection` returns intersecting ids for lasso/rubber-band selection.
- `align_rects` and `distribute_rects` provide pure alignment/distribution helpers for selected item rectangles.

## Drop Descriptors

`EditorDropRequest` and `EditorDropItem` describe file/object/text drops as plain data. They do not open native dialogs, read files, mutate the filesystem, load resources, or perform async work. Platform-certified drag/drop adapters remain Stage 8+ work.

## Inspector Hooks

`EditorInspectorTarget`, `EditorInspectorField`, and `EditorInspectorUpdate` are pure metadata/update descriptors for selected editor objects. Fields use `FormFieldValue` so Forms v2 can render or validate inspector values without reimplementing forms in the editor layer.

These descriptors are intentionally not a full inspector UI. Apps decide whether to render them with Forms v2, a property grid, a side panel, or custom controls.

## Undo / Shortcuts / Focus

Use Stage 4 primitives rather than creating editor-specific registries:

- Store editor snapshots in `UndoStack<T>` or use `UndoEntry::label` / `merge_key` for app-level history.
- Wire keyboard commands through `ScopedShortcutRegistry` and call `CanvasInteraction::keyboard_nudge` from the resolved action.
- Keep focus traversal in `FocusScope`; editor-specific focus metadata should be data, not a second focus system.

`EditorInteractionSnapshot<K>` bundles `EditorViewSnapshot` and selected ids for snapshot-based undo proofs.

## Piano-Roll Decision

`PianoRollView` is the Stage 6 name for the existing view-only piano-roll renderer. `PianoRoll` remains as a compatibility alias. Create/move/resize/select interaction proof lives in generic editor/canvas primitives and the `generic_editor_canvas` example, avoiding a DAW-specific sequencer implementation.

## DAW Namespace Decision

The historical `widgets::daw_editors` and optional `daw` feature remain for compatibility, but Stage 6 adds `widgets::editor_tools` as the DAW-neutral entry point and removes DAW-only package keywording. New general-purpose editor work should prefer `src/editor/*` and `widgets::editor_tools`.

## Example Proof

`examples/generic_editor_canvas.rs` demonstrates Stage 6 behavior without DAW modules:

- selection and marquee/lasso ids
- move, resize, snap, alignment, and distribution
- keyboard-style nudge helpers
- pure drop descriptors
- inspector hook descriptors
- `UndoStack` snapshot integration

The optional separate timeline-style example was deferred to keep Stage 6 bounded; timeline-style use cases are represented by the generic example's `Axis::time` canvas and remain available through `LaneStack`/`ValueLane` reuse rather than a new DAW/timeline product surface.

## Test Traceability

| Capability | Proof path |
| --- | --- |
| Coordinate model and viewport culling | `tests/performance_smoke.rs::release_smoke_viewport_culler_limits_visible_rows_and_columns` |
| Interaction controller move/resize/marquee/nudge | `src/editor/interaction.rs` unit tests and `tests/interaction_smoke.rs::editor_canvas_interaction_select_move_resize_marquee_smoke` |
| Selection, hit testing, and resize edges | `src/editor/selection.rs`, `src/editor/item_interaction.rs`, and the editor interaction smoke test |
| Snapping | `src/editor/snap.rs` unit tests plus editor interaction/performance smoke tests |
| Alignment and distribution | `src/editor/alignment.rs` unit tests plus `tests/interaction_smoke.rs::editor_alignment_drop_inspector_lane_smoke` and `tests/performance_smoke.rs::release_smoke_editor_alignment_many_rects` |
| Drop descriptors | `src/editor/drop.rs` unit tests plus `tests/interaction_smoke.rs::editor_alignment_drop_inspector_lane_smoke` |
| Inspector descriptors | `src/editor/inspector.rs` unit tests plus `tests/interaction_smoke.rs::editor_alignment_drop_inspector_lane_smoke` |
| Lane/value helpers | `src/editor/lane_stack.rs`, `src/editor/value_lane.rs`, and `tests/interaction_smoke.rs::editor_alignment_drop_inspector_lane_smoke` |
| Undo/persistence | `src/editor/persistence.rs` unit tests and `examples/generic_editor_canvas.rs` snapshot usage |
| Release-scale editor operation counts | `tests/performance_smoke.rs::release_smoke_editor_hit_test_with_many_items` and `tests/performance_smoke.rs::release_smoke_editor_alignment_many_rects` |
| Deterministic editor visual-state proof | `tests/visual_diff/fixtures/manifest.tsv` row `editor-canvas-selection-states` and `tests/visual_diff_harness.rs` |

## Deferred Scope

- Stage 7 owns visual-fidelity effects and animation polish.
- Stage 8 owns native file dialogs, platform-certified drag/drop, clipboard, accessibility/i18n, live-region, and system-theme integration.
- Stage 9 owns full visual regression, benchmarks, release docs, legacy cleanup, inactive scene cleanup, and global file-size hardening.
