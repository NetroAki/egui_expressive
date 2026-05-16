//! App-provided backdrop snapshot rendering helpers.
//!
//! These helpers blur caller-supplied RGBA pixels. They do not capture host or
//! native framebuffers.

#[cfg(feature = "wgpu")]
use std::hash::{Hash, Hasher};

#[cfg(feature = "wgpu")]
use crate::platform::{
    load_app_owned_offscreen_backdrop_source, AppOwnedBackdropAlphaMode, AppOwnedBackdropFrameId,
    AppOwnedBackdropSurfaceId, AppOwnedOffscreenBackdropSource,
};
use crate::platform::{
    load_backdrop_snapshot_provider, BackdropCaptureError, BackdropCaptureRequest,
};
#[cfg(feature = "wgpu")]
use crate::render::{OffscreenRequest, RenderCapabilities};
use crate::render::{
    RenderBackendKind, RenderFeature, RenderIssue, RenderIssueKind, RenderQuality, RenderReport,
};

#[cfg(feature = "wgpu")]
const BACKDROP_BLUR_MAX_PIXELS: u64 = 4_096 * 4_096;

#[cfg(feature = "wgpu")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AppOwnedBackdropPhysicalRequest {
    pub rect: egui::Rect,
    pub pixels_per_point: f32,
    pub physical_min: [u32; 2],
    pub physical_size: [u32; 2],
    pub source_physical_size: [u32; 2],
    pub surface_id: AppOwnedBackdropSurfaceId,
    pub frame_id: AppOwnedBackdropFrameId,
}

#[cfg(feature = "wgpu")]
#[derive(Clone, Copy, Debug)]
struct AppOwnedBackdropSourceMetadata {
    surface_id: AppOwnedBackdropSurfaceId,
    frame_id: AppOwnedBackdropFrameId,
    pixels_per_point: f32,
    physical_size: [u32; 2],
    format: egui_wgpu::wgpu::TextureFormat,
    sample_count: u32,
    alpha_mode: AppOwnedBackdropAlphaMode,
}

#[cfg(feature = "wgpu")]
struct AppOwnedBackdropPhysicalRect {
    min: [f32; 2],
    max: [f32; 2],
    size: [u32; 2],
}

#[cfg(feature = "wgpu")]
impl From<&AppOwnedOffscreenBackdropSource> for AppOwnedBackdropSourceMetadata {
    fn from(source: &AppOwnedOffscreenBackdropSource) -> Self {
        Self {
            surface_id: source.surface_id,
            frame_id: source.frame_id,
            pixels_per_point: source.pixels_per_point,
            physical_size: source.physical_size,
            format: source.format,
            sample_count: source.sample_count,
            alpha_mode: source.alpha_mode,
        }
    }
}

/// Report whether app-provided snapshot backdrop blur is exact-eligible.
///
/// This is pure preflight: it validates context/request readiness and provider
/// presence, but it does not call the provider.
pub fn app_provided_backdrop_blur_report(
    ctx: &egui::Context,
    rect: egui::Rect,
    radius: f32,
) -> RenderReport {
    match app_provided_backdrop_blur_preflight(ctx, rect, radius) {
        Ok((_, report)) | Err(report) => report,
    }
}

/// Build an exact WGPU callback shape for app-provided snapshot backdrop blur.
///
/// Returns `None` with a non-exact report when the provider is missing, invalid,
/// or when exact WGPU callback requirements are not met.
pub fn app_provided_backdrop_blur_shape(
    ui: &egui::Ui,
    rect: egui::Rect,
    radius: f32,
) -> (Option<egui::Shape>, RenderReport) {
    let (request, exact_report) = match app_provided_backdrop_blur_preflight(ui.ctx(), rect, radius)
    {
        Ok(preflight) => preflight,
        Err(report) => return (None, report),
    };
    let Some(provider) = load_backdrop_snapshot_provider(ui.ctx()) else {
        return (
            None,
            report_with_issue(
                backdrop_backend(),
                RenderIssueKind::ApproximateFallback,
                RenderQuality::Approximate,
                "no app-provided snapshot provider is installed; use bounded overlay fallback",
            ),
        );
    };
    let snapshot = match provider.capture_backdrop_snapshot(&request) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            return (
                None,
                backdrop_capture_error_report(backdrop_backend(), error),
            );
        }
    };
    if let Err(error) = snapshot.validate_for_request(&request) {
        return (
            None,
            backdrop_capture_error_report(backdrop_backend(), error),
        );
    }

    #[cfg(feature = "wgpu")]
    {
        let callback = egui_wgpu::Callback::new_paint_callback(
            rect,
            crate::gpu::GpuSourceLayerEffectCallback::new_blur(
                backdrop_blur_callback_id(rect, &request, radius),
                [request.requested_width, request.requested_height],
                snapshot.pixels,
                radius,
            ),
        );
        (Some(egui::Shape::Callback(callback)), exact_report)
    }
    #[cfg(not(feature = "wgpu"))]
    {
        let _ = (rect, radius, snapshot, exact_report);
        (
            None,
            report_with_issue(
                RenderBackendKind::EguiPainter,
                RenderIssueKind::MissingBackend,
                RenderQuality::Approximate,
                "wgpu feature disabled; exact snapshot backdrop blur unavailable",
            ),
        )
    }
}

/// Report whether app-owned WGPU offscreen backdrop blur is exact-eligible.
///
/// Exactness requires a same-context B2 source and, in B3, a renderer-bound
/// source binding created with the active egui-wgpu renderer. This helper never
/// captures native or host framebuffers.
#[cfg(feature = "wgpu")]
pub fn app_owned_offscreen_backdrop_blur_report(
    ctx: &egui::Context,
    rect: egui::Rect,
    radius: f32,
    surface_id: AppOwnedBackdropSurfaceId,
    frame_id: AppOwnedBackdropFrameId,
) -> RenderReport {
    match app_owned_offscreen_backdrop_blur_preflight(ctx, surface_id, frame_id, rect, radius) {
        Ok((_, report)) | Err(report) => report,
    }
}

/// Build a WGPU callback shape for app-owned offscreen backdrop blur.
///
/// B3 returns a shape only after source metadata and renderer-bound sidecar
/// validation both report exact. Stage 1 keeps this fail-closed until the
/// renderer-bound sidecar exists.
#[cfg(feature = "wgpu")]
pub fn app_owned_offscreen_backdrop_blur_shape(
    ui: &egui::Ui,
    rect: egui::Rect,
    radius: f32,
    surface_id: AppOwnedBackdropSurfaceId,
    frame_id: AppOwnedBackdropFrameId,
) -> (Option<egui::Shape>, RenderReport) {
    let (request, exact_report) = match app_owned_offscreen_backdrop_blur_preflight(
        ui.ctx(),
        surface_id,
        frame_id,
        rect,
        radius,
    ) {
        Ok(preflight) => preflight,
        Err(report) => return (None, report),
    };
    let Some(bound_source) = crate::gpu::load_bound_app_owned_offscreen_backdrop_source(ui.ctx())
    else {
        return (
            None,
            app_owned_approximate_report(
                "exact app-owned offscreen backdrop blur requires renderer-bound source binding",
            ),
        );
    };
    let callback = egui_wgpu::Callback::new_paint_callback(
        rect,
        crate::gpu::GpuAppOwnedOffscreenBackdropCallback::new_blur(
            app_owned_backdrop_blur_callback_id(&request, radius),
            bound_source,
            request.physical_min,
            request.physical_size,
            request.source_physical_size,
            radius,
        ),
    );
    (Some(egui::Shape::Callback(callback)), exact_report)
}

#[cfg(feature = "wgpu")]
fn app_owned_offscreen_backdrop_blur_preflight(
    ctx: &egui::Context,
    surface_id: AppOwnedBackdropSurfaceId,
    frame_id: AppOwnedBackdropFrameId,
    rect: egui::Rect,
    radius: f32,
) -> Result<(AppOwnedBackdropPhysicalRequest, RenderReport), RenderReport> {
    let _ = app_owned_offscreen_backdrop_physical_rect(rect, radius, ctx.pixels_per_point())?;

    let source = load_app_owned_offscreen_backdrop_source(ctx).ok_or_else(|| {
        report_with_issue(
            RenderBackendKind::EguiWgpuCallback,
            RenderIssueKind::ApproximateFallback,
            RenderQuality::Approximate,
            "no app-owned offscreen backdrop source is installed; use bounded overlay fallback",
        )
    })?;

    let request = app_owned_offscreen_backdrop_request_from_metadata(
        rect,
        radius,
        ctx.pixels_per_point(),
        surface_id,
        frame_id,
        AppOwnedBackdropSourceMetadata::from(source.as_ref()),
    )?;

    if !crate::gpu::gpu_effects_initialized_for_context(ctx) {
        return Err(report_with_issue(
            RenderBackendKind::EguiWgpuCallback,
            RenderIssueKind::ApproximateFallback,
            RenderQuality::Approximate,
            "exact app-owned offscreen backdrop blur requires init_gpu_effects_for_context(...) on this context",
        ));
    }

    let Some(bound_source) = crate::gpu::load_bound_app_owned_offscreen_backdrop_source(ctx) else {
        return Err(app_owned_approximate_report(
            "exact app-owned offscreen backdrop blur requires renderer-bound source binding",
        ));
    };
    if !bound_source.matches_request(
        &source,
        request.surface_id,
        request.frame_id,
        request.pixels_per_point,
        request.source_physical_size,
    ) {
        return Err(app_owned_approximate_report(
            "renderer-bound app-owned backdrop source does not match this request",
        ));
    }

    let capabilities = RenderCapabilities::egui_wgpu_callback(BACKDROP_BLUR_MAX_PIXELS);
    let offscreen_request = OffscreenRequest {
        feature: RenderFeature::BackdropBlur,
        width: request.physical_size[0],
        height: request.physical_size[1],
        requested_quality: RenderQuality::Exact,
    };
    let report = crate::gpu::bound_app_owned_offscreen_backdrop_effect_report(
        &capabilities,
        offscreen_request,
    );
    if report.is_exact() {
        Ok((request, report))
    } else {
        Err(report)
    }
}

#[cfg(feature = "wgpu")]
fn app_owned_offscreen_backdrop_request_from_metadata(
    rect: egui::Rect,
    radius: f32,
    context_pixels_per_point: f32,
    surface_id: AppOwnedBackdropSurfaceId,
    frame_id: AppOwnedBackdropFrameId,
    metadata: AppOwnedBackdropSourceMetadata,
) -> Result<AppOwnedBackdropPhysicalRequest, RenderReport> {
    let physical_rect =
        app_owned_offscreen_backdrop_physical_rect(rect, radius, context_pixels_per_point)?;

    if metadata.surface_id != surface_id {
        return Err(app_owned_approximate_report(
            "app-owned backdrop source surface token does not match the request",
        ));
    }
    if metadata.frame_id != frame_id {
        return Err(app_owned_approximate_report(
            "app-owned backdrop source frame token does not match the request",
        ));
    }
    if !metadata.pixels_per_point.is_finite()
        || metadata.pixels_per_point <= 0.0
        || metadata.pixels_per_point.to_bits() != context_pixels_per_point.to_bits()
    {
        return Err(app_owned_approximate_report(
            "app-owned backdrop source scale does not match the egui context",
        ));
    }
    if metadata.format != egui_wgpu::wgpu::TextureFormat::Rgba8UnormSrgb {
        return Err(app_owned_approximate_report(
            "app-owned backdrop source format is outside the B3 contract",
        ));
    }
    if metadata.sample_count != 1 {
        return Err(app_owned_approximate_report(
            "app-owned backdrop source must be single-sampled for B3",
        ));
    }
    if metadata.alpha_mode != AppOwnedBackdropAlphaMode::Straight {
        return Err(app_owned_approximate_report(
            "app-owned backdrop source alpha mode is outside the B3 contract",
        ));
    }
    if metadata.physical_size[0] == 0
        || metadata.physical_size[1] == 0
        || metadata.physical_size[0] > 4_096
        || metadata.physical_size[1] > 4_096
    {
        return Err(app_owned_approximate_report(
            "app-owned backdrop source extent is outside the B3 contract",
        ));
    }
    if physical_rect.min[0] < 0.0
        || physical_rect.min[1] < 0.0
        || physical_rect.max[0] > metadata.physical_size[0] as f32
        || physical_rect.max[1] > metadata.physical_size[1] as f32
    {
        return Err(app_owned_approximate_report(
            "requested app-owned backdrop rect is outside the source extent",
        ));
    }

    Ok(AppOwnedBackdropPhysicalRequest {
        rect,
        pixels_per_point: context_pixels_per_point,
        physical_min: [physical_rect.min[0] as u32, physical_rect.min[1] as u32],
        physical_size: physical_rect.size,
        source_physical_size: metadata.physical_size,
        surface_id,
        frame_id,
    })
}

#[cfg(feature = "wgpu")]
fn app_owned_offscreen_backdrop_physical_rect(
    rect: egui::Rect,
    radius: f32,
    pixels_per_point: f32,
) -> Result<AppOwnedBackdropPhysicalRect, RenderReport> {
    if !radius.is_finite() || radius < 1.0 {
        return Err(app_owned_approximate_report(
            "exact app-owned offscreen backdrop blur requires radius >= 1.0",
        ));
    }
    if !rect.min.x.is_finite()
        || !rect.min.y.is_finite()
        || !rect.max.x.is_finite()
        || !rect.max.y.is_finite()
        || rect.width() <= 0.0
        || rect.height() <= 0.0
        || !pixels_per_point.is_finite()
        || pixels_per_point <= 0.0
    {
        return Err(report_with_issue(
            RenderBackendKind::EguiWgpuCallback,
            RenderIssueKind::InvalidBounds,
            RenderQuality::Unsupported,
            "invalid app-owned backdrop rect or scale; exact path skipped",
        ));
    }

    let physical_min = [
        (rect.min.x * pixels_per_point).floor(),
        (rect.min.y * pixels_per_point).floor(),
    ];
    let physical_max = [
        (rect.max.x * pixels_per_point).ceil(),
        (rect.max.y * pixels_per_point).ceil(),
    ];
    if !physical_min[0].is_finite()
        || !physical_min[1].is_finite()
        || !physical_max[0].is_finite()
        || !physical_max[1].is_finite()
    {
        return Err(report_with_issue(
            RenderBackendKind::EguiWgpuCallback,
            RenderIssueKind::InvalidBounds,
            RenderQuality::Unsupported,
            "invalid app-owned backdrop physical rect; exact path skipped",
        ));
    }
    let width = physical_max[0] - physical_min[0];
    let height = physical_max[1] - physical_min[1];
    if width <= 0.0 || height <= 0.0 || width > 4_096.0 || height > 4_096.0 {
        return Err(report_with_issue(
            RenderBackendKind::EguiWgpuCallback,
            RenderIssueKind::SizeBudgetExceeded,
            RenderQuality::Unsupported,
            "app-owned offscreen backdrop blur exceeds the approved source-layer budget",
        ));
    }

    Ok(AppOwnedBackdropPhysicalRect {
        min: physical_min,
        max: physical_max,
        size: [width as u32, height as u32],
    })
}

#[cfg(feature = "wgpu")]
fn app_owned_approximate_report(message: impl Into<String>) -> RenderReport {
    report_with_issue(
        RenderBackendKind::EguiWgpuCallback,
        RenderIssueKind::ApproximateFallback,
        RenderQuality::Approximate,
        message,
    )
}

fn app_provided_backdrop_blur_preflight(
    ctx: &egui::Context,
    rect: egui::Rect,
    radius: f32,
) -> Result<(BackdropCaptureRequest, RenderReport), RenderReport> {
    if !radius.is_finite() || radius < 1.0 {
        return Err(report_with_issue(
            backdrop_backend(),
            RenderIssueKind::ApproximateFallback,
            RenderQuality::Approximate,
            "exact app-provided snapshot backdrop blur requires radius >= 1.0",
        ));
    }

    let request = BackdropCaptureRequest::new(rect, ctx.pixels_per_point()).map_err(|error| {
        backdrop_capture_request_error_report(backdrop_backend(), radius, error)
    })?;

    #[cfg(not(feature = "wgpu"))]
    {
        let _ = request;
        return Err(report_with_issue(
            RenderBackendKind::EguiPainter,
            RenderIssueKind::MissingBackend,
            RenderQuality::Approximate,
            "wgpu feature disabled; exact snapshot backdrop blur unavailable",
        ));
    }

    #[cfg(feature = "wgpu")]
    {
        if load_backdrop_snapshot_provider(ctx).is_none() {
            return Err(report_with_issue(
                RenderBackendKind::EguiWgpuCallback,
                RenderIssueKind::ApproximateFallback,
                RenderQuality::Approximate,
                "no app-provided snapshot provider is installed; use bounded overlay fallback",
            ));
        }
        if !crate::gpu::gpu_effects_initialized_for_context(ctx) {
            return Err(report_with_issue(
                RenderBackendKind::EguiWgpuCallback,
                RenderIssueKind::ApproximateFallback,
                RenderQuality::Approximate,
                "exact snapshot backdrop blur requires init_gpu_effects_for_context(...) on this context",
            ));
        }
        let capabilities = RenderCapabilities::egui_wgpu_callback(BACKDROP_BLUR_MAX_PIXELS);
        let offscreen_request = OffscreenRequest {
            feature: RenderFeature::BackdropBlur,
            width: request.requested_width,
            height: request.requested_height,
            requested_quality: RenderQuality::Exact,
        };
        let report = crate::gpu::wgpu_source_layer_effect_report(
            &capabilities,
            offscreen_request,
            crate::gpu::GpuEffectSource::AppProvidedBackdropSnapshot,
        );
        if report.is_exact() {
            Ok((request, report))
        } else {
            Err(report)
        }
    }
}

fn backdrop_backend() -> RenderBackendKind {
    #[cfg(feature = "wgpu")]
    {
        RenderBackendKind::EguiWgpuCallback
    }
    #[cfg(not(feature = "wgpu"))]
    {
        RenderBackendKind::EguiPainter
    }
}

fn report_with_issue(
    backend: RenderBackendKind,
    kind: RenderIssueKind,
    actual: RenderQuality,
    message: impl Into<String>,
) -> RenderReport {
    let mut report = RenderReport::new(backend, RenderQuality::Exact);
    report.add_issue(RenderIssue::new(
        RenderFeature::BackdropBlur,
        kind,
        RenderQuality::Exact,
        actual,
        message,
    ));
    report
}

fn backdrop_capture_request_error_report(
    backend: RenderBackendKind,
    _radius: f32,
    error: BackdropCaptureError,
) -> RenderReport {
    match error {
        BackdropCaptureError::InvalidRect | BackdropCaptureError::InvalidScale => {
            report_with_issue(
                backend,
                RenderIssueKind::InvalidBounds,
                RenderQuality::Unsupported,
                "invalid rect/scale; exact snapshot backdrop path skipped",
            )
        }
        BackdropCaptureError::SizeBudgetExceeded { .. } => report_with_issue(
            backend,
            RenderIssueKind::SizeBudgetExceeded,
            RenderQuality::Unsupported,
            "app-provided snapshot backdrop blur exceeds the approved source-layer budget",
        ),
        other => backdrop_capture_error_report(backend, other),
    }
}

fn backdrop_capture_error_report(
    backend: RenderBackendKind,
    error: BackdropCaptureError,
) -> RenderReport {
    let message = match error {
        BackdropCaptureError::SnapshotSizeMismatch { .. } => {
            "provider returned a snapshot with unexpected dimensions; use bounded fallback".into()
        }
        BackdropCaptureError::InvalidPixelLength { .. } => {
            "provider returned invalid RGBA data length; use bounded fallback".into()
        }
        BackdropCaptureError::CaptureFailed(message) => {
            format!("provider failed to capture snapshot: {message}; use bounded fallback")
        }
        BackdropCaptureError::ProviderUnavailable => {
            "no app-provided snapshot provider is installed; use bounded overlay fallback".into()
        }
        BackdropCaptureError::InvalidRect | BackdropCaptureError::InvalidScale => {
            return backdrop_capture_request_error_report(backend, 0.0, error)
        }
        BackdropCaptureError::SizeBudgetExceeded { .. } => {
            return backdrop_capture_request_error_report(backend, 0.0, error)
        }
    };
    report_with_issue(
        backend,
        RenderIssueKind::ApproximateFallback,
        RenderQuality::Approximate,
        message,
    )
}

#[cfg(feature = "wgpu")]
fn backdrop_blur_callback_id(
    rect: egui::Rect,
    request: &BackdropCaptureRequest,
    radius: f32,
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    "r100-001a-app-provided-backdrop-blur".hash(&mut hasher);
    rect.min.x.to_bits().hash(&mut hasher);
    rect.min.y.to_bits().hash(&mut hasher);
    rect.max.x.to_bits().hash(&mut hasher);
    rect.max.y.to_bits().hash(&mut hasher);
    request.pixels_per_point.to_bits().hash(&mut hasher);
    request.requested_width.hash(&mut hasher);
    request.requested_height.hash(&mut hasher);
    radius.to_bits().hash(&mut hasher);
    hasher.finish()
}

#[cfg(feature = "wgpu")]
fn app_owned_backdrop_blur_callback_id(
    request: &AppOwnedBackdropPhysicalRequest,
    radius: f32,
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    "r100-001b-b3-app-owned-backdrop-blur".hash(&mut hasher);
    request.rect.min.x.to_bits().hash(&mut hasher);
    request.rect.min.y.to_bits().hash(&mut hasher);
    request.rect.max.x.to_bits().hash(&mut hasher);
    request.rect.max.y.to_bits().hash(&mut hasher);
    request.pixels_per_point.to_bits().hash(&mut hasher);
    request.physical_min.hash(&mut hasher);
    request.physical_size.hash(&mut hasher);
    request.source_physical_size.hash(&mut hasher);
    request.surface_id.hash(&mut hasher);
    request.frame_id.hash(&mut hasher);
    radius.to_bits().hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::{
        install_backdrop_snapshot_provider, BackdropSnapshot, BackdropSnapshotProvider,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[cfg(feature = "wgpu")]
    static GPU_INIT_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    struct TestProvider {
        calls: Arc<AtomicUsize>,
        mode: ProviderMode,
    }

    #[derive(Clone, Copy)]
    #[cfg_attr(not(feature = "wgpu"), allow(dead_code))]
    enum ProviderMode {
        Valid,
        CaptureFailed,
        WrongSize,
        InvalidLength,
        MalformedSnapshot,
    }

    impl BackdropSnapshotProvider for TestProvider {
        fn capture_backdrop_snapshot(
            &self,
            request: &BackdropCaptureRequest,
        ) -> Result<BackdropSnapshot, BackdropCaptureError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            match self.mode {
                ProviderMode::Valid => BackdropSnapshot::new(
                    request.requested_width,
                    request.requested_height,
                    vec![64; request.expected_len()?],
                ),
                ProviderMode::CaptureFailed => Err(BackdropCaptureError::CaptureFailed(
                    "test capture failed".into(),
                )),
                ProviderMode::WrongSize => BackdropSnapshot::new(1, 1, vec![0; 4]),
                ProviderMode::InvalidLength => Err(BackdropCaptureError::InvalidPixelLength {
                    expected: request.expected_len()?,
                    actual: 1,
                }),
                ProviderMode::MalformedSnapshot => Ok(BackdropSnapshot {
                    width: request.requested_width,
                    height: request.requested_height,
                    pixels: vec![0],
                }),
            }
        }
    }

    fn install_provider(ctx: &egui::Context, mode: ProviderMode) -> Arc<AtomicUsize> {
        let calls = Arc::new(AtomicUsize::new(0));
        install_backdrop_snapshot_provider(
            ctx,
            Arc::new(TestProvider {
                calls: calls.clone(),
                mode,
            }),
        );
        calls
    }

    fn rect() -> egui::Rect {
        egui::Rect::from_min_size(egui::pos2(2.0, 3.0), egui::vec2(16.0, 12.0))
    }

    #[test]
    fn report_helper_is_pure_preflight_and_does_not_call_provider() {
        #[cfg(feature = "wgpu")]
        let _guard = GPU_INIT_TEST_LOCK.lock().unwrap();
        let ctx = egui::Context::default();
        let calls = install_provider(&ctx, ProviderMode::Valid);
        #[cfg(feature = "wgpu")]
        crate::gpu::set_gpu_effects_initialized_for_tests(true);

        let report = app_provided_backdrop_blur_report(&ctx, rect(), 4.0);

        #[cfg(feature = "wgpu")]
        assert!(report.is_exact());
        #[cfg(not(feature = "wgpu"))]
        assert_eq!(report.issues[0].kind, RenderIssueKind::MissingBackend);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        #[cfg(feature = "wgpu")]
        crate::gpu::set_gpu_effects_initialized_for_tests(false);
    }

    #[test]
    fn missing_provider_returns_approximate_fallback() {
        let ctx = egui::Context::default();
        let report = app_provided_backdrop_blur_report(&ctx, rect(), 4.0);
        #[cfg(feature = "wgpu")]
        {
            assert_eq!(report.actual_quality, RenderQuality::Approximate);
            assert_eq!(report.issues[0].kind, RenderIssueKind::ApproximateFallback);
        }
        #[cfg(not(feature = "wgpu"))]
        assert_eq!(report.issues[0].kind, RenderIssueKind::MissingBackend);
    }

    #[test]
    fn invalid_rect_and_low_radius_do_not_call_provider() {
        let ctx = egui::Context::default();
        let calls = install_provider(&ctx, ProviderMode::Valid);
        let empty = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(0.0, 1.0));

        let invalid_rect = app_provided_backdrop_blur_report(&ctx, empty, 4.0);
        assert_eq!(invalid_rect.actual_quality, RenderQuality::Unsupported);
        assert_eq!(invalid_rect.issues[0].kind, RenderIssueKind::InvalidBounds);

        let low_radius = app_provided_backdrop_blur_report(&ctx, rect(), 0.5);
        assert_eq!(low_radius.actual_quality, RenderQuality::Approximate);
        assert_eq!(
            low_radius.issues[0].kind,
            RenderIssueKind::ApproximateFallback
        );
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn oversized_request_returns_unsupported_without_provider_call() {
        let ctx = egui::Context::default();
        let calls = install_provider(&ctx, ProviderMode::Valid);
        let huge = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(4097.0, 1.0));

        let report = app_provided_backdrop_blur_report(&ctx, huge, 4.0);

        assert_eq!(report.actual_quality, RenderQuality::Unsupported);
        assert_eq!(report.issues[0].kind, RenderIssueKind::SizeBudgetExceeded);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[cfg(feature = "wgpu")]
    fn app_owned_metadata() -> AppOwnedBackdropSourceMetadata {
        AppOwnedBackdropSourceMetadata {
            surface_id: AppOwnedBackdropSurfaceId(7),
            frame_id: AppOwnedBackdropFrameId(11),
            pixels_per_point: 2.0,
            physical_size: [64, 64],
            format: egui_wgpu::wgpu::TextureFormat::Rgba8UnormSrgb,
            sample_count: 1,
            alpha_mode: AppOwnedBackdropAlphaMode::Straight,
        }
    }

    #[cfg(feature = "wgpu")]
    #[test]
    fn app_owned_physical_request_uses_floor_min_and_ceil_max() {
        let rect = egui::Rect::from_min_size(egui::pos2(1.25, 2.5), egui::vec2(4.2, 3.1));

        let request = app_owned_offscreen_backdrop_request_from_metadata(
            rect,
            4.0,
            2.0,
            AppOwnedBackdropSurfaceId(7),
            AppOwnedBackdropFrameId(11),
            app_owned_metadata(),
        )
        .unwrap();

        assert_eq!(request.physical_min, [2, 5]);
        assert_eq!(request.physical_size, [9, 7]);
        assert_eq!(request.source_physical_size, [64, 64]);
        assert_eq!(request.surface_id, AppOwnedBackdropSurfaceId(7));
        assert_eq!(request.frame_id, AppOwnedBackdropFrameId(11));
    }

    #[cfg(feature = "wgpu")]
    #[test]
    fn app_owned_metadata_mismatch_returns_approximate_fallback() {
        let rect = egui::Rect::from_min_size(egui::pos2(2.0, 2.0), egui::vec2(8.0, 8.0));

        let wrong_surface = app_owned_offscreen_backdrop_request_from_metadata(
            rect,
            4.0,
            2.0,
            AppOwnedBackdropSurfaceId(99),
            AppOwnedBackdropFrameId(11),
            app_owned_metadata(),
        )
        .unwrap_err();
        assert_eq!(wrong_surface.actual_quality, RenderQuality::Approximate);
        assert_eq!(
            wrong_surface.issues[0].kind,
            RenderIssueKind::ApproximateFallback
        );

        let wrong_frame = app_owned_offscreen_backdrop_request_from_metadata(
            rect,
            4.0,
            2.0,
            AppOwnedBackdropSurfaceId(7),
            AppOwnedBackdropFrameId(99),
            app_owned_metadata(),
        )
        .unwrap_err();
        assert_eq!(wrong_frame.actual_quality, RenderQuality::Approximate);
        assert_eq!(
            wrong_frame.issues[0].kind,
            RenderIssueKind::ApproximateFallback
        );

        let mut wrong_scale = app_owned_metadata();
        wrong_scale.pixels_per_point = 1.0;
        let report = app_owned_offscreen_backdrop_request_from_metadata(
            rect,
            4.0,
            2.0,
            AppOwnedBackdropSurfaceId(7),
            AppOwnedBackdropFrameId(11),
            wrong_scale,
        )
        .unwrap_err();
        assert_eq!(report.issues[0].kind, RenderIssueKind::ApproximateFallback);

        let outside = egui::Rect::from_min_size(egui::pos2(40.0, 40.0), egui::vec2(8.0, 8.0));
        let report = app_owned_offscreen_backdrop_request_from_metadata(
            outside,
            4.0,
            2.0,
            AppOwnedBackdropSurfaceId(7),
            AppOwnedBackdropFrameId(11),
            app_owned_metadata(),
        )
        .unwrap_err();
        assert_eq!(report.issues[0].kind, RenderIssueKind::ApproximateFallback);
    }

    #[cfg(feature = "wgpu")]
    #[test]
    fn app_owned_valid_metadata_without_renderer_binding_has_request_only() {
        let rect = egui::Rect::from_min_size(egui::pos2(2.0, 2.0), egui::vec2(8.0, 8.0));

        let request = app_owned_offscreen_backdrop_request_from_metadata(
            rect,
            4.0,
            2.0,
            AppOwnedBackdropSurfaceId(7),
            AppOwnedBackdropFrameId(11),
            app_owned_metadata(),
        )
        .unwrap();

        assert_eq!(request.physical_min, [4, 4]);
        assert_eq!(request.physical_size, [16, 16]);
        assert_eq!(request.source_physical_size, [64, 64]);
    }

    #[cfg(feature = "wgpu")]
    #[test]
    fn app_owned_report_rejects_invalid_geometry_before_source_lookup() {
        let ctx = egui::Context::default();
        let empty = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(0.0, 1.0));
        let huge = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(4097.0, 1.0));

        let invalid_rect = app_owned_offscreen_backdrop_blur_report(
            &ctx,
            empty,
            4.0,
            AppOwnedBackdropSurfaceId(7),
            AppOwnedBackdropFrameId(11),
        );
        assert_eq!(invalid_rect.actual_quality, RenderQuality::Unsupported);
        assert_eq!(invalid_rect.issues[0].kind, RenderIssueKind::InvalidBounds);

        let low_radius = app_owned_offscreen_backdrop_blur_report(
            &ctx,
            rect(),
            0.5,
            AppOwnedBackdropSurfaceId(7),
            AppOwnedBackdropFrameId(11),
        );
        assert_eq!(low_radius.actual_quality, RenderQuality::Approximate);
        assert_eq!(
            low_radius.issues[0].kind,
            RenderIssueKind::ApproximateFallback
        );

        let oversized = app_owned_offscreen_backdrop_blur_report(
            &ctx,
            huge,
            4.0,
            AppOwnedBackdropSurfaceId(7),
            AppOwnedBackdropFrameId(11),
        );
        assert_eq!(oversized.actual_quality, RenderQuality::Unsupported);
        assert_eq!(
            oversized.issues[0].kind,
            RenderIssueKind::SizeBudgetExceeded
        );
    }

    #[cfg(feature = "wgpu")]
    #[test]
    fn app_owned_missing_source_returns_approximate_fallback() {
        let ctx = egui::Context::default();

        let report = app_owned_offscreen_backdrop_blur_report(
            &ctx,
            rect(),
            4.0,
            AppOwnedBackdropSurfaceId(7),
            AppOwnedBackdropFrameId(11),
        );

        assert_eq!(report.actual_quality, RenderQuality::Approximate);
        assert_eq!(report.issues[0].kind, RenderIssueKind::ApproximateFallback);
    }

    #[cfg(feature = "wgpu")]
    #[test]
    fn app_owned_shape_returns_no_shape_without_source_or_binding() {
        let ctx = egui::Context::default();

        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let (shape, report) = app_owned_offscreen_backdrop_blur_shape(
                    ui,
                    rect(),
                    4.0,
                    AppOwnedBackdropSurfaceId(7),
                    AppOwnedBackdropFrameId(11),
                );
                assert!(shape.is_none());
                assert_eq!(report.actual_quality, RenderQuality::Approximate);
                assert_eq!(report.issues[0].kind, RenderIssueKind::ApproximateFallback);
            });
        });
    }

    #[cfg(not(feature = "wgpu"))]
    #[test]
    fn shape_returns_missing_backend_without_wgpu() {
        let ctx = egui::Context::default();
        let calls = install_provider(&ctx, ProviderMode::Valid);
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let (shape, report) = app_provided_backdrop_blur_shape(ui, rect(), 4.0);
                assert!(shape.is_none());
                assert_eq!(report.actual_quality, RenderQuality::Approximate);
                assert_eq!(report.issues[0].kind, RenderIssueKind::MissingBackend);
            });
        });
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[cfg(feature = "wgpu")]
    #[test]
    fn unmarked_context_returns_approximate_without_provider_call() {
        let _guard = GPU_INIT_TEST_LOCK.lock().unwrap();
        crate::gpu::set_gpu_effects_initialized_for_tests(false);
        let ctx = egui::Context::default();
        let calls = install_provider(&ctx, ProviderMode::Valid);

        let report = app_provided_backdrop_blur_report(&ctx, rect(), 4.0);

        assert_eq!(report.actual_quality, RenderQuality::Approximate);
        assert_eq!(report.issues[0].kind, RenderIssueKind::ApproximateFallback);
        assert!(report.issues[0]
            .message
            .contains("init_gpu_effects_for_context"));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[cfg(feature = "wgpu")]
    #[test]
    fn valid_provider_and_ready_context_return_callback_shape() {
        let _guard = GPU_INIT_TEST_LOCK.lock().unwrap();
        crate::gpu::set_gpu_effects_initialized_for_tests(true);
        let ctx = egui::Context::default();
        let calls = Arc::new(AtomicUsize::new(0));

        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                install_backdrop_snapshot_provider(
                    ui.ctx(),
                    Arc::new(TestProvider {
                        calls: calls.clone(),
                        mode: ProviderMode::Valid,
                    }),
                );
                let (shape, report) = app_provided_backdrop_blur_shape(ui, rect(), 4.0);
                assert!(report.is_exact(), "{report:?}");
                assert!(matches!(shape, Some(egui::Shape::Callback(_))));
            });
        });

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        crate::gpu::set_gpu_effects_initialized_for_tests(false);
    }

    #[cfg(feature = "wgpu")]
    #[test]
    fn provider_failures_return_approximate_fallback() {
        let _guard = GPU_INIT_TEST_LOCK.lock().unwrap();
        crate::gpu::set_gpu_effects_initialized_for_tests(true);
        for mode in [
            ProviderMode::CaptureFailed,
            ProviderMode::WrongSize,
            ProviderMode::InvalidLength,
            ProviderMode::MalformedSnapshot,
        ] {
            let ctx = egui::Context::default();
            let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show_inside(ctx, |ui| {
                    install_provider(ui.ctx(), mode);
                    let (shape, report) = app_provided_backdrop_blur_shape(ui, rect(), 4.0);
                    assert!(shape.is_none());
                    assert_eq!(report.actual_quality, RenderQuality::Approximate);
                    assert_eq!(report.issues[0].kind, RenderIssueKind::ApproximateFallback);
                });
            });
        }
        crate::gpu::set_gpu_effects_initialized_for_tests(false);
    }
}
