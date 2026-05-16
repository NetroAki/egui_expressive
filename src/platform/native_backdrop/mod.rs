//! Common native backdrop adapter substrate.
//!
//! This module intentionally does not capture pixels by itself. It only freezes
//! shared feature names, platform labels, and initialization errors for later
//! platform-specific providers. Real adapters must still feed the existing
//! [`crate::platform::BackdropSnapshotProvider`] contract and install providers
//! per `egui::Context`; no global capture/session state is introduced here.
//!
//! `GpuEffectSource::HostFramebufferBackdrop` remains unsupported. Native
//! adapters in `R100-001B` are app/native ways to supply
//! `GpuEffectSource::AppProvidedBackdropSnapshot`, not a backend-global backdrop
//! capture claim.

use std::fmt;

/// Feature name for the common native backdrop adapter substrate.
pub const NATIVE_BACKDROP_FEATURE: &str = "native-backdrop";

/// Feature name reserved for a future Linux/X11 bound-window provider.
pub const NATIVE_BACKDROP_X11_FEATURE: &str = "native-backdrop-x11";

/// Feature name reserved for a future macOS bound-window provider.
pub const NATIVE_BACKDROP_MACOS_FEATURE: &str = "native-backdrop-macos";

/// Feature name reserved for a future Windows bound-window provider.
pub const NATIVE_BACKDROP_WINDOWS_FEATURE: &str = "native-backdrop-windows";

/// Feature name reserved for a future Wayland portal/provider path.
pub const NATIVE_BACKDROP_WAYLAND_FEATURE: &str = "native-backdrop-wayland";

/// Platform bucket for native backdrop adapter planning and diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeBackdropPlatform {
    /// Linux/X11 explicit bound client-window provider.
    X11,
    /// macOS permissioned bound-window/surface provider.
    Macos,
    /// Windows explicit bound-window provider.
    Windows,
    /// Wayland portal/session provider.
    WaylandPortal,
}

impl NativeBackdropPlatform {
    /// Returns the Cargo feature name reserved for this platform adapter.
    pub const fn feature_name(self) -> &'static str {
        match self {
            Self::X11 => NATIVE_BACKDROP_X11_FEATURE,
            Self::Macos => NATIVE_BACKDROP_MACOS_FEATURE,
            Self::Windows => NATIVE_BACKDROP_WINDOWS_FEATURE,
            Self::WaylandPortal => NATIVE_BACKDROP_WAYLAND_FEATURE,
        }
    }
}

/// Initialization failures shared by future native backdrop adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeBackdropInitError {
    /// The current target OS/session cannot support the requested adapter.
    UnsupportedPlatform,
    /// The current desktop/session type is unsupported, such as a compositor or
    /// portal path that cannot provide a bound-window snapshot.
    UnsupportedSession,
    /// The caller supplied an invalid, stale, or mismatched native surface handle.
    InvalidSurfaceHandle,
    /// The OS or portal explicitly denied screen/window capture permission.
    PermissionDenied,
    /// The OS requires an interactive permission grant before capture can start.
    PermissionRequired,
    /// A platform backend was unavailable or failed during initialization.
    BackendUnavailable(String),
}

impl fmt::Display for NativeBackdropInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                f.write_str("native backdrop capture is unsupported on this platform")
            }
            Self::UnsupportedSession => {
                f.write_str("native backdrop capture is unsupported in this session")
            }
            Self::InvalidSurfaceHandle => {
                f.write_str("native backdrop capture received an invalid surface handle")
            }
            Self::PermissionDenied => f.write_str("native backdrop capture permission was denied"),
            Self::PermissionRequired => {
                f.write_str("native backdrop capture requires user permission")
            }
            Self::BackendUnavailable(message) => {
                write!(f, "native backdrop backend unavailable: {message}")
            }
        }
    }
}

impl std::error::Error for NativeBackdropInitError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_feature_names_are_frozen() {
        assert_eq!(NATIVE_BACKDROP_FEATURE, "native-backdrop");
        assert_eq!(
            NativeBackdropPlatform::X11.feature_name(),
            "native-backdrop-x11"
        );
        assert_eq!(
            NativeBackdropPlatform::Macos.feature_name(),
            "native-backdrop-macos"
        );
        assert_eq!(
            NativeBackdropPlatform::Windows.feature_name(),
            "native-backdrop-windows"
        );
        assert_eq!(
            NativeBackdropPlatform::WaylandPortal.feature_name(),
            "native-backdrop-wayland"
        );
    }

    #[test]
    fn init_errors_display_without_sensitive_state() {
        let variants = [
            NativeBackdropInitError::UnsupportedPlatform,
            NativeBackdropInitError::UnsupportedSession,
            NativeBackdropInitError::InvalidSurfaceHandle,
            NativeBackdropInitError::PermissionDenied,
            NativeBackdropInitError::PermissionRequired,
            NativeBackdropInitError::BackendUnavailable("x11 connection failed".to_owned()),
        ];

        for error in variants {
            let message = error.to_string();
            assert!(!message.is_empty());
            assert!(!message.contains("password"));
            assert!(!message.contains("token"));
        }
    }
}
