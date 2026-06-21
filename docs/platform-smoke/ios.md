# iOS Platform Smoke Artifact

Status: `planned` for runtime support.

iOS support is not claimed until the shared showcase, or an equivalent iOS host
surface, is built and run on an iOS simulator or device with visible rendering,
lifecycle metadata, screenshots, and logs.

## Current Evidence

| Check | Result | Notes |
| --- | --- | --- |
| Library compile readiness | planned | Requires an Apple toolchain and configured iOS target. |
| Simulator/device launch | not claimed | Requires Xcode simulator or physical device. |
| Visible render screenshot | not claimed | Requires runtime capture. |
| Rotation/resume lifecycle | not claimed | Requires runtime capture. |

## Required Runtime Proof

An iOS support artifact should record:

- Xcode/iOS SDK version and simulator/device identity.
- Rust target and build command.
- App launch command or Xcode run metadata.
- Visible non-black screenshot.
- Rotation/resume lifecycle result.
- Captured logs and error scan.
- Signing/provisioning notes if device or distribution support is claimed.
