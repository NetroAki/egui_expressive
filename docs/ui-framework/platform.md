# Platform Integration Guide

Stage 8 keeps platform integration dependency-free in core. The crate provides
pure descriptors and egui-native wiring guidance; host apps decide whether to add
native crates such as `rfd`, `arboard`, `copypasta`, `sys-locale`, or a
localization framework.

## Dependency / License / Security Review

No new native integration dependency was added in Stage 8.

Reviewed options and decision:

| Area | Candidate dependency | Stage 8 decision | Reason |
| --- | --- | --- | --- |
| Clipboard | `arboard`, `copypasta` | Deferred | egui host apps can fulfill `ClipboardCommand`; adding a native crate would expand OS surface. |
| File dialogs | `rfd` | Deferred | `FilePickerRequest` remains a pure intent descriptor; app chooses native dialog crate. |
| File drop | None required | Supported as descriptors | egui exposes dropped-file input; `PlatformDropBatch` maps summaries into editor drop requests. |
| System theme | None required | Docs/descriptors | egui visuals and app shell own theme switching. |
| High-DPI | None required | Docs/descriptors | egui `pixels_per_point` remains source of truth. |
| Localization | `fluent`, `rust-i18n`, `sys-locale` | Deferred | Stage 8 adds guidance/contracts, not string-catalog runtime. |

## Clipboard

Use `ClipboardCommand` as an app-owned copy intent. The host decides whether to
write through egui output or a native clipboard crate.

```rust,no_run
use egui_expressive::ClipboardCommand;

let command = ClipboardCommand::copy_text("Shareable label");
if command.should_log_value() {
    // Safe to include value in app diagnostics.
}
```

Mark secrets/tokens as sensitive so diagnostics do not log copied values.

## File Dialogs

Use `FilePickerRequest` from Forms v2 to describe intent:

```rust,no_run
use egui_expressive::FilePickerRequest;

let request = FilePickerRequest::new("preset", "Open preset").allow_extension("json");
# let _ = request;
```

If an app chooses `rfd` or another native dialog crate, review license, portal
behavior, sandbox behavior, blocking/threading model, and target OS support in
that app before wiring it.

## File Drop

Use `PlatformDropBatch` to normalize platform drop summaries into editor drop
requests without reading files or mutating the filesystem.

```rust,no_run
use egui_expressive::{DroppedFileDescriptor, PlatformDropBatch};

let batch = PlatformDropBatch::new(
    egui::pos2(12.0, 24.0),
    [DroppedFileDescriptor::new("file-1", "design.json")],
);
let editor_request = batch.to_editor_request();
# let _ = editor_request;
```

The host app still owns path validation, MIME sniffing, byte loading, permission
errors, and user prompts.

## System Theme and High-DPI

- Use `SystemThemePreference` to record app/user/system theme intent.
- Use `DisplayScale::new(ctx.pixels_per_point())` when documenting physical
  pixel expectations, screenshots, or hit-target review.
- Prefer egui defaults unless a target platform proves they are insufficient.

```rust,no_run
use egui_expressive::{DisplayScale, SystemThemePreference};

let scale = DisplayScale::new(2.0);
assert_eq!(scale.logical_to_physical(24.0), 48.0);
assert_eq!(SystemThemePreference::Dark.prefers_dark(), Some(true));
```

## App-Provided Backdrop Snapshots

Use `BackdropSnapshotProvider` when an app can supply its own backdrop pixels for
the same egui context/surface. This is a cross-platform substrate, not native
framebuffer capture: the crate asks the installed provider for a logical egui
rect and `ctx.pixels_per_point()`, validates a tightly packed row-major 8-bit
sRGB straight-alpha RGBA snapshot, then optional `wgpu` helpers may blur that
snapshot with the existing source-layer callback path.

```rust,no_run
use egui_expressive::{
    app_provided_backdrop_blur_report, install_backdrop_snapshot_provider,
    BackdropCaptureRequest, BackdropSnapshot, BackdropSnapshotProvider,
};

struct MyProvider;

impl BackdropSnapshotProvider for MyProvider {
    fn capture_backdrop_snapshot(
        &self,
        request: &BackdropCaptureRequest,
    ) -> Result<BackdropSnapshot, egui_expressive::BackdropCaptureError> {
        BackdropSnapshot::new(
            request.requested_width,
            request.requested_height,
            vec![0; request.expected_len()?],
        )
    }
}

# let ctx = egui::Context::default();
install_backdrop_snapshot_provider(&ctx, std::sync::Arc::new(MyProvider));
let report = app_provided_backdrop_blur_report(
    &ctx,
    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(96.0, 64.0)),
    4.0,
);
# let _ = report;
```

`R100-001A` is limited to one provider per egui context/surface. It has no
viewport/window identifier and does not certify multi-window capture, OS
compositor materials, host framebuffer sampling, or Tailwind `backdrop_blur`
parity. Apps that want exact WGPU output must also initialize callback resources
with `init_gpu_effects_for_context(...)` and use the shape helper in an eligible
`wgpu` build.

## WGPU App-Owned Backdrop Sources

`R100-001B` pivots the active cross-device path to app-owned WGPU pixels. With
the `wgpu` feature, hosts may register an app-owned same-frame offscreen backdrop
source for the current egui context/surface. The B2 contract freezes source
identity (`AppOwnedBackdropSurfaceId`), frame freshness (`AppOwnedBackdropFrameId`),
scale, physical size, format, sample count, alpha mode, and the `TextureView`
handle.

B3 adds renderer-bound runtime sampling for that contract. The host must install
the B2 source, initialize the same egui context with `init_gpu_effects_for_context(...)`,
then call `bind_app_owned_offscreen_backdrop_source_for_context(...)` with the
active `egui-wgpu` render state and the matching surface/frame tokens. The report
and shape helpers only return exact/callback output after that sidecar exists,
the installed source is the same source allocation retained by the sidecar, and
the requested logical rect maps to an in-bounds physical subrect. Missing, stale,
same-metadata source reinstall without rebinding, wrong-surface, wrong-scale,
wrong-format, unbound, uninitialized, or non-WebGPU runtime cases remain
non-exact and should use a bounded overlay fallback.

This WGPU path is for pixels the app owns. It is not OS/native compositor capture,
not browser `backdrop-filter`, not a foreign-window readback, and not default
`Tw::backdrop_blur` behavior. Web/wasm support means a WebGPU-capable host can
provide the same app-owned source contract; browsers without WebGPU must remain
non-exact rather than falling back to screenshots or CSS blur.

### R100-001B WGPU-First Support Matrix

| Target family | Supported app-owned path | Still unsupported | Smoke proof required before app release |
| --- | --- | --- | --- |
| Linux desktop | Same-context app-owned WGPU source with renderer-bound sidecar | Native compositor/framebuffer capture | Context loss, resize, high-DPI, and rebind-after-source-reinstall smoke |
| Windows desktop | Same contract through the app's egui-wgpu renderer | DWM/window capture | Device smoke for the app renderer/backend combination |
| macOS/iOS | Same contract through WGPU/Metal where the host initializes WebGPU resources | ScreenCaptureKit/CoreGraphics capture without a later child plan | Permission/privacy review plus lifecycle smoke |
| Android | Same contract where the host can provide a fresh app-owned texture | System screen capture | Rotation, DPI, pause/resume, and context-loss smoke |
| Web/wasm | Same contract only when WebGPU is available and the host binds the source | Browser CSS `backdrop-filter`, canvas screenshot fallback, native browser pixels | Browser runtime smoke; absent WebGPU returns non-exact/no shape |

Manual/device smoke checklist for hosts using the B3 helper path:

1. Initialize WGPU resources with `init_gpu_effects_for_context(...)` for the same `egui::Context` that will draw the helper.
2. Install a fresh app-owned source with matching surface/frame tokens, bind it with `bind_app_owned_offscreen_backdrop_source_for_context(...)`, and verify report/shape helpers return exact only after binding succeeds.
3. Reinstall a same-metadata source without rebinding and verify helpers return non-exact until the new source is bound.
4. Resize the surface and verify old-size requests fail closed while a reinstalled and rebound source succeeds.
5. Change or simulate high-DPI scale and verify scale-mismatched sources return non-exact/no shape.
6. Trigger WGPU context loss or renderer recreation and verify the app reinitializes resources, reinstalls, and rebinds before claiming exactness.
7. On mobile, rotate the device and exercise pause/resume before accepting production support.
8. On web, test at least one browser with WebGPU enabled and one browser/path without usable WebGPU; the unavailable path must remain non-exact and must not use CSS/screenshot fallback.

## Native Backdrop Adapter Substrate

`R100-001B` starts native backdrop work as a staged, feature-gated program. The
common `native-backdrop` substrate freezes adapter feature names and shared
initialization errors, but it does not capture pixels by itself. Future platform
providers must still implement `BackdropSnapshotProvider`, install per egui
context/surface, and feed the existing app-provided snapshot path.

Reserved native adapter feature names:

- `native-backdrop` — common substrate only.
- `native-backdrop-x11` — future explicit bound X11 client-window provider.
- `native-backdrop-macos` — future permissioned macOS bound-window provider.
- `native-backdrop-windows` — future explicit Windows bound-window provider.
- `native-backdrop-wayland` — future permissioned Wayland portal provider.
- `native-backdrop-android` — future permissioned Android app/screen capture
  feasibility track.

All native adapter features must remain optional and disabled by default.
`R100-001B` common substrate does not promote
`GpuEffectSource::HostFramebufferBackdrop`, does not add monitor/window
enumeration, and does not change `Tw::backdrop_blur` behavior.

### Deferred Native Adapter Appendix

B5 preserves native-adapter planning as a deferred backlog. It does not mark any
adapter as implemented and does not make native capture part of the default
runtime contract.

| Deferred adapter | Required proof before implementation | Current status |
| --- | --- | --- |
| X11 bound-window snapshot | Oracle-approved child plan, visible-window and occlusion semantics, target feature gate, manual X11 smoke, no broad monitor capture claim | Superseded by WGPU-first track until explicitly resumed |
| Windows bound-window / host-renderer capture | API feasibility, DPI/window ownership proof, DWM/host-renderer boundary, permission/privacy review, manual target smoke | Feasibility only |
| macOS/iOS permissioned capture | ScreenCaptureKit/CoreGraphics feasibility, sandbox and permission-prompt review, lifecycle smoke, app-store-policy check | Feasibility only |
| Wayland portal/PipeWire | Portal consent model, PipeWire stream lifecycle, compositor variance matrix, explicit user-facing permission story | Feasibility only |
| Android app/screen capture | MediaProjection or host-owned-surface feasibility, rotation/pause/resume lifecycle smoke, permission UX, privacy proof | Feasibility only |

Any resumed adapter must either install a validated `BackdropSnapshotProvider` for
the app-provided snapshot path or introduce a separate Oracle-approved source
contract. Native adapters must not relabel arbitrary OS/compositor pixels as the
B3 app-owned WGPU source, and `GpuEffectSource::HostFramebufferBackdrop` remains
unsupported until a later approved contract changes that exact source behavior.

## Cross-Platform Boundary

Validation in this repo is Linux-first. Windows/macOS claims require host-app
testing or a later CI/release-readiness stage. Stage 8 documents the adapter
boundary; it does not certify native clipboard, file dialogs, screen readers, or
IME behavior on every OS.
