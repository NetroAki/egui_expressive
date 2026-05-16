use super::*;
use crate::render::{
    RenderBackendKind, RenderFeature, RenderIssue, RenderIssueKind, RenderQuality, RenderReport,
};

/// A layer of shapes to be composited with a specific blend mode and opacity.
pub struct BlendLayer {
    /// Shapes to render in this layer.
    pub shapes: Vec<egui::Shape>,
    /// Optional polygon masks applied to this layer before it is blended.
    pub clip_polygons: Vec<Vec<egui::Pos2>>,
    /// Blend mode for compositing this layer over the layers below it.
    pub blend_mode: crate::codegen::BlendMode,
    /// Overall opacity of this layer (0.0–1.0).
    pub opacity: f32,
}

pub(crate) type RasterizedBlendGroup = (egui::Rect, [u32; 2], Vec<egui::Color32>, Vec<egui::Shape>);

fn paint_layers_without_group_compositing(ui: &mut egui::Ui, layers: Vec<BlendLayer>) {
    for layer in layers {
        for shape in layer.shapes {
            ui.painter().add(shape);
        }
    }
}

impl BlendLayer {
    /// Create a new blend layer with Normal blend mode and full opacity.
    pub fn new(shapes: Vec<egui::Shape>) -> Self {
        Self {
            shapes,
            clip_polygons: Vec::new(),
            blend_mode: crate::codegen::BlendMode::Normal,
            opacity: 1.0,
        }
    }

    /// Set the blend mode.
    pub fn blend_mode(mut self, mode: crate::codegen::BlendMode) -> Self {
        self.blend_mode = mode;
        self
    }

    /// Set the opacity (0.0–1.0).
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Apply a polygon clip mask to this layer before compositing.
    pub fn clip_polygon(mut self, polygon: Vec<egui::Pos2>) -> Self {
        self.clip_polygons.push(polygon);
        self
    }
}

/// Composite multiple [`BlendLayer`]s bottom-to-top using per-pixel blend math.
///
/// Solid rect, circle, and filled path shapes are rasterized into layer buffers,
/// then composited with the same W3C/Illustrator-style blend equations exposed by
/// [`blend_color`]. This preserves Multiply/Screen/Overlay/etc between supplied
/// layers instead of blending against the theme background. Unsupported egui shape
/// variants are ignored by the rasterizer and should be emitted as vector shapes
/// outside the blend group by callers that need them.
pub fn composite_layers(ui: &mut egui::Ui, layers: Vec<BlendLayer>) {
    let _ = composite_layers_report(ui, layers);
}

/// Composite layers and return an explicit fidelity report.
///
/// Compatibility callers can keep using [`composite_layers`]. New exactness-sensitive
/// paths should inspect the returned [`RenderReport`] so unsupported shapes or
/// offscreen budget failures cannot silently masquerade as exact compositing.
pub fn composite_layers_report(ui: &mut egui::Ui, layers: Vec<BlendLayer>) -> RenderReport {
    let mut report = RenderReport::new(RenderBackendKind::CpuOffscreen, RenderQuality::Exact);
    if layers.is_empty() {
        report.add_issue(RenderIssue::new(
            RenderFeature::BlendGroup,
            RenderIssueKind::EmptyInput,
            RenderQuality::Exact,
            RenderQuality::Exact,
            "blend group was empty; nothing was painted",
        ));
        return report;
    }
    let (rect, size, pixels, unhandled) = match rasterize_composited_layers_result(&layers) {
        Ok(result) => result,
        Err(error) => {
            report.add_issue(error.to_render_issue(RenderFeature::BlendGroup));
            paint_layers_without_group_compositing(ui, layers);
            return report;
        }
    };

    let image = egui::ColorImage {
        size: [size[0] as usize, size[1] as usize],
        pixels,
        source_size: egui::vec2(size[0] as f32, size[1] as f32),
    };
    let texture = ui.ctx().load_texture(
        format!(
            "__egui_expressive_composite_{:x}",
            blend_layers_hash(&layers, &image.pixels)
        ),
        image,
        egui::TextureOptions::LINEAR,
    );
    ui.painter().image(
        texture.id(),
        rect,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );
    for shape in unhandled {
        ui.painter().add(shape);
    }
    report
}

/// Composite layers through an egui-wgpu paint callback when the `wgpu`
/// feature is enabled. Call [`crate::init_gpu_effects`] once during app startup
/// before using this path. Without `wgpu`, this falls back to [`composite_layers`].
#[cfg(feature = "wgpu")]
pub fn composite_layers_gpu(ui: &mut egui::Ui, layers: Vec<BlendLayer>) {
    let _ = composite_layers_gpu_report(ui, layers);
}

#[cfg(feature = "wgpu")]
pub fn composite_layers_gpu_report(ui: &mut egui::Ui, layers: Vec<BlendLayer>) -> RenderReport {
    let mut report = RenderReport::new(RenderBackendKind::EguiWgpuCallback, RenderQuality::Exact);
    if layers.is_empty() {
        report.add_issue(RenderIssue::new(
            RenderFeature::BlendGroup,
            RenderIssueKind::EmptyInput,
            RenderQuality::Exact,
            RenderQuality::Exact,
            "blend group was empty; nothing was painted",
        ));
        return report;
    }
    let (rect, size, pixels, unhandled) = match rasterize_composited_layers_result(&layers) {
        Ok(result) => result,
        Err(error) => {
            report.add_issue(error.to_render_issue(RenderFeature::BlendGroup));
            paint_layers_without_group_compositing(ui, layers);
            return report;
        }
    };
    let rgba = pixels_to_rgba(&pixels);
    let id = blend_layers_hash(&layers, &pixels);
    let callback = egui_wgpu::Callback::new_paint_callback(
        rect,
        crate::gpu::GpuCompositeCallback::new(id, size, rgba),
    );
    ui.painter().add(egui::Shape::Callback(callback));
    for shape in unhandled {
        ui.painter().add(shape);
    }
    report
}

#[cfg(not(feature = "wgpu"))]
pub fn composite_layers_gpu(ui: &mut egui::Ui, layers: Vec<BlendLayer>) {
    let _ = composite_layers_gpu_report(ui, layers);
}

#[cfg(not(feature = "wgpu"))]
pub fn composite_layers_gpu_report(ui: &mut egui::Ui, layers: Vec<BlendLayer>) -> RenderReport {
    let mut report = composite_layers_report(ui, layers);
    report.add_issue(RenderIssue::new(
        RenderFeature::BlendGroup,
        RenderIssueKind::MissingBackend,
        RenderQuality::Exact,
        report.actual_quality,
        "wgpu feature is disabled; used CPU offscreen compositing instead of egui-wgpu callback presentation",
    ));
    report
}

/// Composite layers and apply an arbitrary polygon mask before painting.
///
/// This is the vector-export friendly clipping path: supplied [`BlendLayer`]s are
/// rasterized into a single per-pixel layer group, every pixel outside
/// `clip_polygon` is made transparent, and the result is painted as one texture.
/// With the `wgpu` feature enabled it is presented through the egui-wgpu callback
/// pipeline; otherwise it uses egui's texture painter as a CPU fallback.
pub fn clipped_layers_gpu(ui: &mut egui::Ui, clip_polygon: &[egui::Pos2], layers: Vec<BlendLayer>) {
    let _ = clipped_layers_gpu_report(ui, clip_polygon, layers);
}

/// Composite layers behind a polygon clip and return explicit fidelity reporting.
pub fn clipped_layers_gpu_report(
    ui: &mut egui::Ui,
    clip_polygon: &[egui::Pos2],
    layers: Vec<BlendLayer>,
) -> RenderReport {
    let mask = ClipMask::from_polygon(clip_polygon.to_vec());
    if !mask.is_valid() {
        let mut report = composite_layers_gpu_report(ui, layers);
        report.add_issue(RenderIssue::new(
            RenderFeature::PolygonClip,
            RenderIssueKind::InvalidBounds,
            RenderQuality::Exact,
            RenderQuality::Approximate,
            "polygon clip is empty, non-finite, or degenerate; rendered unmasked blend group",
        ));
        return report;
    }
    let mut report = RenderReport::new(RenderBackendKind::CpuOffscreen, RenderQuality::Exact);
    let (rect, size, mut pixels, unhandled) = match rasterize_composited_layers_result(&layers) {
        Ok(result) => result,
        Err(error) => {
            report.add_issue(error.to_render_issue(RenderFeature::PolygonClip));
            paint_layers_without_group_compositing(ui, layers);
            return report;
        }
    };
    apply_polygon_alpha_mask(&mut pixels, size[0], size[1], rect.min, clip_polygon);

    #[cfg(feature = "wgpu")]
    {
        let rgba = pixels_to_rgba(&pixels);
        let id = blend_layers_hash(&layers, &pixels) ^ polygon_hash(clip_polygon);
        let callback = egui_wgpu::Callback::new_paint_callback(
            rect,
            crate::gpu::GpuCompositeCallback::new(id, size, rgba),
        );
        ui.painter().add(egui::Shape::Callback(callback));
    }

    #[cfg(not(feature = "wgpu"))]
    {
        let image = egui::ColorImage {
            size: [size[0] as usize, size[1] as usize],
            pixels,
            source_size: egui::vec2(size[0] as f32, size[1] as f32),
        };
        let texture = ui.ctx().load_texture(
            format!(
                "__egui_expressive_clipped_layers_{:x}_{:x}",
                blend_layers_hash(&layers, &image.pixels),
                polygon_hash(clip_polygon)
            ),
            image,
            egui::TextureOptions::LINEAR,
        );
        ui.painter().image(
            texture.id(),
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }

    for shape in unhandled {
        ui.painter().add(shape);
    }
    report
}

/// Composite layers and apply a CPU offscreen clip mask before painting.
pub fn clipped_layers_mask(ui: &mut egui::Ui, mask: &ClipMask, layers: Vec<BlendLayer>) {
    let _ = clipped_layers_mask_report(ui, mask, layers);
}

/// Composite layers behind a vector or alpha mask and return explicit fidelity reporting.
pub fn clipped_layers_mask_report(
    ui: &mut egui::Ui,
    mask: &ClipMask,
    layers: Vec<BlendLayer>,
) -> RenderReport {
    if !mask.is_valid() {
        let mut report = composite_layers_report(ui, layers);
        report.add_issue(RenderIssue::new(
            RenderFeature::CompoundClip,
            RenderIssueKind::InvalidBounds,
            RenderQuality::Exact,
            RenderQuality::Approximate,
            "clip mask is empty or invalid; rendered unmasked blend group",
        ));
        return report;
    }

    let mut report = RenderReport::new(RenderBackendKind::CpuOffscreen, RenderQuality::Exact);
    let (rect, size, mut pixels, unhandled) = match rasterize_composited_layers_result(&layers) {
        Ok(result) => result,
        Err(error) => {
            report.add_issue(error.to_render_issue(RenderFeature::CompoundClip));
            paint_layers_without_group_compositing(ui, layers);
            return report;
        }
    };
    apply_clip_mask(&mut pixels, size[0], size[1], rect.min, mask);

    let image = egui::ColorImage {
        size: [size[0] as usize, size[1] as usize],
        pixels,
        source_size: egui::vec2(size[0] as f32, size[1] as f32),
    };
    let texture = ui.ctx().load_texture(
        format!(
            "__egui_expressive_clipped_mask_{:x}",
            clip_mask_hash(mask) ^ blend_layers_hash(&layers, &image.pixels)
        ),
        image,
        egui::TextureOptions::LINEAR,
    );
    ui.painter().image(
        texture.id(),
        rect,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );
    for shape in unhandled {
        ui.painter().add(shape);
    }
    report
}
