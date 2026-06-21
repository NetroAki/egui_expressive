# Platform Integration and Support Evidence

`egui_expressive` keeps core platform integration dependency-light. Platform
support claims are evidence-scoped: a target is described as validated only when
a corresponding build/runtime artifact records the environment, renderer path,
visible rendering, lifecycle behavior, logs, and boundaries.

## Current platform status

| Platform | Status | Evidence boundary |
| --- | --- | --- |
| Linux | validated | Local virtual-desktop smoke passes on X11/Xvfb/Openbox and Wayland/Sway/wlroots. See `docs/platform-smoke/linux.md`. |
| Web | validated-bounded | The Web harness builds for `wasm32-unknown-unknown` and runs in Chromium loopback. See `docs/platform-smoke/web.md`. |
| Android | validated-bounded | The shared-showcase APK builds and runs on an API 35 x86_64 emulator. See `docs/platform-smoke/android.md`. |
| Windows | planned | Compile checks are useful, but runtime support requires a Windows visible-render artifact. |
| macOS | planned | Runtime support requires a macOS host/runner artifact. |
| iOS | planned | Runtime support requires simulator or device evidence. |

Compile-only checks do not replace runtime support evidence. Support claims should
name the artifact that backs them and keep untested environments planned.

## Support artifact schema

The public support vocabulary lives in `src/platform/support.rs`:

- `PlatformFamily` identifies Linux, Windows, macOS, iOS, Android, and Web.
- `PlatformSupportStatus` records whether a target is supported, planned,
  blocked, unsupported, or not run.
- `PlatformSupportArtifact` records build status, OS/runtime version, Rust
  target, renderer backend, GPU/software path, lifecycle checks, logs, artifact
  path, and final result.
- `PlatformSmokeResult` records pass/fail/blocked/not-run outcome.

A support artifact is strong enough for a public support label only when it has a
passing result, complete lifecycle checks, runtime metadata, logs, and artifacts.

## Runtime lifecycle expectations

Desktop targets should record:

- build;
- launch;
- visible rendering;
- resize;
- focus or focused-toplevel behavior;
- high-DPI or scale behavior;
- renderer lifecycle/teardown.

Mobile targets should record:

- build;
- launch;
- visible rendering;
- rotation;
- pause/resume;
- high-DPI/density;
- renderer lifecycle/teardown.

Web targets should record:

- wasm/Web build;
- browser launch;
- visible rendering;
- resize/viewport behavior;
- high-DPI or scale behavior;
- renderer lifecycle and console/log scan.

## Native and OS integration boundaries

The core crate does not add native file dialogs, screen capture, system clipboard
mutation, localization runtimes, signing/provisioning, or store submission. Host
applications can layer those capabilities around `egui_expressive` and should
record their own permissions, privacy, and lifecycle evidence before making
product support claims.

Native backdrop and capture-related feature flags are diagnostic/planning
surfaces unless a host supplies validated app-owned pixels or explicit platform
capture artifacts. No default feature performs broad monitor capture or samples
another application's framebuffer.

## Privacy and redaction

Platform artifacts should avoid secrets and personal data. Prefer relative or
generic artifact paths in docs, redact user home directories from logs, and avoid
publishing screenshots that contain private content.
