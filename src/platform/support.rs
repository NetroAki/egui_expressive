//! Cross-platform support status and smoke artifact vocabulary.
//!
//! This module is dependency-free by design. It records the evidence a host app
//! or CI/manual runner must provide before docs can make a production support
//! claim for a target family. The presence of these types is not itself support
//! evidence; a platform is supportable only when a passing artifact exists.

/// Platform family covered by the production support program.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlatformFamily {
    Linux,
    Windows,
    Macos,
    Ios,
    Android,
    Web,
}

impl PlatformFamily {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Linux => "linux",
            Self::Windows => "windows",
            Self::Macos => "macos",
            Self::Ios => "ios",
            Self::Android => "android",
            Self::Web => "web",
        }
    }
}

/// Public support label. `Supported` is valid only with a passing smoke artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlatformSupportStatus {
    /// Runtime proof exists and all required lifecycle checks passed.
    Supported,
    /// Work is planned but no passing proof exists yet.
    Planned,
    /// Local or CI proof is blocked by missing SDK/device/runner/tooling.
    Blocked,
    /// Target is intentionally outside the supported contract.
    Unsupported,
    /// Validation has not been attempted.
    NotRun,
}

impl PlatformSupportStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Planned => "planned",
            Self::Blocked => "blocked",
            Self::Unsupported => "unsupported",
            Self::NotRun => "not-run",
        }
    }
}

/// Individual runtime lifecycle check recorded by a smoke artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformLifecycleCheck {
    pub name: &'static str,
    pub passed: bool,
    pub notes: Option<String>,
}

impl PlatformLifecycleCheck {
    pub fn passed(name: &'static str) -> Self {
        Self {
            name,
            passed: true,
            notes: None,
        }
    }

    pub fn failed(name: &'static str, notes: impl Into<String>) -> Self {
        Self {
            name,
            passed: false,
            notes: Some(notes.into()),
        }
    }
}

/// Lightweight performance sample captured during runtime smoke.
#[derive(Debug, Clone, PartialEq)]
pub struct PlatformPerformanceSample {
    pub startup_ms: Option<f32>,
    pub frame_time_ms: Option<f32>,
    pub memory_mb: Option<f32>,
}

impl PlatformPerformanceSample {
    pub const fn empty() -> Self {
        Self {
            startup_ms: None,
            frame_time_ms: None,
            memory_mb: None,
        }
    }
}

/// Final result of a platform smoke attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlatformSmokeResult {
    Passed,
    Failed,
    Blocked,
    NotRun,
}

impl PlatformSmokeResult {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Blocked => "blocked",
            Self::NotRun => "not-run",
        }
    }
}

/// Dependency-free schema for simulator/device/runner proof.
#[derive(Debug, Clone, PartialEq)]
pub struct PlatformSupportArtifact {
    pub platform: PlatformFamily,
    pub status: PlatformSupportStatus,
    pub build_passed: bool,
    pub os_version: Option<String>,
    pub sdk_version: Option<String>,
    pub rust_target: String,
    pub renderer_backend: String,
    pub gpu_adapter: Option<String>,
    pub lifecycle_checks: Vec<PlatformLifecycleCheck>,
    pub performance_sample: Option<PlatformPerformanceSample>,
    pub logs_path: Option<String>,
    pub artifact_path: Option<String>,
    pub result: PlatformSmokeResult,
}

impl PlatformSupportArtifact {
    pub fn new(
        platform: PlatformFamily,
        rust_target: impl Into<String>,
        renderer_backend: impl Into<String>,
    ) -> Self {
        Self {
            platform,
            status: PlatformSupportStatus::NotRun,
            build_passed: false,
            os_version: None,
            sdk_version: None,
            rust_target: rust_target.into(),
            renderer_backend: renderer_backend.into(),
            gpu_adapter: None,
            lifecycle_checks: Vec::new(),
            performance_sample: None,
            logs_path: None,
            artifact_path: None,
            result: PlatformSmokeResult::NotRun,
        }
    }

    pub fn with_status(mut self, status: PlatformSupportStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_result(mut self, result: PlatformSmokeResult) -> Self {
        self.result = result;
        self
    }

    pub fn with_build_passed(mut self, build_passed: bool) -> Self {
        self.build_passed = build_passed;
        self
    }

    pub fn with_lifecycle_check(mut self, check: PlatformLifecycleCheck) -> Self {
        self.lifecycle_checks.push(check);
        self
    }

    pub fn lifecycle_checks_passed(&self) -> bool {
        !self.lifecycle_checks.is_empty() && self.lifecycle_checks.iter().all(|check| check.passed)
    }

    pub fn has_required_lifecycle_checks(&self) -> bool {
        required_lifecycle_check_names(self.platform)
            .iter()
            .all(|required| {
                self.lifecycle_checks
                    .iter()
                    .any(|check| check.passed && check.name == *required)
            })
    }

    pub fn has_runtime_metadata(&self) -> bool {
        !self.rust_target.trim().is_empty()
            && !self.renderer_backend.trim().is_empty()
            && self
                .os_version
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
            && self
                .gpu_adapter
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
            && (!matches!(self.platform, PlatformFamily::Ios | PlatformFamily::Android)
                || self
                    .sdk_version
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty()))
            && self
                .logs_path
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
            && self
                .artifact_path
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
    }

    /// Returns true only when this artifact is strong enough for `Supported` docs.
    pub fn is_support_evidence(&self) -> bool {
        self.status == PlatformSupportStatus::Supported
            && self.result == PlatformSmokeResult::Passed
            && self.build_passed
            && self.lifecycle_checks_passed()
            && self.has_required_lifecycle_checks()
            && self.has_runtime_metadata()
    }
}

pub fn required_lifecycle_check_names(platform: PlatformFamily) -> &'static [&'static str] {
    match platform {
        PlatformFamily::Linux | PlatformFamily::Windows | PlatformFamily::Macos => &[
            "build",
            "launch",
            "visible-rendering",
            "resize",
            "focus",
            "high-dpi",
            "renderer-lifecycle",
        ],
        PlatformFamily::Ios | PlatformFamily::Android => &[
            "build",
            "launch",
            "visible-rendering",
            "rotation",
            "pause-resume",
            "high-dpi",
            "renderer-lifecycle",
        ],
        PlatformFamily::Web => &[
            "build",
            "browser-launch",
            "visible-rendering",
            "resize",
            "high-dpi",
            "renderer-lifecycle",
        ],
    }
}

/// Initial no-claim matrix used until platform smoke artifacts are recorded.
pub fn planned_platform_support_matrix() -> [PlatformSupportArtifact; 6] {
    [
        PlatformSupportArtifact::new(
            PlatformFamily::Linux,
            "x86_64-unknown-linux-gnu",
            "eframe/wgpu",
        )
        .with_status(PlatformSupportStatus::Planned),
        PlatformSupportArtifact::new(
            PlatformFamily::Windows,
            "x86_64-pc-windows-msvc",
            "eframe/wgpu",
        )
        .with_status(PlatformSupportStatus::Planned),
        PlatformSupportArtifact::new(PlatformFamily::Macos, "aarch64-apple-darwin", "eframe/wgpu")
            .with_status(PlatformSupportStatus::Planned),
        PlatformSupportArtifact::new(
            PlatformFamily::Ios,
            "aarch64-apple-ios-sim",
            "mobile host/wgpu",
        )
        .with_status(PlatformSupportStatus::Planned),
        PlatformSupportArtifact::new(
            PlatformFamily::Android,
            "aarch64-linux-android",
            "mobile host/wgpu",
        )
        .with_status(PlatformSupportStatus::Planned),
        PlatformSupportArtifact::new(
            PlatformFamily::Web,
            "wasm32-unknown-unknown",
            "browser/wasm",
        )
        .with_status(PlatformSupportStatus::Planned),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn support_labels_are_stable() {
        assert_eq!(PlatformFamily::Linux.label(), "linux");
        assert_eq!(PlatformFamily::Windows.label(), "windows");
        assert_eq!(PlatformFamily::Macos.label(), "macos");
        assert_eq!(PlatformFamily::Ios.label(), "ios");
        assert_eq!(PlatformFamily::Android.label(), "android");
        assert_eq!(PlatformFamily::Web.label(), "web");
        assert_eq!(PlatformSupportStatus::Supported.label(), "supported");
        assert_eq!(PlatformSupportStatus::Planned.label(), "planned");
        assert_eq!(PlatformSmokeResult::Blocked.label(), "blocked");
        assert_eq!(
            required_lifecycle_check_names(PlatformFamily::Linux)[0],
            "build"
        );
        assert!(required_lifecycle_check_names(PlatformFamily::Android).contains(&"rotation"));
        assert!(required_lifecycle_check_names(PlatformFamily::Web).contains(&"browser-launch"));
    }

    #[test]
    fn planned_matrix_makes_no_support_claims() {
        let matrix = planned_platform_support_matrix();
        assert_eq!(matrix.len(), 6);
        assert!(matrix.iter().all(|artifact| {
            artifact.status == PlatformSupportStatus::Planned && !artifact.is_support_evidence()
        }));
    }

    #[test]
    fn support_evidence_requires_runtime_metadata_and_passed_lifecycle() {
        let mut artifact = PlatformSupportArtifact::new(
            PlatformFamily::Linux,
            "x86_64-unknown-linux-gnu",
            "eframe/wgpu",
        )
        .with_status(PlatformSupportStatus::Supported)
        .with_result(PlatformSmokeResult::Passed)
        .with_build_passed(true)
        .with_lifecycle_check(PlatformLifecycleCheck::passed("build"))
        .with_lifecycle_check(PlatformLifecycleCheck::passed("launch"))
        .with_lifecycle_check(PlatformLifecycleCheck::passed("visible-rendering"))
        .with_lifecycle_check(PlatformLifecycleCheck::passed("resize"))
        .with_lifecycle_check(PlatformLifecycleCheck::passed("focus"))
        .with_lifecycle_check(PlatformLifecycleCheck::passed("high-dpi"))
        .with_lifecycle_check(PlatformLifecycleCheck::passed("renderer-lifecycle"));

        assert!(!artifact.is_support_evidence());
        artifact.os_version = Some("Ubuntu 24.04".to_owned());
        artifact.gpu_adapter = Some("llvmpipe".to_owned());
        artifact.logs_path = Some("docs/platform-smoke/linux.log".to_owned());
        artifact.artifact_path = Some("docs/platform-smoke/linux.md".to_owned());
        assert!(artifact.is_support_evidence());
        artifact
            .lifecycle_checks
            .push(PlatformLifecycleCheck::failed(
                "context-recreate",
                "device lost path not exercised",
            ));
        assert!(!artifact.is_support_evidence());
    }

    #[test]
    fn mobile_support_evidence_requires_sdk_and_mobile_lifecycle() {
        let mut artifact = PlatformSupportArtifact::new(
            PlatformFamily::Ios,
            "aarch64-apple-ios-sim",
            "mobile host/wgpu",
        )
        .with_status(PlatformSupportStatus::Supported)
        .with_result(PlatformSmokeResult::Passed)
        .with_build_passed(true)
        .with_lifecycle_check(PlatformLifecycleCheck::passed("build"))
        .with_lifecycle_check(PlatformLifecycleCheck::passed("launch"))
        .with_lifecycle_check(PlatformLifecycleCheck::passed("visible-rendering"))
        .with_lifecycle_check(PlatformLifecycleCheck::passed("rotation"))
        .with_lifecycle_check(PlatformLifecycleCheck::passed("pause-resume"))
        .with_lifecycle_check(PlatformLifecycleCheck::passed("high-dpi"))
        .with_lifecycle_check(PlatformLifecycleCheck::passed("renderer-lifecycle"));

        artifact.os_version = Some("iOS Simulator 19".to_owned());
        artifact.gpu_adapter = Some("Apple Simulator GPU".to_owned());
        artifact.logs_path = Some("docs/platform-smoke/ios.log".to_owned());
        artifact.artifact_path = Some("docs/platform-smoke/ios.md".to_owned());
        assert!(!artifact.is_support_evidence());
        artifact.sdk_version = Some("Xcode 18 / iOS SDK 19".to_owned());
        assert!(artifact.is_support_evidence());
    }

    #[test]
    fn debug_formatting_contains_artifact_contract_fields() {
        let artifact = PlatformSupportArtifact::new(
            PlatformFamily::Android,
            "aarch64-linux-android",
            "mobile host/wgpu",
        )
        .with_status(PlatformSupportStatus::Blocked)
        .with_result(PlatformSmokeResult::Blocked)
        .with_lifecycle_check(PlatformLifecycleCheck::failed(
            "emulator-launch",
            "Android SDK unavailable",
        ));

        let formatted = format!("{artifact:?}");
        assert!(formatted.contains("Android"));
        assert!(formatted.contains("aarch64-linux-android"));
        assert!(formatted.contains("emulator-launch"));
        assert!(formatted.contains("Android SDK unavailable"));
    }
}
