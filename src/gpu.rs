//! Optional WGPU resources for Illustrator-parity effects.
//!
//! The CPU scene renderer preserves the full appearance stack, but true Illustrator parity for
//! blend modes, blur chains, masks, and isolated groups requires offscreen GPU passes. This module
//! exposes the initialization hook and shader resources used by callback-owned offscreen passes while
//! keeping the crate usable without WGPU by default.

use egui::PaintCallbackInfo;
use egui_wgpu::{wgpu, CallbackResources, CallbackTrait, ScreenDescriptor};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::panic::{catch_unwind, AssertUnwindSafe};
#[cfg(test)]
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::platform::{
    load_app_owned_offscreen_backdrop_source, AppOwnedBackdropAlphaMode, AppOwnedBackdropFrameId,
    AppOwnedBackdropSurfaceId, SharedAppOwnedOffscreenBackdropSource,
};
use crate::render::{
    OffscreenRequest, RenderBackendKind, RenderCapabilities, RenderFeature, RenderIssue,
    RenderIssueKind, RenderQuality, RenderReport,
};

const MAX_UPLOADED_COMPOSITES: usize = 64;
const MAX_SOURCE_LAYER_EFFECT_AXIS: u32 = 4_096;
const OFFSCREEN_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
const BLEND_NORMAL: u32 = 0;
const BLEND_MULTIPLY: u32 = 1;
const BLEND_SCREEN: u32 = 2;
const BLEND_OVERLAY: u32 = 3;
const BLEND_DARKEN: u32 = 4;
const BLEND_LIGHTEN: u32 = 5;
const BLEND_COLOR_DODGE: u32 = 6;
const BLEND_COLOR_BURN: u32 = 7;
const BLEND_HARD_LIGHT: u32 = 8;
const BLEND_SOFT_LIGHT: u32 = 9;
const BLEND_DIFFERENCE: u32 = 10;
const BLEND_EXCLUSION: u32 = 11;
const BLEND_HUE: u32 = 12;
const BLEND_SATURATION: u32 = 13;
const BLEND_COLOR: u32 = 14;
const BLEND_LUMINOSITY: u32 = 15;
const GPU_EFFECTS_CONTEXT_READY_ID: &str = "egui_expressive.gpu_effects.context_ready";
const BOUND_APP_OWNED_OFFSCREEN_BACKDROP_SOURCE_ID: &str =
    "egui_expressive.bound_app_owned_offscreen_backdrop_source";
#[cfg(test)]
static TEST_GPU_EFFECTS_INITIALIZED: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug, PartialEq)]
struct BlendUniforms {
    blend_mode: u32,
    opacity: f32,
}

impl BlendUniforms {
    fn new(blend_mode: u32, opacity: f32) -> Self {
        Self {
            blend_mode,
            opacity: opacity.clamp(0.0, 1.0),
        }
    }

    fn as_bytes(self) -> [u8; 16] {
        let words = [self.blend_mode, self.opacity.to_bits(), 0, 0];
        let mut bytes = [0u8; 16];
        for (idx, word) in words.into_iter().enumerate() {
            bytes[idx * 4..idx * 4 + 4].copy_from_slice(&word.to_ne_bytes());
        }
        bytes
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct BlurUniforms {
    radius: f32,
    direction_x: f32,
    direction_y: f32,
}

impl BlurUniforms {
    fn new(radius: f32, direction: [f32; 2]) -> Self {
        let radius = radius.max(1.0);
        let length = (direction[0] * direction[0] + direction[1] * direction[1]).sqrt();
        let (direction_x, direction_y) = if length > f32::EPSILON {
            (direction[0] / length, direction[1] / length)
        } else {
            (1.0, 0.0)
        };
        Self {
            radius,
            direction_x,
            direction_y,
        }
    }

    fn as_bytes(self) -> [u8; 16] {
        let words = [
            self.radius.to_bits(),
            self.direction_x.to_bits(),
            self.direction_y.to_bits(),
            0,
        ];
        let mut bytes = [0u8; 16];
        for (idx, word) in words.into_iter().enumerate() {
            bytes[idx * 4..idx * 4 + 4].copy_from_slice(&word.to_ne_bytes());
        }
        bytes
    }

    fn perpendicular(self) -> Self {
        Self::new(self.radius, [-self.direction_y, self.direction_x])
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct AppOwnedBackdropBlurUniforms {
    radius: f32,
    direction_x: f32,
    direction_y: f32,
    uv_origin_x: f32,
    uv_origin_y: f32,
    uv_scale_x: f32,
    uv_scale_y: f32,
}

impl AppOwnedBackdropBlurUniforms {
    fn new(
        radius: f32,
        direction: [f32; 2],
        physical_min: [u32; 2],
        physical_size: [u32; 2],
        source_physical_size: [u32; 2],
    ) -> Self {
        let blur = BlurUniforms::new(radius, direction);
        let source_width = source_physical_size[0].max(1) as f32;
        let source_height = source_physical_size[1].max(1) as f32;
        Self {
            radius: blur.radius,
            direction_x: blur.direction_x,
            direction_y: blur.direction_y,
            uv_origin_x: physical_min[0] as f32 / source_width,
            uv_origin_y: physical_min[1] as f32 / source_height,
            uv_scale_x: physical_size[0] as f32 / source_width,
            uv_scale_y: physical_size[1] as f32 / source_height,
        }
    }

    fn as_bytes(self) -> [u8; 32] {
        let words = [
            self.radius.to_bits(),
            self.direction_x.to_bits(),
            self.direction_y.to_bits(),
            0,
            self.uv_origin_x.to_bits(),
            self.uv_origin_y.to_bits(),
            self.uv_scale_x.to_bits(),
            self.uv_scale_y.to_bits(),
        ];
        let mut bytes = [0u8; 32];
        for (idx, word) in words.into_iter().enumerate() {
            bytes[idx * 4..idx * 4 + 4].copy_from_slice(&word.to_ne_bytes());
        }
        bytes
    }

    fn vertical_blur(self) -> BlurUniforms {
        BlurUniforms::new(self.radius, [-self.direction_y, self.direction_x])
    }
}

fn present_uniforms() -> BlendUniforms {
    BlendUniforms::new(BLEND_NORMAL, 1.0)
}

/// GPU resources installed into `egui_wgpu::Renderer::callback_resources`.
pub struct GpuEffectsResources {
    pub offscreen_pipeline: wgpu::RenderPipeline,
    pub blend_pipeline: wgpu::RenderPipeline,
    pub blur_pipeline: wgpu::RenderPipeline,
    app_owned_backdrop_first_pass_pipeline: wgpu::RenderPipeline,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub uniform_bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    transparent_base_bind_group: wgpu::BindGroup,
    uploaded_composites: HashMap<u64, UploadedCompositeTexture>,
    app_owned_backdrops: HashMap<u64, AppOwnedBackdropTexture>,
    frame_counter: u64,
}

struct UploadedCompositeTexture {
    source_texture: wgpu::Texture,
    source_bind_group: wgpu::BindGroup,
    _intermediate_texture: wgpu::Texture,
    intermediate_view: wgpu::TextureView,
    intermediate_bind_group: wgpu::BindGroup,
    _offscreen_texture: wgpu::Texture,
    offscreen_view: wgpu::TextureView,
    offscreen_bind_group: wgpu::BindGroup,
    pass_uniform_buffer: wgpu::Buffer,
    pass_uniform_bind_group: wgpu::BindGroup,
    secondary_uniform_buffer: wgpu::Buffer,
    secondary_uniform_bind_group: wgpu::BindGroup,
    present_uniform_buffer: wgpu::Buffer,
    present_uniform_bind_group: wgpu::BindGroup,
    size: [u32; 2],
    last_used_frame: u64,
}

struct AppOwnedBackdropTexture {
    _intermediate_texture: wgpu::Texture,
    intermediate_view: wgpu::TextureView,
    intermediate_bind_group: wgpu::BindGroup,
    _offscreen_texture: wgpu::Texture,
    offscreen_view: wgpu::TextureView,
    offscreen_bind_group: wgpu::BindGroup,
    pass_uniform_buffer: wgpu::Buffer,
    pass_uniform_bind_group: wgpu::BindGroup,
    secondary_uniform_buffer: wgpu::Buffer,
    secondary_uniform_bind_group: wgpu::BindGroup,
    present_uniform_buffer: wgpu::Buffer,
    present_uniform_bind_group: wgpu::BindGroup,
    size: [u32; 2],
    last_used_frame: u64,
}

#[derive(Clone)]
pub(crate) struct BoundAppOwnedOffscreenBackdropSource {
    source: SharedAppOwnedOffscreenBackdropSource,
    surface_id: AppOwnedBackdropSurfaceId,
    frame_id: AppOwnedBackdropFrameId,
    pixels_per_point_bits: u32,
    physical_size: [u32; 2],
    format: wgpu::TextureFormat,
    sample_count: u32,
    alpha_mode: AppOwnedBackdropAlphaMode,
    source_bind_group: Arc<wgpu::BindGroup>,
}

pub(crate) type SharedBoundAppOwnedOffscreenBackdropSource =
    Arc<BoundAppOwnedOffscreenBackdropSource>;

/// Per-frame GPU callback that paints a pre-composited RGBA texture through the
/// same wgpu callback pipeline used by richer blend/mask passes.
pub struct GpuCompositeCallback {
    id: u64,
    size: [u32; 2],
    rgba: Vec<u8>,
    uniforms: BlendUniforms,
}

/// Per-frame GPU callback that applies the Phase 5 source-layer blur shader to a
/// library-owned RGBA texture and presents the callback-owned offscreen target.
pub struct GpuSourceLayerEffectCallback {
    id: u64,
    size: [u32; 2],
    rgba: Vec<u8>,
    uniforms: BlurUniforms,
}

/// Per-frame GPU callback that samples a renderer-bound app-owned TextureView,
/// blurs a validated subrect, and presents only the callback-owned result.
pub(crate) struct GpuAppOwnedOffscreenBackdropCallback {
    id: u64,
    source: SharedBoundAppOwnedOffscreenBackdropSource,
    physical_min: [u32; 2],
    physical_size: [u32; 2],
    source_physical_size: [u32; 2],
    uniforms: AppOwnedBackdropBlurUniforms,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuEffectSource {
    LibraryOwnedSourceLayer,
    AppProvidedBackdropSnapshot,
    AppOwnedOffscreenBackdrop,
    HostFramebufferBackdrop,
}

pub fn wgpu_source_layer_effect_report(
    capabilities: &RenderCapabilities,
    request: OffscreenRequest,
    source: GpuEffectSource,
) -> RenderReport {
    let mut report = RenderReport::new(capabilities.backend, request.requested_quality);

    if source == GpuEffectSource::HostFramebufferBackdrop {
        report.add_issue(RenderIssue::new(
            request.feature,
            RenderIssueKind::UnsupportedFeature,
            request.requested_quality,
            RenderQuality::Unsupported,
            "host framebuffer backdrop capture is outside the Phase 5 WGPU contract",
        ));
        return report;
    }

    if source == GpuEffectSource::AppOwnedOffscreenBackdrop {
        report.add_issue(RenderIssue::new(
            request.feature,
            RenderIssueKind::UnsupportedFeature,
            request.requested_quality,
            RenderQuality::Unsupported,
            "app-owned offscreen backdrop exactness requires renderer-bound B3 sidecar proof; direct generic GPU report calls remain non-exact",
        ));
        return report;
    }

    let source_layer_backend = match (request.feature, source) {
        (RenderFeature::BackdropBlur, GpuEffectSource::AppProvidedBackdropSnapshot) => {
            capabilities.backend == RenderBackendKind::EguiWgpuCallback
        }
        _ => matches!(
            capabilities.backend,
            RenderBackendKind::EguiWgpuCallback | RenderBackendKind::WgpuOffscreen
        ),
    };
    if !source_layer_backend {
        report.add_issue(RenderIssue::new(
            request.feature,
            RenderIssueKind::MissingBackend,
            request.requested_quality,
            RenderQuality::Unsupported,
            "exact source-layer effects require an approved WGPU source backend; app-provided backdrop snapshots require the egui-wgpu callback path",
        ));
        return report;
    }

    if request.width == 0
        || request.height == 0
        || request.width > MAX_SOURCE_LAYER_EFFECT_AXIS
        || request.height > MAX_SOURCE_LAYER_EFFECT_AXIS
    {
        report.add_issue(RenderIssue::new(
            request.feature,
            RenderIssueKind::SizeBudgetExceeded,
            request.requested_quality,
            RenderQuality::Unsupported,
            "source-layer effect request exceeds the per-axis 4096 pixel budget",
        ));
        return report;
    }

    if !request.fits(capabilities) {
        report.add_issue(RenderIssue::new(
            request.feature,
            RenderIssueKind::SizeBudgetExceeded,
            request.requested_quality,
            RenderQuality::Unsupported,
            "source-layer effect request exceeds the WGPU offscreen pixel budget",
        ));
        return report;
    }

    let supported = match (request.feature, source) {
        (RenderFeature::Blur | RenderFeature::Shadow, GpuEffectSource::LibraryOwnedSourceLayer) => {
            capabilities.exact_large_blur
        }
        (RenderFeature::BackdropBlur, GpuEffectSource::AppProvidedBackdropSnapshot) => {
            capabilities.exact_large_blur
        }
        _ => false,
    };
    if !supported {
        report.add_issue(RenderIssue::new(
            request.feature,
            RenderIssueKind::UnsupportedFeature,
            request.requested_quality,
            RenderQuality::Unsupported,
            "WGPU effects only support library-owned blur/shadow or app-provided snapshot backdrop within the declared contract",
        ));
    }

    report
}

pub(crate) fn bound_app_owned_offscreen_backdrop_effect_report(
    capabilities: &RenderCapabilities,
    request: OffscreenRequest,
) -> RenderReport {
    let mut report = RenderReport::new(capabilities.backend, request.requested_quality);

    if capabilities.backend != RenderBackendKind::EguiWgpuCallback {
        report.add_issue(RenderIssue::new(
            request.feature,
            RenderIssueKind::MissingBackend,
            request.requested_quality,
            RenderQuality::Unsupported,
            "renderer-bound app-owned backdrop blur requires the egui-wgpu callback backend",
        ));
        return report;
    }

    if request.width == 0
        || request.height == 0
        || request.width > MAX_SOURCE_LAYER_EFFECT_AXIS
        || request.height > MAX_SOURCE_LAYER_EFFECT_AXIS
    {
        report.add_issue(RenderIssue::new(
            request.feature,
            RenderIssueKind::SizeBudgetExceeded,
            request.requested_quality,
            RenderQuality::Unsupported,
            "app-owned backdrop blur request exceeds the per-axis 4096 pixel budget",
        ));
        return report;
    }

    if !request.fits(capabilities) {
        report.add_issue(RenderIssue::new(
            request.feature,
            RenderIssueKind::SizeBudgetExceeded,
            request.requested_quality,
            RenderQuality::Unsupported,
            "app-owned backdrop blur request exceeds the WGPU offscreen pixel budget",
        ));
        return report;
    }

    if request.feature != RenderFeature::BackdropBlur || !capabilities.exact_large_blur {
        report.add_issue(RenderIssue::new(
            request.feature,
            RenderIssueKind::UnsupportedFeature,
            request.requested_quality,
            RenderQuality::Unsupported,
            "renderer-bound app-owned backdrop blur requires exact large-blur callback capability",
        ));
    }

    report
}

pub fn bind_app_owned_offscreen_backdrop_source_for_context(
    render_state: &egui_wgpu::RenderState,
    ctx: &egui::Context,
    surface_id: AppOwnedBackdropSurfaceId,
    frame_id: AppOwnedBackdropFrameId,
) -> RenderReport {
    let failure = |ctx: &egui::Context, message: &'static str| {
        clear_bound_app_owned_offscreen_backdrop_source(ctx);
        app_owned_binding_fallback_report(message)
    };

    let Some(source) = load_app_owned_offscreen_backdrop_source(ctx) else {
        return failure(
            ctx,
            "no app-owned offscreen backdrop source is installed; renderer-bound binding skipped",
        );
    };
    if source.surface_id != surface_id {
        return failure(
            ctx,
            "app-owned backdrop source surface token does not match the binding request",
        );
    }
    if source.frame_id != frame_id {
        return failure(
            ctx,
            "app-owned backdrop source frame token does not match the binding request",
        );
    }
    if !source.pixels_per_point.is_finite()
        || source.pixels_per_point <= 0.0
        || source.pixels_per_point.to_bits() != ctx.pixels_per_point().to_bits()
    {
        return failure(
            ctx,
            "app-owned backdrop source scale does not match the egui context",
        );
    }
    if source.format != OFFSCREEN_TEXTURE_FORMAT {
        return failure(
            ctx,
            "app-owned backdrop source format is outside the B3 contract",
        );
    }
    if source.sample_count != 1 {
        return failure(
            ctx,
            "app-owned backdrop source must be single-sampled for B3",
        );
    }
    if source.alpha_mode != AppOwnedBackdropAlphaMode::Straight {
        return failure(
            ctx,
            "app-owned backdrop source alpha mode is outside the B3 contract",
        );
    }
    if source.physical_size[0] == 0
        || source.physical_size[1] == 0
        || source.physical_size[0] > MAX_SOURCE_LAYER_EFFECT_AXIS
        || source.physical_size[1] > MAX_SOURCE_LAYER_EFFECT_AXIS
    {
        return failure(
            ctx,
            "app-owned backdrop source extent is outside the B3 contract",
        );
    }
    if !gpu_effects_initialized_for_context(ctx) {
        return failure(
            ctx,
            "exact app-owned offscreen backdrop blur requires init_gpu_effects_for_context(...) on this context",
        );
    }
    let bind_group = {
        let mut renderer = render_state.renderer.write();
        let Some(resources) = renderer.callback_resources.get_mut::<GpuEffectsResources>() else {
            return failure(
                ctx,
                "GPU effect resources are not installed for renderer-bound app-owned backdrop binding",
            );
        };
        match catch_unwind(AssertUnwindSafe(|| {
            create_texture_bind_group(
                &render_state.device,
                &resources.texture_bind_group_layout,
                &resources.sampler,
                &source.texture_view,
                "egui_expressive_app_owned_backdrop_source_bg",
            )
        })) {
            Ok(bind_group) => Arc::new(bind_group),
            Err(_) => {
                return failure(
                    ctx,
                    "renderer-bound app-owned backdrop source binding failed validation",
                );
            }
        }
    };

    let bound = Arc::new(BoundAppOwnedOffscreenBackdropSource {
        source: Arc::clone(&source),
        surface_id: source.surface_id,
        frame_id: source.frame_id,
        pixels_per_point_bits: source.pixels_per_point.to_bits(),
        physical_size: source.physical_size,
        format: source.format,
        sample_count: source.sample_count,
        alpha_mode: source.alpha_mode,
        source_bind_group: bind_group,
    });
    install_bound_app_owned_offscreen_backdrop_source(ctx, bound);

    bound_app_owned_offscreen_backdrop_effect_report(
        &RenderCapabilities::egui_wgpu_callback(
            u64::from(MAX_SOURCE_LAYER_EFFECT_AXIS) * u64::from(MAX_SOURCE_LAYER_EFFECT_AXIS),
        ),
        OffscreenRequest {
            feature: RenderFeature::BackdropBlur,
            width: source.physical_size[0],
            height: source.physical_size[1],
            requested_quality: RenderQuality::Exact,
        },
    )
}

pub(crate) fn load_bound_app_owned_offscreen_backdrop_source(
    ctx: &egui::Context,
) -> Option<SharedBoundAppOwnedOffscreenBackdropSource> {
    ctx.data(|data| {
        data.get_temp::<Option<SharedBoundAppOwnedOffscreenBackdropSource>>(egui::Id::new(
            BOUND_APP_OWNED_OFFSCREEN_BACKDROP_SOURCE_ID,
        ))
        .flatten()
    })
}

fn install_bound_app_owned_offscreen_backdrop_source(
    ctx: &egui::Context,
    source: SharedBoundAppOwnedOffscreenBackdropSource,
) {
    ctx.data_mut(|data| {
        data.insert_temp(
            egui::Id::new(BOUND_APP_OWNED_OFFSCREEN_BACKDROP_SOURCE_ID),
            Some(source),
        );
    });
}

fn clear_bound_app_owned_offscreen_backdrop_source(ctx: &egui::Context) {
    ctx.data_mut(|data| {
        data.insert_temp::<Option<SharedBoundAppOwnedOffscreenBackdropSource>>(
            egui::Id::new(BOUND_APP_OWNED_OFFSCREEN_BACKDROP_SOURCE_ID),
            None,
        );
    });
}

fn app_owned_binding_fallback_report(message: impl Into<String>) -> RenderReport {
    let mut report = RenderReport::new(RenderBackendKind::EguiWgpuCallback, RenderQuality::Exact);
    report.add_issue(RenderIssue::new(
        RenderFeature::BackdropBlur,
        RenderIssueKind::ApproximateFallback,
        RenderQuality::Exact,
        RenderQuality::Approximate,
        message,
    ));
    report
}

impl BoundAppOwnedOffscreenBackdropSource {
    pub(crate) fn matches_request(
        &self,
        current_source: &SharedAppOwnedOffscreenBackdropSource,
        surface_id: AppOwnedBackdropSurfaceId,
        frame_id: AppOwnedBackdropFrameId,
        pixels_per_point: f32,
        source_physical_size: [u32; 2],
    ) -> bool {
        same_app_owned_source_allocation(&self.source, current_source)
            && self.surface_id == surface_id
            && self.frame_id == frame_id
            && self.pixels_per_point_bits == pixels_per_point.to_bits()
            && self.physical_size == source_physical_size
            && self.format == OFFSCREEN_TEXTURE_FORMAT
            && self.sample_count == 1
            && self.alpha_mode == AppOwnedBackdropAlphaMode::Straight
    }
}

fn same_app_owned_source_allocation<T>(bound_source: &Arc<T>, current_source: &Arc<T>) -> bool {
    Arc::ptr_eq(bound_source, current_source)
}

impl GpuSourceLayerEffectCallback {
    pub fn new_blur(id: u64, size: [u32; 2], rgba: Vec<u8>, radius: f32) -> Self {
        Self::new_blur_with_direction(id, size, rgba, radius, [1.0, 0.0])
    }

    pub fn new_blur_with_direction(
        id: u64,
        size: [u32; 2],
        rgba: Vec<u8>,
        radius: f32,
        direction: [f32; 2],
    ) -> Self {
        Self {
            id,
            size,
            rgba,
            uniforms: BlurUniforms::new(radius, direction),
        }
    }

    fn cache_id(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        "phase5-source-layer-blur".hash(&mut hasher);
        self.id.hash(&mut hasher);
        self.uniforms.radius.to_bits().hash(&mut hasher);
        self.uniforms.direction_x.to_bits().hash(&mut hasher);
        self.uniforms.direction_y.to_bits().hash(&mut hasher);
        hasher.finish()
    }

    pub fn radius(&self) -> f32 {
        self.uniforms.radius
    }

    pub fn direction(&self) -> [f32; 2] {
        [self.uniforms.direction_x, self.uniforms.direction_y]
    }
}

impl GpuAppOwnedOffscreenBackdropCallback {
    pub(crate) fn new_blur(
        id: u64,
        source: SharedBoundAppOwnedOffscreenBackdropSource,
        physical_min: [u32; 2],
        physical_size: [u32; 2],
        source_physical_size: [u32; 2],
        radius: f32,
    ) -> Self {
        Self {
            id,
            source,
            physical_min,
            physical_size,
            source_physical_size,
            uniforms: AppOwnedBackdropBlurUniforms::new(
                radius,
                [1.0, 0.0],
                physical_min,
                physical_size,
                source_physical_size,
            ),
        }
    }

    fn cache_id(&self) -> u64 {
        app_owned_backdrop_callback_cache_id(AppOwnedBackdropCallbackCacheKey {
            id: self.id,
            surface_id: self.source.surface_id,
            frame_id: self.source.frame_id,
            pixels_per_point_bits: self.source.pixels_per_point_bits,
            physical_min: self.physical_min,
            physical_size: self.physical_size,
            source_physical_size: self.source_physical_size,
            uniforms: self.uniforms,
        })
    }
}

#[derive(Clone, Copy)]
struct AppOwnedBackdropCallbackCacheKey {
    id: u64,
    surface_id: AppOwnedBackdropSurfaceId,
    frame_id: AppOwnedBackdropFrameId,
    pixels_per_point_bits: u32,
    physical_min: [u32; 2],
    physical_size: [u32; 2],
    source_physical_size: [u32; 2],
    uniforms: AppOwnedBackdropBlurUniforms,
}

fn app_owned_backdrop_callback_cache_id(key: AppOwnedBackdropCallbackCacheKey) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    "r100-001b-b3-app-owned-backdrop-blur".hash(&mut hasher);
    key.id.hash(&mut hasher);
    key.surface_id.hash(&mut hasher);
    key.frame_id.hash(&mut hasher);
    key.pixels_per_point_bits.hash(&mut hasher);
    key.physical_min.hash(&mut hasher);
    key.physical_size.hash(&mut hasher);
    key.source_physical_size.hash(&mut hasher);
    key.uniforms.radius.to_bits().hash(&mut hasher);
    key.uniforms.uv_origin_x.to_bits().hash(&mut hasher);
    key.uniforms.uv_origin_y.to_bits().hash(&mut hasher);
    key.uniforms.uv_scale_x.to_bits().hash(&mut hasher);
    key.uniforms.uv_scale_y.to_bits().hash(&mut hasher);
    hasher.finish()
}

impl GpuCompositeCallback {
    /// Present a CPU-composited RGBA texture through the WGPU callback path.
    ///
    /// This defaults to NORMAL/1.0 because the current public compositor already
    /// bakes blend math into `rgba` on the CPU before the callback-owned offscreen
    /// pass and presentation pass.
    pub fn new(id: u64, size: [u32; 2], rgba: Vec<u8>) -> Self {
        Self::new_with_uniforms(id, size, rgba, BLEND_NORMAL, 1.0)
    }

    /// Construct a callback with explicit shader uniforms for future pass tests
    /// and bounded custom GPU presentations. Do not use non-NORMAL uniforms to
    /// claim exact group blending unless a real base texture/pass is provided.
    fn new_with_uniforms(
        id: u64,
        size: [u32; 2],
        rgba: Vec<u8>,
        blend_mode: u32,
        opacity: f32,
    ) -> Self {
        Self {
            id,
            size,
            rgba,
            uniforms: BlendUniforms::new(blend_mode, opacity),
        }
    }

    pub fn new_with_blend_mode(
        id: u64,
        size: [u32; 2],
        rgba: Vec<u8>,
        blend_mode: crate::codegen::BlendMode,
        opacity: f32,
    ) -> Self {
        Self::new_with_uniforms(
            id,
            size,
            rgba,
            blend_mode_to_shader_id(&blend_mode),
            opacity,
        )
    }

    fn cache_id(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.id.hash(&mut hasher);
        self.uniforms.blend_mode.hash(&mut hasher);
        self.uniforms.opacity.to_bits().hash(&mut hasher);
        hasher.finish()
    }

    pub fn shader_blend_mode(&self) -> u32 {
        self.uniforms.blend_mode
    }

    pub fn opacity(&self) -> f32 {
        self.uniforms.opacity
    }
}

pub fn blend_mode_to_shader_id(mode: &crate::codegen::BlendMode) -> u32 {
    match mode {
        crate::codegen::BlendMode::Normal => BLEND_NORMAL,
        crate::codegen::BlendMode::Multiply => BLEND_MULTIPLY,
        crate::codegen::BlendMode::Screen => BLEND_SCREEN,
        crate::codegen::BlendMode::Overlay => BLEND_OVERLAY,
        crate::codegen::BlendMode::Darken => BLEND_DARKEN,
        crate::codegen::BlendMode::Lighten => BLEND_LIGHTEN,
        crate::codegen::BlendMode::ColorDodge => BLEND_COLOR_DODGE,
        crate::codegen::BlendMode::ColorBurn => BLEND_COLOR_BURN,
        crate::codegen::BlendMode::HardLight => BLEND_HARD_LIGHT,
        crate::codegen::BlendMode::SoftLight => BLEND_SOFT_LIGHT,
        crate::codegen::BlendMode::Difference => BLEND_DIFFERENCE,
        crate::codegen::BlendMode::Exclusion => BLEND_EXCLUSION,
        crate::codegen::BlendMode::Hue => BLEND_HUE,
        crate::codegen::BlendMode::Saturation => BLEND_SATURATION,
        crate::codegen::BlendMode::Color => BLEND_COLOR,
        crate::codegen::BlendMode::Luminosity => BLEND_LUMINOSITY,
    }
}

/// Initialize GPU effect resources from an egui-wgpu render state.
///
/// Call this once from an eframe app's creation context when WGPU rendering is enabled.
/// Use [`init_gpu_effects_for_context`] when scene source-layer exact effects should be eligible
/// for a specific egui context:
///
/// ```ignore
/// if let Some(render_state) = cc.wgpu_render_state.as_ref() {
///     egui_expressive::init_gpu_effects_for_context(render_state, &cc.egui_ctx);
/// }
/// ```
pub fn init_gpu_effects(render_state: &egui_wgpu::RenderState) {
    let resources = create_gpu_effects_resources(&render_state.device, render_state.target_format);
    render_state
        .renderer
        .write()
        .callback_resources
        .insert(resources);
}

/// Initialize GPU effect resources and mark one egui context as eligible for exact scene effects.
///
/// The context marker prevents a resource initialized for one renderer from globally enabling exact
/// scene callback paths for unrelated contexts. Apps that only call [`init_gpu_effects`] can still
/// use direct GPU callbacks, but scene `GaussianBlur`/`Feather`/`DropShadow`/`OuterGlow` exact paths
/// remain disabled until their context is marked here.
pub fn init_gpu_effects_for_context(render_state: &egui_wgpu::RenderState, ctx: &egui::Context) {
    init_gpu_effects(render_state);
    mark_gpu_effects_context_ready(ctx, true);
}

fn mark_gpu_effects_context_ready(ctx: &egui::Context, ready: bool) {
    ctx.data_mut(|data| {
        data.insert_temp(egui::Id::new(GPU_EFFECTS_CONTEXT_READY_ID), ready);
    });
}

pub(crate) fn gpu_effects_initialized_for_context(ctx: &egui::Context) -> bool {
    let context_ready = ctx.data(|data| {
        data.get_temp::<bool>(egui::Id::new(GPU_EFFECTS_CONTEXT_READY_ID))
            .unwrap_or(false)
    });
    #[cfg(test)]
    {
        context_ready || TEST_GPU_EFFECTS_INITIALIZED.load(Ordering::SeqCst)
    }
    #[cfg(not(test))]
    {
        context_ready
    }
}

#[cfg(test)]
pub(crate) fn set_gpu_effects_initialized_for_tests(initialized: bool) {
    TEST_GPU_EFFECTS_INITIALIZED.store(initialized, Ordering::SeqCst);
}

fn create_gpu_effects_resources(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
) -> GpuEffectsResources {
    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("egui_expressive_blend_texture_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("egui_expressive_blend_sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });

    let transparent_base_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("egui_expressive_transparent_base_texture"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: OFFSCREEN_TEXTURE_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let transparent_base_view = transparent_base_texture.create_view(&Default::default());
    let transparent_base_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("egui_expressive_transparent_base_bg"),
        layout: &texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&transparent_base_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    let uniform_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("egui_expressive_blend_uniform_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("egui_expressive_blend_shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("draw/blend_shader.wgsl").into()),
    });
    let blur_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("egui_expressive_source_layer_blur_shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("draw/blur_shader.wgsl").into()),
    });
    let app_owned_backdrop_blur_shader =
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("egui_expressive_app_owned_backdrop_blur_shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("draw/app_owned_backdrop_blur_shader.wgsl").into(),
            ),
        });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("egui_expressive_blend_pipeline_layout"),
        bind_group_layouts: &[
            Some(&texture_bind_group_layout),
            Some(&texture_bind_group_layout),
            Some(&uniform_bind_group_layout),
        ],
        immediate_size: 0,
    });

    // The offscreen pass writes straight-alpha shader output into a private
    // texture. Fixed-function alpha blending must stay disabled here; otherwise
    // semi-transparent pixels would be premultiplied once offscreen and blended a
    // second time during presentation.
    let offscreen_pipeline = create_blend_pipeline(
        device,
        &pipeline_layout,
        &shader,
        OFFSCREEN_TEXTURE_FORMAT,
        None,
        "egui_expressive_offscreen_blend_pipeline",
    );
    // The present pass is the only place that applies fixed-function alpha
    // blending against egui's main render target.
    let blend_pipeline = create_blend_pipeline(
        device,
        &pipeline_layout,
        &shader,
        target_format,
        Some(wgpu::BlendState::ALPHA_BLENDING),
        "egui_expressive_present_blend_pipeline",
    );
    let blur_pipeline = create_blend_pipeline(
        device,
        &pipeline_layout,
        &blur_shader,
        OFFSCREEN_TEXTURE_FORMAT,
        None,
        "egui_expressive_source_layer_blur_pipeline",
    );
    let app_owned_backdrop_first_pass_pipeline = create_blend_pipeline(
        device,
        &pipeline_layout,
        &app_owned_backdrop_blur_shader,
        OFFSCREEN_TEXTURE_FORMAT,
        None,
        "egui_expressive_app_owned_backdrop_first_pass_pipeline",
    );

    GpuEffectsResources {
        offscreen_pipeline,
        blend_pipeline,
        blur_pipeline,
        app_owned_backdrop_first_pass_pipeline,
        texture_bind_group_layout,
        uniform_bind_group_layout,
        sampler,
        transparent_base_bind_group,
        uploaded_composites: HashMap::new(),
        app_owned_backdrops: HashMap::new(),
        frame_counter: 0,
    }
}

fn create_blend_pipeline(
    device: &wgpu::Device,
    pipeline_layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    target_format: wgpu::TextureFormat,
    blend: Option<wgpu::BlendState>,
    label: &'static str,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: target_format,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

fn create_texture_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    view: &wgpu::TextureView,
    label: &'static str,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

fn create_uniform_binding(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    buffer_label: &'static str,
    bind_group_label: &'static str,
) -> (wgpu::Buffer, wgpu::BindGroup) {
    create_uniform_binding_with_size(device, layout, buffer_label, bind_group_label, 16)
}

fn create_uniform_binding_with_size(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    buffer_label: &'static str,
    bind_group_label: &'static str,
    size: u64,
) -> (wgpu::Buffer, wgpu::BindGroup) {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(buffer_label),
        size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(bind_group_label),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });
    (buffer, bind_group)
}

impl CallbackTrait for GpuCompositeCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &ScreenDescriptor,
        egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let Some(resources) = callback_resources.get_mut::<GpuEffectsResources>() else {
            return Vec::new();
        };
        if self.size[0] == 0 || self.size[1] == 0 || self.rgba.is_empty() {
            return Vec::new();
        }

        resources.frame_counter += 1;
        let current_frame = resources.frame_counter;

        let cache_id = self.cache_id();

        if resources.uploaded_composites.len() > MAX_UPLOADED_COMPOSITES {
            let mut oldest_id = None;
            let mut oldest_frame = u64::MAX;
            for (id, tex) in &resources.uploaded_composites {
                if tex.last_used_frame < oldest_frame {
                    oldest_frame = tex.last_used_frame;
                    oldest_id = Some(*id);
                }
            }
            if let Some(id) = oldest_id {
                resources.uploaded_composites.remove(&id);
            }
        }

        let recreate = resources
            .uploaded_composites
            .get(&cache_id)
            .map(|uploaded| uploaded.size != self.size)
            .unwrap_or(true);

        if recreate {
            let source_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("egui_expressive_composite_source_texture"),
                size: wgpu::Extent3d {
                    width: self.size[0],
                    height: self.size[1],
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: OFFSCREEN_TEXTURE_FORMAT,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let source_view = source_texture.create_view(&Default::default());
            let source_bind_group = create_texture_bind_group(
                device,
                &resources.texture_bind_group_layout,
                &resources.sampler,
                &source_view,
                "egui_expressive_composite_source_bg",
            );
            let intermediate_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("egui_expressive_composite_intermediate_texture"),
                size: wgpu::Extent3d {
                    width: self.size[0],
                    height: self.size[1],
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: OFFSCREEN_TEXTURE_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let intermediate_view = intermediate_texture.create_view(&Default::default());
            let intermediate_bind_group = create_texture_bind_group(
                device,
                &resources.texture_bind_group_layout,
                &resources.sampler,
                &intermediate_view,
                "egui_expressive_composite_intermediate_bg",
            );
            let offscreen_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("egui_expressive_composite_offscreen_texture"),
                size: wgpu::Extent3d {
                    width: self.size[0],
                    height: self.size[1],
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: OFFSCREEN_TEXTURE_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let offscreen_view = offscreen_texture.create_view(&Default::default());
            let offscreen_bind_group = create_texture_bind_group(
                device,
                &resources.texture_bind_group_layout,
                &resources.sampler,
                &offscreen_view,
                "egui_expressive_composite_offscreen_bg",
            );
            let (pass_uniform_buffer, pass_uniform_bind_group) = create_uniform_binding(
                device,
                &resources.uniform_bind_group_layout,
                "egui_expressive_composite_pass_uniforms",
                "egui_expressive_composite_pass_uniform_bg",
            );
            let (secondary_uniform_buffer, secondary_uniform_bind_group) = create_uniform_binding(
                device,
                &resources.uniform_bind_group_layout,
                "egui_expressive_composite_secondary_uniforms",
                "egui_expressive_composite_secondary_uniform_bg",
            );
            let (present_uniform_buffer, present_uniform_bind_group) = create_uniform_binding(
                device,
                &resources.uniform_bind_group_layout,
                "egui_expressive_composite_present_uniforms",
                "egui_expressive_composite_present_uniform_bg",
            );
            resources.uploaded_composites.insert(
                cache_id,
                UploadedCompositeTexture {
                    source_texture,
                    source_bind_group,
                    _intermediate_texture: intermediate_texture,
                    intermediate_view,
                    intermediate_bind_group,
                    _offscreen_texture: offscreen_texture,
                    offscreen_view,
                    offscreen_bind_group,
                    pass_uniform_buffer,
                    pass_uniform_bind_group,
                    secondary_uniform_buffer,
                    secondary_uniform_bind_group,
                    present_uniform_buffer,
                    present_uniform_bind_group,
                    size: self.size,
                    last_used_frame: current_frame,
                },
            );
        }

        if let Some(uploaded) = resources.uploaded_composites.get_mut(&cache_id) {
            uploaded.last_used_frame = current_frame;
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &uploaded.source_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &self.rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * self.size[0]),
                    rows_per_image: Some(self.size[1]),
                },
                wgpu::Extent3d {
                    width: self.size[0],
                    height: self.size[1],
                    depth_or_array_layers: 1,
                },
            );
            queue.write_buffer(&uploaded.pass_uniform_buffer, 0, &self.uniforms.as_bytes());
            queue.write_buffer(
                &uploaded.present_uniform_buffer,
                0,
                &present_uniforms().as_bytes(),
            );
        }

        if let Some(uploaded) = resources.uploaded_composites.get(&cache_id) {
            let color_attachments = [Some(wgpu::RenderPassColorAttachment {
                view: &uploaded.offscreen_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })];
            let mut render_pass = egui_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui_expressive_composite_offscreen_pass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            render_pass.set_pipeline(&resources.offscreen_pipeline);
            render_pass.set_bind_group(0, &resources.transparent_base_bind_group, &[]);
            render_pass.set_bind_group(1, &uploaded.source_bind_group, &[]);
            render_pass.set_bind_group(2, &uploaded.pass_uniform_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        Vec::new()
    }

    fn paint(
        &self,
        _info: PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &CallbackResources,
    ) {
        let Some(resources) = callback_resources.get::<GpuEffectsResources>() else {
            return;
        };
        let Some(uploaded) = resources.uploaded_composites.get(&self.cache_id()) else {
            return;
        };
        render_pass.set_pipeline(&resources.blend_pipeline);
        render_pass.set_bind_group(0, &resources.transparent_base_bind_group, &[]);
        render_pass.set_bind_group(1, &uploaded.offscreen_bind_group, &[]);
        render_pass.set_bind_group(2, &uploaded.present_uniform_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

impl CallbackTrait for GpuSourceLayerEffectCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &ScreenDescriptor,
        egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let Some(resources) = callback_resources.get_mut::<GpuEffectsResources>() else {
            return Vec::new();
        };
        if self.size[0] == 0 || self.size[1] == 0 || self.rgba.is_empty() {
            return Vec::new();
        }

        resources.frame_counter += 1;
        let current_frame = resources.frame_counter;
        let cache_id = self.cache_id();

        if resources.uploaded_composites.len() > MAX_UPLOADED_COMPOSITES {
            let mut oldest_id = None;
            let mut oldest_frame = u64::MAX;
            for (id, tex) in &resources.uploaded_composites {
                if tex.last_used_frame < oldest_frame {
                    oldest_frame = tex.last_used_frame;
                    oldest_id = Some(*id);
                }
            }
            if let Some(id) = oldest_id {
                resources.uploaded_composites.remove(&id);
            }
        }

        let recreate = resources
            .uploaded_composites
            .get(&cache_id)
            .map(|uploaded| uploaded.size != self.size)
            .unwrap_or(true);

        if recreate {
            let source_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("egui_expressive_source_layer_blur_texture"),
                size: wgpu::Extent3d {
                    width: self.size[0],
                    height: self.size[1],
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: OFFSCREEN_TEXTURE_FORMAT,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let source_view = source_texture.create_view(&Default::default());
            let source_bind_group = create_texture_bind_group(
                device,
                &resources.texture_bind_group_layout,
                &resources.sampler,
                &source_view,
                "egui_expressive_source_layer_blur_source_bg",
            );
            let intermediate_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("egui_expressive_source_layer_blur_intermediate_texture"),
                size: wgpu::Extent3d {
                    width: self.size[0],
                    height: self.size[1],
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: OFFSCREEN_TEXTURE_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let intermediate_view = intermediate_texture.create_view(&Default::default());
            let intermediate_bind_group = create_texture_bind_group(
                device,
                &resources.texture_bind_group_layout,
                &resources.sampler,
                &intermediate_view,
                "egui_expressive_source_layer_blur_intermediate_bg",
            );
            let offscreen_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("egui_expressive_source_layer_blur_offscreen_texture"),
                size: wgpu::Extent3d {
                    width: self.size[0],
                    height: self.size[1],
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: OFFSCREEN_TEXTURE_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let offscreen_view = offscreen_texture.create_view(&Default::default());
            let offscreen_bind_group = create_texture_bind_group(
                device,
                &resources.texture_bind_group_layout,
                &resources.sampler,
                &offscreen_view,
                "egui_expressive_source_layer_blur_offscreen_bg",
            );
            let (pass_uniform_buffer, pass_uniform_bind_group) = create_uniform_binding(
                device,
                &resources.uniform_bind_group_layout,
                "egui_expressive_source_layer_blur_uniforms",
                "egui_expressive_source_layer_blur_uniform_bg",
            );
            let (secondary_uniform_buffer, secondary_uniform_bind_group) = create_uniform_binding(
                device,
                &resources.uniform_bind_group_layout,
                "egui_expressive_source_layer_blur_secondary_uniforms",
                "egui_expressive_source_layer_blur_secondary_uniform_bg",
            );
            let (present_uniform_buffer, present_uniform_bind_group) = create_uniform_binding(
                device,
                &resources.uniform_bind_group_layout,
                "egui_expressive_source_layer_blur_present_uniforms",
                "egui_expressive_source_layer_blur_present_uniform_bg",
            );
            resources.uploaded_composites.insert(
                cache_id,
                UploadedCompositeTexture {
                    source_texture,
                    source_bind_group,
                    _intermediate_texture: intermediate_texture,
                    intermediate_view,
                    intermediate_bind_group,
                    _offscreen_texture: offscreen_texture,
                    offscreen_view,
                    offscreen_bind_group,
                    pass_uniform_buffer,
                    pass_uniform_bind_group,
                    secondary_uniform_buffer,
                    secondary_uniform_bind_group,
                    present_uniform_buffer,
                    present_uniform_bind_group,
                    size: self.size,
                    last_used_frame: current_frame,
                },
            );
        }

        if let Some(uploaded) = resources.uploaded_composites.get_mut(&cache_id) {
            uploaded.last_used_frame = current_frame;
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &uploaded.source_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &self.rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * self.size[0]),
                    rows_per_image: Some(self.size[1]),
                },
                wgpu::Extent3d {
                    width: self.size[0],
                    height: self.size[1],
                    depth_or_array_layers: 1,
                },
            );
            queue.write_buffer(&uploaded.pass_uniform_buffer, 0, &self.uniforms.as_bytes());
            queue.write_buffer(
                &uploaded.present_uniform_buffer,
                0,
                &present_uniforms().as_bytes(),
            );
        }

        if let Some(uploaded) = resources.uploaded_composites.get(&cache_id) {
            queue.write_buffer(
                &uploaded.secondary_uniform_buffer,
                0,
                &self.uniforms.perpendicular().as_bytes(),
            );

            let color_attachments = [Some(wgpu::RenderPassColorAttachment {
                view: &uploaded.intermediate_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })];
            let mut render_pass = egui_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui_expressive_source_layer_blur_horizontal_pass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            render_pass.set_pipeline(&resources.blur_pipeline);
            render_pass.set_bind_group(0, &resources.transparent_base_bind_group, &[]);
            render_pass.set_bind_group(1, &uploaded.source_bind_group, &[]);
            render_pass.set_bind_group(2, &uploaded.pass_uniform_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
            drop(render_pass);

            let color_attachments = [Some(wgpu::RenderPassColorAttachment {
                view: &uploaded.offscreen_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })];
            let mut render_pass = egui_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui_expressive_source_layer_blur_vertical_pass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            render_pass.set_pipeline(&resources.blur_pipeline);
            render_pass.set_bind_group(0, &resources.transparent_base_bind_group, &[]);
            render_pass.set_bind_group(1, &uploaded.intermediate_bind_group, &[]);
            render_pass.set_bind_group(2, &uploaded.secondary_uniform_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        Vec::new()
    }

    fn paint(
        &self,
        _info: PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &CallbackResources,
    ) {
        let Some(resources) = callback_resources.get::<GpuEffectsResources>() else {
            return;
        };
        let Some(uploaded) = resources.uploaded_composites.get(&self.cache_id()) else {
            return;
        };
        render_pass.set_pipeline(&resources.blend_pipeline);
        render_pass.set_bind_group(0, &resources.transparent_base_bind_group, &[]);
        render_pass.set_bind_group(1, &uploaded.offscreen_bind_group, &[]);
        render_pass.set_bind_group(2, &uploaded.present_uniform_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

impl CallbackTrait for GpuAppOwnedOffscreenBackdropCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &ScreenDescriptor,
        egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let Some(resources) = callback_resources.get_mut::<GpuEffectsResources>() else {
            return Vec::new();
        };
        if self.physical_size[0] == 0 || self.physical_size[1] == 0 {
            return Vec::new();
        }

        resources.frame_counter += 1;
        let current_frame = resources.frame_counter;
        let cache_id = self.cache_id();

        if resources.app_owned_backdrops.len() > MAX_UPLOADED_COMPOSITES {
            let mut oldest_id = None;
            let mut oldest_frame = u64::MAX;
            for (id, tex) in &resources.app_owned_backdrops {
                if tex.last_used_frame < oldest_frame {
                    oldest_frame = tex.last_used_frame;
                    oldest_id = Some(*id);
                }
            }
            if let Some(id) = oldest_id {
                resources.app_owned_backdrops.remove(&id);
            }
        }

        let recreate = resources
            .app_owned_backdrops
            .get(&cache_id)
            .map(|uploaded| uploaded.size != self.physical_size)
            .unwrap_or(true);

        if recreate {
            let intermediate_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("egui_expressive_app_owned_backdrop_intermediate_texture"),
                size: wgpu::Extent3d {
                    width: self.physical_size[0],
                    height: self.physical_size[1],
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: OFFSCREEN_TEXTURE_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let intermediate_view = intermediate_texture.create_view(&Default::default());
            let intermediate_bind_group = create_texture_bind_group(
                device,
                &resources.texture_bind_group_layout,
                &resources.sampler,
                &intermediate_view,
                "egui_expressive_app_owned_backdrop_intermediate_bg",
            );
            let offscreen_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("egui_expressive_app_owned_backdrop_offscreen_texture"),
                size: wgpu::Extent3d {
                    width: self.physical_size[0],
                    height: self.physical_size[1],
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: OFFSCREEN_TEXTURE_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let offscreen_view = offscreen_texture.create_view(&Default::default());
            let offscreen_bind_group = create_texture_bind_group(
                device,
                &resources.texture_bind_group_layout,
                &resources.sampler,
                &offscreen_view,
                "egui_expressive_app_owned_backdrop_offscreen_bg",
            );
            let (pass_uniform_buffer, pass_uniform_bind_group) = create_uniform_binding_with_size(
                device,
                &resources.uniform_bind_group_layout,
                "egui_expressive_app_owned_backdrop_first_pass_uniforms",
                "egui_expressive_app_owned_backdrop_first_pass_uniform_bg",
                32,
            );
            let (secondary_uniform_buffer, secondary_uniform_bind_group) = create_uniform_binding(
                device,
                &resources.uniform_bind_group_layout,
                "egui_expressive_app_owned_backdrop_secondary_uniforms",
                "egui_expressive_app_owned_backdrop_secondary_uniform_bg",
            );
            let (present_uniform_buffer, present_uniform_bind_group) = create_uniform_binding(
                device,
                &resources.uniform_bind_group_layout,
                "egui_expressive_app_owned_backdrop_present_uniforms",
                "egui_expressive_app_owned_backdrop_present_uniform_bg",
            );
            resources.app_owned_backdrops.insert(
                cache_id,
                AppOwnedBackdropTexture {
                    _intermediate_texture: intermediate_texture,
                    intermediate_view,
                    intermediate_bind_group,
                    _offscreen_texture: offscreen_texture,
                    offscreen_view,
                    offscreen_bind_group,
                    pass_uniform_buffer,
                    pass_uniform_bind_group,
                    secondary_uniform_buffer,
                    secondary_uniform_bind_group,
                    present_uniform_buffer,
                    present_uniform_bind_group,
                    size: self.physical_size,
                    last_used_frame: current_frame,
                },
            );
        }

        if let Some(uploaded) = resources.app_owned_backdrops.get_mut(&cache_id) {
            uploaded.last_used_frame = current_frame;
            queue.write_buffer(&uploaded.pass_uniform_buffer, 0, &self.uniforms.as_bytes());
            queue.write_buffer(
                &uploaded.secondary_uniform_buffer,
                0,
                &self.uniforms.vertical_blur().as_bytes(),
            );
            queue.write_buffer(
                &uploaded.present_uniform_buffer,
                0,
                &present_uniforms().as_bytes(),
            );
        }

        if let Some(uploaded) = resources.app_owned_backdrops.get(&cache_id) {
            let color_attachments = [Some(wgpu::RenderPassColorAttachment {
                view: &uploaded.intermediate_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })];
            let mut render_pass = egui_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui_expressive_app_owned_backdrop_horizontal_pass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            render_pass.set_pipeline(&resources.app_owned_backdrop_first_pass_pipeline);
            render_pass.set_bind_group(0, &resources.transparent_base_bind_group, &[]);
            render_pass.set_bind_group(1, self.source.source_bind_group.as_ref(), &[]);
            render_pass.set_bind_group(2, &uploaded.pass_uniform_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
            drop(render_pass);

            let color_attachments = [Some(wgpu::RenderPassColorAttachment {
                view: &uploaded.offscreen_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })];
            let mut render_pass = egui_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui_expressive_app_owned_backdrop_vertical_pass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            render_pass.set_pipeline(&resources.blur_pipeline);
            render_pass.set_bind_group(0, &resources.transparent_base_bind_group, &[]);
            render_pass.set_bind_group(1, &uploaded.intermediate_bind_group, &[]);
            render_pass.set_bind_group(2, &uploaded.secondary_uniform_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        Vec::new()
    }

    fn paint(
        &self,
        _info: PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &CallbackResources,
    ) {
        let Some(resources) = callback_resources.get::<GpuEffectsResources>() else {
            return;
        };
        let Some(uploaded) = resources.app_owned_backdrops.get(&self.cache_id()) else {
            return;
        };
        render_pass.set_pipeline(&resources.blend_pipeline);
        render_pass.set_bind_group(0, &resources.transparent_base_bind_group, &[]);
        render_pass.set_bind_group(1, &uploaded.offscreen_bind_group, &[]);
        render_pass.set_bind_group(2, &uploaded.present_uniform_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_gpu_effects_resources() {
        // Creating a wgpu device in a unit test is environment-dependent.
        // We just verify the module compiles and the function signature is correct.
        // A full test would require an async runtime and a valid GPU adapter.
        let _ = create_gpu_effects_resources;
        let callback = GpuCompositeCallback::new(1, [1, 1], vec![0, 0, 0, 0]);
        assert_eq!(callback.size, [1, 1]);
    }

    #[test]
    fn gpu_effects_context_marker_is_context_scoped() {
        set_gpu_effects_initialized_for_tests(false);
        let ready_ctx = egui::Context::default();
        let unready_ctx = egui::Context::default();

        assert!(!gpu_effects_initialized_for_context(&ready_ctx));
        assert!(!gpu_effects_initialized_for_context(&unready_ctx));

        mark_gpu_effects_context_ready(&ready_ctx, true);

        assert!(gpu_effects_initialized_for_context(&ready_ctx));
        assert!(!gpu_effects_initialized_for_context(&unready_ctx));
    }

    #[test]
    fn test_gpu_composite_callback_creation() {
        let callback = GpuCompositeCallback::new(42, [100, 200], vec![0; 100 * 200 * 4]);
        assert_eq!(callback.size, [100, 200]);
        assert_eq!(callback.rgba.len(), 100 * 200 * 4);
        assert_eq!(callback.shader_blend_mode(), BLEND_NORMAL);
        assert_eq!(callback.opacity(), 1.0);
    }

    #[test]
    fn test_source_layer_blur_callback_creation() {
        let callback =
            GpuSourceLayerEffectCallback::new_blur(99, [16, 8], vec![0; 16 * 8 * 4], 6.0);
        assert_eq!(callback.size, [16, 8]);
        assert_eq!(callback.rgba.len(), 16 * 8 * 4);
        assert_eq!(callback.radius(), 6.0);
        assert_eq!(callback.direction(), [1.0, 0.0]);

        let vertical = GpuSourceLayerEffectCallback::new_blur_with_direction(
            99,
            [16, 8],
            vec![0; 16 * 8 * 4],
            6.0,
            [0.0, 2.0],
        );
        assert_eq!(vertical.direction(), [0.0, 1.0]);
        assert_ne!(callback.cache_id(), vertical.cache_id());
    }

    #[test]
    fn test_blend_uniforms_pack_to_wgsl_layout() {
        let uniforms = BlendUniforms::new(BLEND_MULTIPLY, 0.5);
        let bytes = uniforms.as_bytes();
        assert_eq!(
            u32::from_ne_bytes(bytes[0..4].try_into().unwrap()),
            BLEND_MULTIPLY
        );
        assert_eq!(
            f32::from_bits(u32::from_ne_bytes(bytes[4..8].try_into().unwrap())),
            0.5
        );
        assert_eq!(u32::from_ne_bytes(bytes[8..12].try_into().unwrap()), 0);
        assert_eq!(u32::from_ne_bytes(bytes[12..16].try_into().unwrap()), 0);
    }

    #[test]
    fn test_blur_uniforms_pack_to_wgsl_layout() {
        let uniforms = BlurUniforms::new(8.0, [0.0, 4.0]);
        let bytes = uniforms.as_bytes();
        assert_eq!(
            f32::from_bits(u32::from_ne_bytes(bytes[0..4].try_into().unwrap())),
            8.0
        );
        assert_eq!(
            f32::from_bits(u32::from_ne_bytes(bytes[4..8].try_into().unwrap())),
            0.0
        );
        assert_eq!(
            f32::from_bits(u32::from_ne_bytes(bytes[8..12].try_into().unwrap())),
            1.0
        );
        assert_eq!(u32::from_ne_bytes(bytes[12..16].try_into().unwrap()), 0);
    }

    #[test]
    fn test_app_owned_backdrop_uniforms_pack_uv_transform() {
        let uniforms =
            AppOwnedBackdropBlurUniforms::new(8.0, [3.0, 0.0], [16, 8], [32, 16], [128, 64]);
        let bytes = uniforms.as_bytes();
        assert_eq!(
            f32::from_bits(u32::from_ne_bytes(bytes[0..4].try_into().unwrap())),
            8.0
        );
        assert_eq!(
            f32::from_bits(u32::from_ne_bytes(bytes[4..8].try_into().unwrap())),
            1.0
        );
        assert_eq!(
            f32::from_bits(u32::from_ne_bytes(bytes[8..12].try_into().unwrap())),
            0.0
        );
        assert_eq!(u32::from_ne_bytes(bytes[12..16].try_into().unwrap()), 0);
        assert_eq!(
            f32::from_bits(u32::from_ne_bytes(bytes[16..20].try_into().unwrap())),
            0.125
        );
        assert_eq!(
            f32::from_bits(u32::from_ne_bytes(bytes[20..24].try_into().unwrap())),
            0.125
        );
        assert_eq!(
            f32::from_bits(u32::from_ne_bytes(bytes[24..28].try_into().unwrap())),
            0.25
        );
        assert_eq!(
            f32::from_bits(u32::from_ne_bytes(bytes[28..32].try_into().unwrap())),
            0.25
        );
    }

    #[test]
    fn test_blur_uniforms_perpendicular_second_pass() {
        let first = BlurUniforms::new(5.0, [3.0, 4.0]);
        let second = first.perpendicular();
        assert_eq!(second.radius, 5.0);
        assert!(
            (first.direction_x * second.direction_x + first.direction_y * second.direction_y).abs()
                < 0.0001
        );
        assert!((second.direction_x.hypot(second.direction_y) - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_blend_mode_shader_ids_match_wgsl_constants() {
        assert_eq!(
            blend_mode_to_shader_id(&crate::codegen::BlendMode::Normal),
            0
        );
        assert_eq!(
            blend_mode_to_shader_id(&crate::codegen::BlendMode::Multiply),
            1
        );
        assert_eq!(
            blend_mode_to_shader_id(&crate::codegen::BlendMode::Screen),
            2
        );
        assert_eq!(
            blend_mode_to_shader_id(&crate::codegen::BlendMode::Luminosity),
            15
        );
    }

    #[test]
    fn test_callback_cache_id_includes_uniforms() {
        let normal = GpuCompositeCallback::new(7, [1, 1], vec![255; 4]);
        let multiply = GpuCompositeCallback::new_with_blend_mode(
            7,
            [1, 1],
            vec![255; 4],
            crate::codegen::BlendMode::Multiply,
            1.0,
        );
        let faded = GpuCompositeCallback::new_with_blend_mode(
            7,
            [1, 1],
            vec![255; 4],
            crate::codegen::BlendMode::Normal,
            0.5,
        );
        assert_ne!(normal.cache_id(), multiply.cache_id());
        assert_ne!(normal.cache_id(), faded.cache_id());
    }

    #[test]
    fn test_present_uniforms_do_not_reapply_callback_blend() {
        let callback = GpuCompositeCallback::new_with_blend_mode(
            9,
            [1, 1],
            vec![255; 4],
            crate::codegen::BlendMode::Multiply,
            0.25,
        );
        assert_eq!(callback.shader_blend_mode(), BLEND_MULTIPLY);
        assert_eq!(callback.opacity(), 0.25);

        let present = present_uniforms();
        assert_eq!(present.blend_mode, BLEND_NORMAL);
        assert_eq!(present.opacity, 1.0);
    }

    #[test]
    fn source_layer_blur_report_is_exact_for_approved_wgpu_offscreen() {
        let capabilities = RenderCapabilities::wgpu_offscreen(4_096, true);
        let request = OffscreenRequest {
            feature: RenderFeature::Blur,
            width: 64,
            height: 64,
            requested_quality: RenderQuality::Exact,
        };

        let report = wgpu_source_layer_effect_report(
            &capabilities,
            request,
            GpuEffectSource::LibraryOwnedSourceLayer,
        );
        assert!(report.is_exact());
    }

    #[test]
    fn source_layer_blur_report_is_exact_for_egui_wgpu_callback() {
        let capabilities = RenderCapabilities::egui_wgpu_callback(4_096);
        let request = OffscreenRequest {
            feature: RenderFeature::Blur,
            width: 64,
            height: 64,
            requested_quality: RenderQuality::Exact,
        };

        let report = wgpu_source_layer_effect_report(
            &capabilities,
            request,
            GpuEffectSource::LibraryOwnedSourceLayer,
        );
        assert!(report.is_exact());
    }

    #[test]
    fn source_layer_shadow_report_is_exact_for_egui_wgpu_callback() {
        let capabilities = RenderCapabilities::egui_wgpu_callback(4_096);
        let request = OffscreenRequest {
            feature: RenderFeature::Shadow,
            width: 64,
            height: 64,
            requested_quality: RenderQuality::Exact,
        };

        let report = wgpu_source_layer_effect_report(
            &capabilities,
            request,
            GpuEffectSource::LibraryOwnedSourceLayer,
        );
        assert!(report.is_exact());
    }

    #[test]
    fn library_source_layer_backdrop_remains_unsupported_without_snapshot_source() {
        let capabilities = RenderCapabilities::egui_wgpu_callback(4_096);
        let request = OffscreenRequest {
            feature: RenderFeature::BackdropBlur,
            width: 64,
            height: 64,
            requested_quality: RenderQuality::Exact,
        };

        let report = wgpu_source_layer_effect_report(
            &capabilities,
            request,
            GpuEffectSource::LibraryOwnedSourceLayer,
        );
        assert_eq!(report.actual_quality, RenderQuality::Unsupported);
        assert_eq!(report.issues[0].kind, RenderIssueKind::UnsupportedFeature);
    }

    #[test]
    fn app_provided_backdrop_snapshot_report_is_source_qualified_exact() {
        let capabilities = RenderCapabilities::egui_wgpu_callback(4_096);
        assert!(
            !capabilities.exact_backdrop_blur,
            "R100-001A must not promote backend-global backdrop exactness"
        );
        let request = OffscreenRequest {
            feature: RenderFeature::BackdropBlur,
            width: 64,
            height: 64,
            requested_quality: RenderQuality::Exact,
        };

        let report = wgpu_source_layer_effect_report(
            &capabilities,
            request,
            GpuEffectSource::AppProvidedBackdropSnapshot,
        );
        assert!(report.is_exact());
    }

    #[test]
    fn app_provided_backdrop_snapshot_requires_callback_backend_not_wgpu_offscreen() {
        let capabilities = RenderCapabilities::wgpu_offscreen(4_096, true);
        let request = OffscreenRequest {
            feature: RenderFeature::BackdropBlur,
            width: 64,
            height: 64,
            requested_quality: RenderQuality::Exact,
        };

        let report = wgpu_source_layer_effect_report(
            &capabilities,
            request,
            GpuEffectSource::AppProvidedBackdropSnapshot,
        );
        assert_eq!(report.actual_quality, RenderQuality::Unsupported);
        assert_eq!(report.issues[0].kind, RenderIssueKind::MissingBackend);
        assert!(report.issues[0].message.contains("egui-wgpu callback"));
    }

    #[test]
    fn app_provided_backdrop_snapshot_requires_wgpu_source_backend() {
        let capabilities = RenderCapabilities::egui_native();
        let request = OffscreenRequest {
            feature: RenderFeature::BackdropBlur,
            width: 64,
            height: 64,
            requested_quality: RenderQuality::Exact,
        };

        let report = wgpu_source_layer_effect_report(
            &capabilities,
            request,
            GpuEffectSource::AppProvidedBackdropSnapshot,
        );
        assert_eq!(report.actual_quality, RenderQuality::Unsupported);
        assert_eq!(report.issues[0].kind, RenderIssueKind::MissingBackend);
    }

    #[test]
    fn bound_app_owned_backdrop_report_is_exact_for_callback_backend() {
        let capabilities = RenderCapabilities::egui_wgpu_callback(4_096 * 4_096);
        let request = OffscreenRequest {
            feature: RenderFeature::BackdropBlur,
            width: 64,
            height: 64,
            requested_quality: RenderQuality::Exact,
        };

        let report = bound_app_owned_offscreen_backdrop_effect_report(&capabilities, request);
        assert!(report.is_exact());
        assert!(!capabilities.exact_backdrop_blur);
    }

    #[test]
    fn bound_app_owned_backdrop_report_rejects_wgpu_offscreen_backend() {
        let capabilities = RenderCapabilities::wgpu_offscreen(4_096, true);
        let request = OffscreenRequest {
            feature: RenderFeature::BackdropBlur,
            width: 64,
            height: 64,
            requested_quality: RenderQuality::Exact,
        };

        let report = bound_app_owned_offscreen_backdrop_effect_report(&capabilities, request);
        assert_eq!(report.actual_quality, RenderQuality::Unsupported);
        assert_eq!(report.issues[0].kind, RenderIssueKind::MissingBackend);
        assert!(report.issues[0].message.contains("egui-wgpu callback"));
    }

    #[test]
    fn bound_app_owned_backdrop_report_enforces_budget() {
        let capabilities = RenderCapabilities::egui_wgpu_callback(4_096 * 4_096);
        let request = OffscreenRequest {
            feature: RenderFeature::BackdropBlur,
            width: 4_097,
            height: 1,
            requested_quality: RenderQuality::Exact,
        };

        let report = bound_app_owned_offscreen_backdrop_effect_report(&capabilities, request);
        assert_eq!(report.actual_quality, RenderQuality::Unsupported);
        assert_eq!(report.issues[0].kind, RenderIssueKind::SizeBudgetExceeded);
    }

    #[test]
    fn app_owned_backdrop_callback_cache_key_tracks_source_request_and_radius() {
        let uniforms =
            AppOwnedBackdropBlurUniforms::new(4.0, [1.0, 0.0], [8, 10], [32, 16], [128, 64]);
        let base_key = AppOwnedBackdropCallbackCacheKey {
            id: 55,
            surface_id: AppOwnedBackdropSurfaceId(7),
            frame_id: AppOwnedBackdropFrameId(11),
            pixels_per_point_bits: 2.0f32.to_bits(),
            physical_min: [8, 10],
            physical_size: [32, 16],
            source_physical_size: [128, 64],
            uniforms,
        };
        let base = app_owned_backdrop_callback_cache_id(base_key);

        assert_ne!(
            base,
            app_owned_backdrop_callback_cache_id(AppOwnedBackdropCallbackCacheKey {
                surface_id: AppOwnedBackdropSurfaceId(8),
                ..base_key
            })
        );
        assert_ne!(
            base,
            app_owned_backdrop_callback_cache_id(AppOwnedBackdropCallbackCacheKey {
                frame_id: AppOwnedBackdropFrameId(12),
                ..base_key
            })
        );
        assert_ne!(
            base,
            app_owned_backdrop_callback_cache_id(AppOwnedBackdropCallbackCacheKey {
                physical_min: [9, 10],
                ..base_key
            })
        );
        assert_ne!(
            base,
            app_owned_backdrop_callback_cache_id(AppOwnedBackdropCallbackCacheKey {
                physical_size: [33, 16],
                uniforms: AppOwnedBackdropBlurUniforms::new(
                    4.0,
                    [1.0, 0.0],
                    [8, 10],
                    [33, 16],
                    [128, 64],
                ),
                ..base_key
            })
        );
        assert_ne!(
            base,
            app_owned_backdrop_callback_cache_id(AppOwnedBackdropCallbackCacheKey {
                source_physical_size: [129, 64],
                uniforms: AppOwnedBackdropBlurUniforms::new(
                    4.0,
                    [1.0, 0.0],
                    [8, 10],
                    [32, 16],
                    [129, 64],
                ),
                ..base_key
            })
        );
        assert_ne!(
            base,
            app_owned_backdrop_callback_cache_id(AppOwnedBackdropCallbackCacheKey {
                uniforms: AppOwnedBackdropBlurUniforms::new(
                    5.0,
                    [1.0, 0.0],
                    [8, 10],
                    [32, 16],
                    [128, 64],
                ),
                ..base_key
            })
        );
    }

    #[test]
    fn app_owned_source_identity_rejects_same_metadata_reinstall() {
        let first_install = Arc::new((AppOwnedBackdropSurfaceId(7), AppOwnedBackdropFrameId(11)));
        let bound_clone = Arc::clone(&first_install);
        let same_metadata_reinstall =
            Arc::new((AppOwnedBackdropSurfaceId(7), AppOwnedBackdropFrameId(11)));

        assert!(same_app_owned_source_allocation(
            &first_install,
            &bound_clone
        ));
        assert!(!same_app_owned_source_allocation(
            &first_install,
            &same_metadata_reinstall
        ));
    }

    #[test]
    fn direct_app_owned_offscreen_backdrop_report_remains_non_exact_without_sidecar() {
        let capabilities = RenderCapabilities::egui_wgpu_callback(4_096);
        let request = OffscreenRequest {
            feature: RenderFeature::BackdropBlur,
            width: 64,
            height: 64,
            requested_quality: RenderQuality::Exact,
        };

        let report = wgpu_source_layer_effect_report(
            &capabilities,
            request,
            GpuEffectSource::AppOwnedOffscreenBackdrop,
        );
        assert_eq!(report.actual_quality, RenderQuality::Unsupported);
        assert_eq!(report.issues[0].kind, RenderIssueKind::UnsupportedFeature);
        assert!(report.issues[0].message.contains("renderer-bound"));
        assert!(report.issues[0].message.contains("non-exact"));
    }

    #[test]
    fn host_framebuffer_backdrop_report_is_unsupported() {
        let capabilities = RenderCapabilities::wgpu_offscreen(4_096, true);
        let request = OffscreenRequest {
            feature: RenderFeature::BackdropBlur,
            width: 64,
            height: 64,
            requested_quality: RenderQuality::Exact,
        };

        let report = wgpu_source_layer_effect_report(
            &capabilities,
            request,
            GpuEffectSource::HostFramebufferBackdrop,
        );
        assert_eq!(report.actual_quality, RenderQuality::Unsupported);
        assert_eq!(report.issues[0].kind, RenderIssueKind::UnsupportedFeature);
        assert!(report.issues[0].message.contains("host framebuffer"));
    }

    #[test]
    fn source_layer_effect_report_enforces_budget() {
        let capabilities = RenderCapabilities::wgpu_offscreen(4_096, true);
        let request = OffscreenRequest {
            feature: RenderFeature::Blur,
            width: 65,
            height: 64,
            requested_quality: RenderQuality::Exact,
        };

        let report = wgpu_source_layer_effect_report(
            &capabilities,
            request,
            GpuEffectSource::LibraryOwnedSourceLayer,
        );
        assert_eq!(report.actual_quality, RenderQuality::Unsupported);
        assert_eq!(report.issues[0].kind, RenderIssueKind::SizeBudgetExceeded);
    }

    #[test]
    fn source_layer_effect_report_rejects_zero_axis_before_exactness() {
        let capabilities = RenderCapabilities::egui_wgpu_callback(4_096 * 4_096);
        let request = OffscreenRequest {
            feature: RenderFeature::BackdropBlur,
            width: 0,
            height: 64,
            requested_quality: RenderQuality::Exact,
        };

        let report = wgpu_source_layer_effect_report(
            &capabilities,
            request,
            GpuEffectSource::AppProvidedBackdropSnapshot,
        );
        assert_eq!(report.actual_quality, RenderQuality::Unsupported);
        assert_eq!(report.issues[0].kind, RenderIssueKind::SizeBudgetExceeded);
        assert!(report.issues[0].message.contains("per-axis 4096"));
    }

    #[test]
    fn source_layer_effect_report_rejects_skinny_over_axis_limit() {
        let capabilities = RenderCapabilities::egui_wgpu_callback(8_192 * 8_192);
        let snapshot_request = OffscreenRequest {
            feature: RenderFeature::BackdropBlur,
            width: 8_192,
            height: 1,
            requested_quality: RenderQuality::Exact,
        };
        let snapshot_report = wgpu_source_layer_effect_report(
            &capabilities,
            snapshot_request,
            GpuEffectSource::AppProvidedBackdropSnapshot,
        );
        assert_eq!(snapshot_report.actual_quality, RenderQuality::Unsupported);
        assert_eq!(
            snapshot_report.issues[0].kind,
            RenderIssueKind::SizeBudgetExceeded
        );

        let library_request = OffscreenRequest {
            feature: RenderFeature::Blur,
            width: 1,
            height: 8_192,
            requested_quality: RenderQuality::Exact,
        };
        let library_report = wgpu_source_layer_effect_report(
            &capabilities,
            library_request,
            GpuEffectSource::LibraryOwnedSourceLayer,
        );
        assert_eq!(library_report.actual_quality, RenderQuality::Unsupported);
        assert_eq!(
            library_report.issues[0].kind,
            RenderIssueKind::SizeBudgetExceeded
        );
    }
}
