# Android Platform Smoke Artifact

Status: `validated-bounded` for the shared showcase on an Android emulator.

The Android path builds the shared showcase into an APK, installs it on an API 35
x86_64 emulator, launches the native activity, captures visible screenshots, and
scans logs for runtime failures. This is emulator evidence for the showcase, not
a blanket guarantee for every device, GPU, input method, packaging channel, or
Android version.

## Environment

| Field | Value |
| --- | --- |
| Package | `dev.egui_expressive.showcase` |
| Activity | `android.app.NativeActivity` |
| Emulator | API 35 x86_64 AVD |
| APK path | generated under the configured Cargo target directory |
| Rust targets | Android shared-showcase targets configured in `platform/android/Cargo.toml` |

## Reproducible Checks

```bash
cargo check --manifest-path platform/android/Cargo.toml \
  --target aarch64-linux-android --no-default-features --features shared-showcase

cargo apk build --manifest-path platform/android/Cargo.toml \
  --target x86_64-linux-android --no-default-features --features shared-showcase
```

A runtime smoke additionally requires an emulator or device:

```bash
adb install -r <generated-showcase.apk>
adb shell am start -n dev.egui_expressive.showcase/android.app.NativeActivity
adb exec-out screencap -p > android-showcase.png
adb logcat -d > android-logcat.txt
```

## Results

| Check | Result | Notes |
| --- | --- | --- |
| Shared-showcase compile | passed | Android library/showcase checks build for configured Android targets. |
| APK build | passed | x86_64 debug APK was generated and signed by the debug toolchain. |
| Install/launch | passed | APK installed and launched on the API 35 x86_64 emulator. |
| Portrait screenshot | passed | Visible non-black 1080x1920 PNG captured. |
| Landscape/resume | passed | Rotation/resume screenshots remained visible and non-black. |
| Log scan | passed | No fatal app/runtime markers were found in the captured log slice. |

## Boundaries

- Emulator proof does not certify every physical device, vendor GPU, Android
  release, text input path, system theme, accessibility service, or store channel.
- Release signing, Play Store submission, and production crash-reporting are not
  implied by this smoke.
- Android support claims should stay scoped to the documented APK/emulator proof
  until device and distribution evidence is added.
