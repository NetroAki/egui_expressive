# Versioning Policy

`egui_expressive` uses SemVer once public releases begin.

The release-facing API status for current public modules lives in `docs/ui-framework/api-stability.md`. That map is the first place to check before treating a module, crate-root re-export, compatibility alias, or feature-gated surface as a supported public contract.

## Public API Compatibility

- Pre-1.0 staged builds may still adjust public APIs, but Stage 11 keeps preferred paths, compatibility-only aliases, and experimental surfaces explicit in `docs/ui-framework/api-stability.md`.
- Patch releases may add docs, examples, tests, and backwards-compatible fixes.
- Minor releases may add new modules, builders, descriptors, examples, and optional feature-gated behavior.
- Major releases may remove compatibility aliases, rename public APIs, or change persisted data formats.

## Compatibility Aliases

Compatibility namespaces such as `daw` and `widgets::daw_editors` are retained until a major release or an explicit migration plan removes them. Prefer DAW-neutral public paths for new app code when both exist.

Compatibility aliases should not receive new generic features first. Add or document the neutral path, then keep aliases as migration support when practical.

## Feature Flags

- `default` remains lightweight and avoids optional GPU/native dependencies.
- `wgpu` / `gpu-effects` are optional acceleration surfaces.
- New runtime dependencies require dependency/license/security review before they become default behavior.

## Rendering and Platform Claims

Rendering fidelity, accessibility, platform dialogs, clipboard, and high-DPI behavior are versioned according to documented support contracts. Approximate or app-owned behavior is not treated as a compatibility guarantee unless the docs explicitly say it is supported.

Experimental or feature-gated modules such as GPU acceleration, debug overlays, and devtools are not default stability guarantees unless promoted by the maturity rubric and documented in the API stability map.
