//! Dependency-free platform integration descriptors.
//!
//! These types document how app code bridges egui-native clipboard, file-drop,
//! system-theme, and high-DPI facts without this crate mutating the OS or adding
//! native dependencies in core.

pub mod backdrop;
pub mod clipboard;
pub mod file_drop;
#[cfg(feature = "native-backdrop")]
pub mod native_backdrop;
pub mod system;

#[cfg(feature = "wgpu")]
pub use backdrop::{
    install_app_owned_offscreen_backdrop_source, load_app_owned_offscreen_backdrop_source,
    AppOwnedBackdropAlphaMode, AppOwnedBackdropFrameId, AppOwnedBackdropSurfaceId,
    AppOwnedOffscreenBackdropSource, SharedAppOwnedOffscreenBackdropSource,
};
pub use backdrop::{
    install_backdrop_snapshot_provider, load_backdrop_snapshot_provider, BackdropCaptureError,
    BackdropCaptureRequest, BackdropSnapshot, BackdropSnapshotProvider,
    SharedBackdropSnapshotProvider, MAX_BACKDROP_SNAPSHOT_AXIS,
};
pub use clipboard::ClipboardCommand;
pub use file_drop::{DroppedFileDescriptor, PlatformDropBatch};
#[cfg(feature = "native-backdrop")]
pub use native_backdrop::{
    NativeBackdropInitError, NativeBackdropPlatform, NATIVE_BACKDROP_FEATURE,
    NATIVE_BACKDROP_MACOS_FEATURE, NATIVE_BACKDROP_WAYLAND_FEATURE,
    NATIVE_BACKDROP_WINDOWS_FEATURE, NATIVE_BACKDROP_X11_FEATURE,
};
pub use system::{DisplayScale, SystemThemePreference};
