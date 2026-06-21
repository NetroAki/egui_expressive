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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeBackdropSupportFamily {
    LinuxX11,
    WaylandPortal,
    Windows,
    Macos,
    Ios,
    Android,
    Web,
}

impl NativeBackdropSupportFamily {
    pub const fn label(self) -> &'static str {
        match self {
            Self::LinuxX11 => "linux-x11",
            Self::WaylandPortal => "wayland-portal",
            Self::Windows => "windows",
            Self::Macos => "macos",
            Self::Ios => "ios-support-family",
            Self::Android => "android-support-family",
            Self::Web => "web-support-family",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeBackdropSupportState {
    Unsupported,
    ContractOnly,
    PermissionRequired,
    SmokeTestRequired,
    SmokeTestPassed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeBackdropPermissionState {
    NotRequested,
    Required,
    Granted,
    Denied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeBackdropSourceScope {
    AppWindowSurface,
    ForeignWindow,
    Monitor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeBackdropContractDiagnostic {
    PermissionRequired,
    PermissionDenied,
    UnsupportedPlatform,
    UnsupportedSession,
    InvalidSurfaceHandle,
    SourceOwnershipMismatch,
    SmokeArtifactMissing,
}

impl NativeBackdropContractDiagnostic {
    pub const fn redacted_message(self) -> &'static str {
        match self {
            Self::PermissionRequired => "native backdrop permission is required",
            Self::PermissionDenied => "native backdrop permission was denied",
            Self::UnsupportedPlatform => "native backdrop platform is unsupported",
            Self::UnsupportedSession => "native backdrop session is unsupported",
            Self::InvalidSurfaceHandle => "native backdrop surface handle is invalid",
            Self::SourceOwnershipMismatch => {
                "native backdrop source is outside the app/window contract"
            }
            Self::SmokeArtifactMissing => {
                "native backdrop production label requires a smoke artifact"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeBackdropSmokeArtifact {
    pub family: NativeBackdropSupportFamily,
    pub support_state: NativeBackdropSupportState,
    pub permission_state: NativeBackdropPermissionState,
    pub source_scope: NativeBackdropSourceScope,
    pub redaction_confirmed: bool,
}

impl NativeBackdropSmokeArtifact {
    pub fn is_production_evidence(&self) -> bool {
        self.support_state == NativeBackdropSupportState::SmokeTestPassed
            && self.permission_state == NativeBackdropPermissionState::Granted
            && self.source_scope == NativeBackdropSourceScope::AppWindowSurface
            && self.redaction_confirmed
    }
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
            Self::BackendUnavailable(_) => f.write_str("native backdrop backend unavailable"),
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
    fn support_family_labels_cover_stage3_matrix() {
        let families = [
            NativeBackdropSupportFamily::LinuxX11,
            NativeBackdropSupportFamily::WaylandPortal,
            NativeBackdropSupportFamily::Windows,
            NativeBackdropSupportFamily::Macos,
            NativeBackdropSupportFamily::Ios,
            NativeBackdropSupportFamily::Android,
            NativeBackdropSupportFamily::Web,
        ];

        for family in families {
            assert!(!family.label().is_empty());
        }
    }

    #[test]
    fn smoke_artifact_requires_app_scope_permission_and_redaction() {
        let artifact = NativeBackdropSmokeArtifact {
            family: NativeBackdropSupportFamily::Windows,
            support_state: NativeBackdropSupportState::SmokeTestPassed,
            permission_state: NativeBackdropPermissionState::Granted,
            source_scope: NativeBackdropSourceScope::AppWindowSurface,
            redaction_confirmed: true,
        };
        assert!(artifact.is_production_evidence());

        let foreign = NativeBackdropSmokeArtifact {
            source_scope: NativeBackdropSourceScope::ForeignWindow,
            ..artifact.clone()
        };
        assert!(!foreign.is_production_evidence());

        let permission_not_requested = NativeBackdropSmokeArtifact {
            permission_state: NativeBackdropPermissionState::NotRequested,
            ..artifact
        };
        assert!(!permission_not_requested.is_production_evidence());
    }

    #[test]
    fn native_contract_diagnostics_are_redacted() {
        for diagnostic in [
            NativeBackdropContractDiagnostic::PermissionRequired,
            NativeBackdropContractDiagnostic::PermissionDenied,
            NativeBackdropContractDiagnostic::UnsupportedPlatform,
            NativeBackdropContractDiagnostic::UnsupportedSession,
            NativeBackdropContractDiagnostic::InvalidSurfaceHandle,
            NativeBackdropContractDiagnostic::SourceOwnershipMismatch,
            NativeBackdropContractDiagnostic::SmokeArtifactMissing,
        ] {
            let message = diagnostic.redacted_message();
            assert!(!message.is_empty());
            assert!(!message.contains("/home/"));
            assert!(!message.contains("token"));
            assert!(!message.contains("password"));
        }
    }

    #[test]
    fn init_errors_display_without_sensitive_state() {
        let variants = [
            NativeBackdropInitError::UnsupportedPlatform,
            NativeBackdropInitError::UnsupportedSession,
            NativeBackdropInitError::InvalidSurfaceHandle,
            NativeBackdropInitError::PermissionDenied,
            NativeBackdropInitError::PermissionRequired,
            NativeBackdropInitError::BackendUnavailable(
                "x11 connection failed for /home/user/window token password".to_owned(),
            ),
        ];

        for error in variants {
            let message = error.to_string();
            assert!(!message.is_empty());
            assert!(!message.contains("password"));
            assert!(!message.contains("token"));
            assert!(!message.contains("/home/"));
            assert!(!message.contains("x11"));
        }
    }
}
