# Migration Guide

This guide captures release-facing migration notes for downstream apps.

## Pre-1.0 Staged Builds

The current staged build is pre-1.0. APIs may still move, but Stage 9 release readiness keeps compatibility aliases where removing them would create avoidable churn.

## Preferred Module Paths

- Check `docs/ui-framework/api-stability.md` before adopting a crate-root re-export or broad module path in downstream app code.
- Prefer `widgets::editor_tools` for generic creative-editor primitives. Legacy DAW-named paths remain compatibility aliases.
- Prefer `src/platform/*` descriptors for clipboard/drop/theme/DPI integration boundaries instead of adding native side effects to core widgets.
- Prefer Forms v2 descriptors and data edit adapters for inline editing over adding edit behavior directly to read-only data widgets.
- Prefer documented entrypoints such as `generate_*`, `infer_layout`, sidecar diff APIs, SVG scaffold helpers, and Figma token parsing when using import/codegen modules; lower-level parser/codegen internals may move before 1.0.

## Compatibility and Experimental Surfaces

- `daw`, `widgets::daw_editors`, and the `PianoRoll` alias are retained for compatibility. New code should use DAW-neutral paths and `PianoRollView` where available.
- `debug`, `devtools`, GPU/`wgpu`, and feature-gated clipping helpers are experimental or optional unless the API stability map says otherwise.
- If a future release removes a compatibility path, the removal must be paired with a major-version plan or explicit migration note.

## Data Widgets

`DataTable`, `TreeTable`, and `PropertyGrid` remain read-only rendered surfaces. Advanced spreadsheet-style interactions such as multi-column sort, column reordering, pinning, and resize handles are unsupported in the current release boundary. Use app-owned state or a later hardening stage if those interactions are required.

## Visual Fidelity

Tailwind/CSS-like helpers use the contracts in `docs/ui-framework/tw-render-contract.md`. Some effects are rendered exactly, some record intent, and some are bounded egui approximations.

## Platform and Accessibility

Accessibility metadata, live regions, keyboard traversal helpers, and platform descriptors are explicit handoff contracts. Native screen-reader, file-dialog, clipboard, and localization framework integration remains app/platform-owned unless a future release adds reviewed dependencies.
