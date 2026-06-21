# Linux Platform Smoke Artifact

Status: `validated` for the bounded Linux runtime path in the `0.1.0` release.
This page is product documentation, not a broad support guarantee.

The shared showcase has been exercised locally on both Linux display stacks that
matter for most egui desktop applications:

- X11 through `Xvfb` plus `openbox`.
- Wayland through headless `sway`/wlroots plus `grim` screenshots.

These checks prove that the showcase can build, launch, render visible non-black
frames, survive resize/lifecycle operations, and exit cleanly on the validated
host. They do not claim every distro, compositor, GPU driver, package format, or
accessibility portal.

## Local Environment

| Field | Value |
| --- | --- |
| Host | Local Linux virtual desktop |
| OS | Linux 7.0.11 x86_64 |
| Rust target | `x86_64-unknown-linux-gnu` |
| X11 path | `Xvfb` + `openbox`, screenshots via `scrot`/ImageMagick |
| Wayland path | headless `sway`/wlroots, screenshots via `grim` |

## Reproducible Commands

X11 virtual desktop smoke:

```bash
ARTIFACT_DIR=/tmp/egui-expressive-linux-x11 \
CARGO_TARGET_DIR=/tmp/egui-expressive-linux-x11-target \
  tools/linux_cross_platform_smoke.sh
```

Wayland virtual desktop smoke:

```bash
ARTIFACT_DIR=/tmp/egui-expressive-linux-wayland \
CARGO_TARGET_DIR=/tmp/egui-expressive-linux-wayland-target \
  tools/linux_wayland_sway_smoke.sh
```

Both scripts write screenshots, compositor/window metadata, app logs, image
statistics, lifecycle status, timing samples, and a `summary.txt` file under the
chosen `ARTIFACT_DIR`.

## X11 Results

| Check | Result | Evidence |
| --- | --- | --- |
| Example build | passed | `tools/linux_cross_platform_smoke.sh` builds `examples/cross_platform_showcase.rs`. |
| Bounded launch | passed | Normal-DPI and high-DPI Xvfb runs both discovered the `Cross-Platform Showcase` X11 window and kept the process alive until capture. |
| Focus smoke | passed | Openbox focus activation passed in both normal-DPI and high-DPI runs. |
| Resize smoke | passed | Normal-DPI resize recorded a `640x480` window. |
| Visual screenshot | passed | Normal-DPI and high-DPI screenshots were non-black with non-zero variance. |
| App error scan | passed | App stdout/stderr contained no panic, device-loss, segmentation-fault, or renderer-error markers. |

## Wayland/Sway Results

| Check | Result | Evidence |
| --- | --- | --- |
| Example build | passed | `tools/linux_wayland_sway_smoke.sh` builds `examples/cross_platform_showcase.rs`. |
| Wayland backend | passed | The script launches the app with `WINIT_UNIX_BACKEND=wayland` under headless Sway/wlroots. |
| Bounded launch | passed | Normal-DPI and scale-2 high-DPI runs found a focused `Cross-Platform Showcase` Wayland toplevel in the Sway tree. |
| Visual screenshot | passed | Normal-DPI and high-DPI `grim` screenshots were non-black with non-zero variance. |
| Floating resize attempt | passed with app/window limits | Normal DPI changed from tiled fullscreen to the app minimum size; high-DPI changed to a smaller floating logical size. The script records the actual Sway tree after resize so claims stay exact. |
| App error scan | passed | App stdout/stderr contained no panic, device-loss, segmentation-fault, or renderer-error markers. |

## Secondary Remote Linux Check

A secondary source-only run also passed on another Linux desktop host for
normal/high-DPI Xvfb rendering, resize/lifecycle, and log scanning. That host did
not have `openbox`, so focus activation was recorded as skipped there and is not
used for the focus claim above.

## Support Mapping

| Support field | Value |
| --- | --- |
| Platform | Linux |
| Release status | validated for the bounded local virtual-desktop smoke scope |
| Public support label | Linux primary runtime path for `0.1.0`; broader support remains evidence-scoped |
| Renderer/backend | egui/eframe native renderer under X11/Xvfb/Openbox and Wayland/Sway/wlroots with software-rendering-friendly settings |
| Lifecycle checks | build, launch, visible rendering, X11 focus, X11 resize, Wayland focused toplevel, Wayland floating resize attempt, normal DPI, high DPI/scale, bounded teardown/relaunch |
| Logs/artifacts | Generated under the caller-provided `ARTIFACT_DIR` for each smoke run |

## Boundaries

- This is a bounded Linux compatibility proof, not a guarantee for every Linux
  distribution or desktop environment.
- Wayland proof uses headless Sway/wlroots; it does not prove GNOME, KDE, nested
  compositor, DRM-session, or portal-specific behavior.
- X11 proof uses Xvfb/Openbox; it does not prove every physical display server,
  window manager, or GPU driver.
- Windows, macOS, iOS, Android, and Web claims require their own artifacts.
- Publishing the crate does not imply a blanket production-ready claim for every
  renderer, platform, or design-tool integration.
