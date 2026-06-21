# macOS Platform Smoke Artifact

Status: `planned` for runtime support.

macOS support is expected to be practical through egui/eframe, but this repository
does not claim macOS runtime support until the shared showcase is built and run
on a macOS host or runner with visible rendering, lifecycle metadata, screenshots,
and logs.

## Current Evidence

| Check | Result | Notes |
| --- | --- | --- |
| Compile/contract readiness | planned | macOS build checks require a macOS toolchain/runner. |
| Runtime launch | not claimed | Requires a macOS host or CI runner. |
| Visible render screenshot | not claimed | Requires runtime capture. |
| Resize/focus/lifecycle | not claimed | Requires runtime capture. |

## Required Runtime Proof

A macOS support artifact should record:

- macOS version, hardware/runner type, and Rust target.
- Exact build and launch commands.
- Visible non-black screenshot.
- Window resize/focus/lifecycle behavior.
- Captured app logs and error scan.
- Any notarization/package evidence if distribution support is claimed.
