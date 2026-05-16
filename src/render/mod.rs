//! Render fidelity contracts shared by egui-native and future offscreen backends.
//!
//! These types describe what a visual feature needs, what the selected backend can
//! provide, and whether a rendered path stayed exact or used a bounded fallback.
//! They intentionally do not introduce a retained UI tree or renderer runtime.

/// Backend family used to satisfy a render/effect request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBackendKind {
    /// Plain egui painter shapes and textures.
    EguiPainter,
    /// CPU raster/offscreen group presented back through egui.
    CpuOffscreen,
    /// Existing egui-wgpu callback upload path.
    EguiWgpuCallback,
    /// Future true WGPU render-target/compositor path.
    WgpuOffscreen,
}

/// Visual feature that may need capability/fallback reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderFeature {
    BlendGroup,
    PolygonClip,
    CompoundClip,
    Blur,
    BackdropBlur,
    Shadow,
    Mask,
    TextureComposite,
    TextShaping,
    CssLayout,
}

/// Requested or achieved visual fidelity for one render/effect path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderQuality {
    /// Pixel/semantic exactness for the declared supported contract.
    Exact,
    /// Deliberate, documented approximation.
    Approximate,
    /// No meaningful implementation exists for the request.
    Unsupported,
}

impl RenderQuality {
    /// Combine two quality values, returning the lower-fidelity result.
    pub fn combine(self, other: Self) -> Self {
        self.max(other)
    }
}

/// Static capabilities for a backend selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderCapabilities {
    pub backend: RenderBackendKind,
    pub exact_blend_groups: bool,
    pub exact_polygon_clips: bool,
    pub exact_compound_clips: bool,
    pub exact_backdrop_blur: bool,
    pub exact_large_blur: bool,
    pub max_offscreen_pixels: Option<u64>,
}

impl RenderCapabilities {
    /// Default egui-native capability set: fast and portable, intentionally bounded.
    pub fn egui_native() -> Self {
        Self {
            backend: RenderBackendKind::EguiPainter,
            exact_blend_groups: false,
            exact_polygon_clips: false,
            exact_compound_clips: false,
            exact_backdrop_blur: false,
            exact_large_blur: false,
            max_offscreen_pixels: None,
        }
    }

    /// Deterministic CPU offscreen capability set used by current blend/mask helpers.
    pub fn cpu_offscreen(max_offscreen_pixels: u64) -> Self {
        Self {
            backend: RenderBackendKind::CpuOffscreen,
            exact_blend_groups: true,
            exact_polygon_clips: true,
            exact_compound_clips: true,
            exact_backdrop_blur: false,
            exact_large_blur: false,
            max_offscreen_pixels: Some(max_offscreen_pixels),
        }
    }

    /// Supported egui-wgpu callback path for presenting bounded CPU-composited
    /// textures through the WGPU renderer and Phase 9A exact source-layer blur.
    /// This is not true framebuffer/backdrop capture.
    pub fn egui_wgpu_callback(max_offscreen_pixels: u64) -> Self {
        Self {
            backend: RenderBackendKind::EguiWgpuCallback,
            exact_blend_groups: true,
            exact_polygon_clips: true,
            exact_compound_clips: true,
            exact_backdrop_blur: false,
            exact_large_blur: true,
            max_offscreen_pixels: Some(max_offscreen_pixels),
        }
    }

    /// Future high-fidelity WGPU render-target capability set. Construction is a
    /// contract for planned true offscreen passes; it does not imply backdrop
    /// capture unless `exact_backdrop_blur` is true.
    pub fn wgpu_offscreen(max_offscreen_pixels: u64, exact_backdrop_blur: bool) -> Self {
        Self {
            backend: RenderBackendKind::WgpuOffscreen,
            exact_blend_groups: true,
            exact_polygon_clips: true,
            exact_compound_clips: true,
            exact_backdrop_blur,
            exact_large_blur: true,
            max_offscreen_pixels: Some(max_offscreen_pixels),
        }
    }
}

/// Bounded offscreen allocation request used for CPU/GPU group rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OffscreenRequest {
    pub feature: RenderFeature,
    pub width: u32,
    pub height: u32,
    pub requested_quality: RenderQuality,
}

impl OffscreenRequest {
    pub fn pixels(self) -> u64 {
        self.width as u64 * self.height as u64
    }

    pub fn fits(self, capabilities: &RenderCapabilities) -> bool {
        capabilities
            .max_offscreen_pixels
            .is_none_or(|max| self.pixels() <= max)
    }
}

/// Category for a render issue/fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderIssueKind {
    EmptyInput,
    InvalidBounds,
    UnsupportedShape,
    SizeBudgetExceeded,
    MissingBackend,
    ApproximateFallback,
    UnsupportedFeature,
}

/// One explicit fidelity issue discovered while satisfying a render request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderIssue {
    pub feature: RenderFeature,
    pub kind: RenderIssueKind,
    pub requested_quality: RenderQuality,
    pub actual_quality: RenderQuality,
    pub message: String,
}

impl RenderIssue {
    pub fn new(
        feature: RenderFeature,
        kind: RenderIssueKind,
        requested_quality: RenderQuality,
        actual_quality: RenderQuality,
        message: impl Into<String>,
    ) -> Self {
        Self {
            feature,
            kind,
            requested_quality,
            actual_quality,
            message: message.into(),
        }
    }
}

/// Outcome of a render/effect path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderReport {
    pub backend: RenderBackendKind,
    pub requested_quality: RenderQuality,
    pub actual_quality: RenderQuality,
    pub issues: Vec<RenderIssue>,
}

impl RenderReport {
    pub fn new(backend: RenderBackendKind, requested_quality: RenderQuality) -> Self {
        Self {
            backend,
            requested_quality,
            actual_quality: requested_quality,
            issues: Vec::new(),
        }
    }

    pub fn add_issue(&mut self, issue: RenderIssue) {
        self.actual_quality = self.actual_quality.combine(issue.actual_quality);
        self.issues.push(issue);
    }

    pub fn is_exact(&self) -> bool {
        self.actual_quality == RenderQuality::Exact && self.issues.is_empty()
    }
}

/// Documented fallback classification for a single effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectFallback {
    Exact { backend: RenderBackendKind },
    Approximate { reason: String },
    Unsupported { reason: String },
}

impl EffectFallback {
    pub fn quality(&self) -> RenderQuality {
        match self {
            Self::Exact { .. } => RenderQuality::Exact,
            Self::Approximate { .. } => RenderQuality::Approximate,
            Self::Unsupported { .. } => RenderQuality::Unsupported,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_quality_combines_to_lowest_fidelity() {
        assert_eq!(
            RenderQuality::Exact.combine(RenderQuality::Approximate),
            RenderQuality::Approximate
        );
        assert_eq!(
            RenderQuality::Approximate.combine(RenderQuality::Unsupported),
            RenderQuality::Unsupported
        );
    }

    #[test]
    fn offscreen_request_checks_pixel_budget() {
        let capabilities = RenderCapabilities::cpu_offscreen(10_000);
        let ok = OffscreenRequest {
            feature: RenderFeature::BlendGroup,
            width: 100,
            height: 100,
            requested_quality: RenderQuality::Exact,
        };
        let too_large = OffscreenRequest { width: 101, ..ok };
        assert!(ok.fits(&capabilities));
        assert!(!too_large.fits(&capabilities));
        assert!(capabilities.exact_compound_clips);
    }

    #[test]
    fn egui_wgpu_callback_capabilities_are_source_layer_blur_only() {
        let capabilities = RenderCapabilities::egui_wgpu_callback(16_384);
        assert_eq!(capabilities.backend, RenderBackendKind::EguiWgpuCallback);
        assert!(capabilities.exact_blend_groups);
        assert!(capabilities.exact_polygon_clips);
        assert!(capabilities.exact_compound_clips);
        assert!(!capabilities.exact_backdrop_blur);
        assert!(capabilities.exact_large_blur);
    }

    #[test]
    fn wgpu_offscreen_can_represent_phase5_exact_backdrop_support() {
        let capabilities = RenderCapabilities::wgpu_offscreen(65_536, true);
        assert_eq!(capabilities.backend, RenderBackendKind::WgpuOffscreen);
        assert!(capabilities.exact_blend_groups);
        assert!(capabilities.exact_polygon_clips);
        assert!(capabilities.exact_compound_clips);
        assert!(capabilities.exact_backdrop_blur);
        assert!(capabilities.exact_large_blur);
        assert_eq!(capabilities.max_offscreen_pixels, Some(65_536));
    }

    #[test]
    fn non_wgpu_backends_do_not_claim_exact_backdrop_blur() {
        for capabilities in [
            RenderCapabilities::egui_native(),
            RenderCapabilities::cpu_offscreen(65_536),
            RenderCapabilities::egui_wgpu_callback(65_536),
            RenderCapabilities::wgpu_offscreen(65_536, false),
        ] {
            assert!(
                !capabilities.exact_backdrop_blur,
                "{:?} must not claim exact backdrop blur",
                capabilities.backend
            );
        }
    }

    #[test]
    fn report_records_approximate_issue() {
        let mut report = RenderReport::new(RenderBackendKind::EguiPainter, RenderQuality::Exact);
        report.add_issue(RenderIssue::new(
            RenderFeature::BackdropBlur,
            RenderIssueKind::ApproximateFallback,
            RenderQuality::Exact,
            RenderQuality::Approximate,
            "egui-native path paints overlay instead of sampling backdrop",
        ));
        assert_eq!(report.actual_quality, RenderQuality::Approximate);
        assert!(!report.is_exact());
    }
}
