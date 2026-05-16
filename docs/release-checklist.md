# Release Checklist

Use this checklist before publishing or tagging a release of `egui_expressive`.

## Required Local Gate

Run on Linux unless another stage adds additional runners:

```bash
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo build --examples
cargo test --test visual_diff_harness
node --check illustrator-plugin/plugin.js
node --check illustrator-plugin/plugin.test.cjs
node illustrator-plugin/plugin.test.cjs
cargo check --manifest-path preview/Cargo.toml
cargo test --manifest-path preview/Cargo.toml
```

## Documentation Gate

- `README.md` describes current product scope and links the docs index.
- `docs/ui-framework/index.md` links all user-facing framework docs.
- `docs/migration-guide.md` records migration notes for changed/deprecated public surfaces.
- `docs/versioning-policy.md` states SemVer and compatibility rules.
- `CHANGELOG.md` has user-facing entries for the release.

## Compatibility Gate

- Compatibility aliases kept for DAW/creative-editor names unless a migration entry says otherwise.
- Unsupported/approximate rendering behavior remains documented in `docs/ui-framework/tw-render-contract.md`.
- Platform/native integrations do not claim behavior beyond the dependency-free descriptors in `docs/ui-framework/platform.md`.
- Advanced data-grid column interactions remain explicitly unsupported unless a later hardening stage implements them.

## Artifact Gate

- Visual-diff fixture manifest uses committed fixture pairs for regression/parity governance.
- `backdrop-supported-app-snapshot-blur` is an exact app-provided snapshot artifact row only; it proves snapshot-input blur with a deterministic source PNG and does not certify native framebuffer capture or Tailwind backdrop parity.
- Current-code visual proof exists only for the named draw subset covered by `cargo test --lib current_render` / `cargo test --all-targets`, including the R100-009A `phase7-supported-compound-hole-fill` proof, the hardened `vector-clip-nested` clip-path proof, and the R100-009B `compositing-blend-boundary` decoded-RGBA proof against the committed headless pair with a bounded asserted blue-mask `Color32` quantization correction; it does not upgrade the whole committed fixture corpus to current-render proof.
- Failed visual-diff heatmaps are uploaded by CI when available.
- No secrets, paid services, network-only validation, or OS mutation are required by release gates.
