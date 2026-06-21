# Windows Platform Smoke Artifact

Status: `planned` for runtime support.

The shared showcase compile-checks for Windows targets, but runtime support is
not claimed until the showcase is built and run on a Windows host with visible
rendering, window lifecycle, screenshots/logs, and error scanning.

## Current Evidence

| Check | Result | Notes |
| --- | --- | --- |
| Windows example compile check | passed | `cargo check --target x86_64-pc-windows-gnu --examples` has been used as a contract check. |
| Runtime launch | not claimed | Requires a Windows host or runner. |
| Visible render screenshot | not claimed | Requires runtime capture. |
| Resize/focus/lifecycle | not claimed | Requires runtime capture. |
| Installer/package validation | not claimed | Requires separate packaging evidence. |

## Required Runtime Proof

A Windows support artifact should record at least:

- OS version and Rust target.
- Exact build command and produced executable path.
- Launch command and process/window identity.
- Visible non-black screenshot.
- Resize/focus or equivalent lifecycle checks.
- Captured stdout/stderr or Windows event/log output.
- Error scan showing no app panic, device loss, segmentation fault, or renderer
  failure markers.

Compile-only evidence is useful, but it is not a Windows support claim.
