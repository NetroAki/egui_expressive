# Release Checklist

Use this checklist before tagging, pushing a release branch, or publishing
`egui_expressive` to a package registry.

Current status: release candidate only. This working tree has Linux-focused
runtime evidence and bounded Web/Android evidence, but the repository owner has
not approved a push or registry publish in the current review. Keep all public
claims evidence-scoped until that approval is explicit.

## Initial Release Scope

| Area | Release position |
| --- | --- |
| Core crate | Candidate pre-1.0 egui-native design layer. Breaking changes may occur before `1.0`. |
| Linux | Primary runtime-validated path with local X11 and Wayland virtual-desktop smoke evidence. |
| Web and Android | Bounded showcase evidence exists; broader support still requires app-specific validation. |
| Windows, macOS, iOS | Planned until runtime artifacts are supplied. Compile-only checks are not support claims. |
| Design-tool integrations | Useful integration paths, but not blanket fidelity/support guarantees. |
| Registry publish | Not authorized by this checklist alone. Requires explicit owner approval and a clean package dry-run. |

## Required Local Validation

Run on Linux before any release push or registry publish:

```bash
cargo fmt --check
cargo test --all-targets -j 1
cargo build --examples
cargo clippy --all-targets --all-features -- -D warnings

node --check illustrator-plugin/plugin.js
node --check illustrator-plugin/plugin.test.cjs
node illustrator-plugin/plugin.test.cjs

bash -n tools/linux_cross_platform_smoke.sh
bash -n tools/linux_wayland_sway_smoke.sh
cargo package --allow-dirty --list
cargo package --allow-dirty
```

Linux platform probes for the current release-candidate scope:

```bash
tools/linux_cross_platform_smoke.sh
tools/linux_wayland_sway_smoke.sh
```

Optional bounded probes for other platform slices:

```bash
cargo check --manifest-path platform/web/Cargo.toml
cargo build --manifest-path platform/web/Cargo.toml --target wasm32-unknown-unknown --release
cargo check --manifest-path platform/android/Cargo.toml \
  --target aarch64-linux-android --no-default-features --features shared-showcase
```

Windows runtime support remains unclaimed until supplied on an appropriate host,
and compile-only evidence does not replace runtime proof. Linux, Android, and Web
evidence is recorded under `docs/platform-smoke/`; support claims must stay
within those documented bounds.

## Claim Rules

- Do not describe the whole crate as production-ready until all claimed platform,
  renderer, packaging, and integration gates have corresponding artifacts.
- Cross-platform support claims for Linux, Windows, macOS, iOS, Android, or Web
  require a docs artifact that records platform/runtime, build pass, renderer
  backend, visible-render proof, lifecycle checks, logs/artifacts, and final
  result.
- Linux support claims require the Linux smoke in `docs/platform-smoke/linux.md`.
  The current artifacts prove bounded local virtual-desktop X11/Xvfb/Openbox and
  Wayland/Sway/wlroots paths; they do not prove every distro, compositor,
  GNOME/KDE/DRM session, real GPU, focus manager, or device-loss scenario.
- Android support claims require APK build and emulator/device identity, OS/API
  version, renderer/backend, visible-render screenshot or log path,
  rotation/density/lifecycle result, and final result.
- Web support claims require wasm/Web compile, browser/version, renderer backend,
  visible-render screenshot or log path, resize/DPI behavior, lifecycle result,
  and renderer fallback status.
- Design-tool export claims must preserve fail-closed behavior: unsupported
  rasters, plugin items, charts, text effects, blend modes, and live effects must
  remain visible as unsupported/approximate rather than silently exact.
- Package publish requires `cargo package --list` review, clean package dry-run,
  and confirmation that no private/local artifacts are included.

## Release-owner checklist

Before broadening any support claim, record:

1. Platform, OS/runtime version, Rust target, renderer/backend, and build command.
2. Visible-render screenshots or equivalent logs.
3. Resize/focus/lifecycle behavior where applicable.
4. App stdout/stderr or platform logs.
5. Explicit boundaries for unsupported or untested environments.
6. A clean package list/dry-run when publishing a crate.
