# egui_expressive UI Framework Docs

This index is the release-facing entry point for the framework docs. Use it with `docs/ui-framework/module-map.md` when deciding where to work next.

## Start Here

- `overview.md` — product shape and framework overview.
- `module-map.md` — source navigation map for active modules, examples, tests, and release boundaries.
- `api-stability.md` — public API stability map, preferred paths, compatibility aliases, experimental surfaces, and feature flags.
- `layout.md` — layout primitives, app-shell composition, responsive dashboard patterns.
- `tokens.md` — design tokens, theme tokens, typography, icons, and Material 3 visual guidance.

## Product Surfaces

- `data-widgets.md` — data grid, tree table, property grid, inline edit adapters, release limits.
- `forms.md` — Forms v2 schemas, validation, rich input descriptors, input correctness contracts.
- `editor-canvas.md` — editor/canvas interactions, snapping, alignment, inspector/drop descriptors.
- `interaction.md` — commands, shortcuts, focus traversal, undo/redo, feedback dispatch.
- `accessibility.md` — metadata, keyboard conventions, live regions, screen-reader checklist, i18n/RTL guidance.
- `platform.md` — clipboard, file dialog/drop, system theme, high-DPI, dependency review.
- `tw-render-contract.md` — Tailwind/CSS-style utility support matrix and approximation contract.

## Release Readiness

- `../release-checklist.md` — commands and review items before publishing.
- `../versioning-policy.md` — SemVer policy and compatibility boundaries.
- `../migration-guide.md` — migration notes for downstream apps.
- `../../CHANGELOG.md` — user-facing changes by release/stage.

## Validation Entry Points

- `tests/api_surface_smoke.rs` — representative public API reachability checks for preferred, beta-supported, and compatibility/default-feature symbols.
- `tests/interaction_smoke.rs` — GUI-free release smoke coverage for key state/model surfaces.
- `tests/performance_smoke.rs` — deterministic large-input/performance-smoke proxies without wall-clock thresholds.
- `tests/visual_diff_harness.rs` — manifest-driven visual regression/parity harness.
- `tests/raster_vectorization.rs` — raster-to-scene-node vectorization regression proof included by the all-targets release gate.
