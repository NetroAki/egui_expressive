use super::*;

pub(crate) fn paint_effect(
    ui: &egui::Ui,
    origin: egui::Vec2,
    geometry: &Geometry,
    effect: &EffectLayer,
    node_opacity: f32,
    node_blend_mode: &BlendMode,
    force_normal_blend: bool,
) -> Vec<egui::Shape> {
    #[cfg(feature = "wgpu")]
    if let Some(shapes) = paint_exact_source_layer_effect(
        ui,
        origin,
        geometry,
        effect,
        node_opacity,
        node_blend_mode,
        force_normal_blend,
    ) {
        return shapes;
    }

    let opacity = effect.opacity * node_opacity;
    let blend_mode = if force_normal_blend {
        &BlendMode::Normal
    } else {
        requested_effect_blend_mode(effect, node_blend_mode)
    };

    let rect = offset_rect(geometry.bounds(), origin);
    let color = resolve_color(ui, effect.params.color, opacity, blend_mode);
    let mut shapes = Vec::new();

    match effect.effect_type {
        EffectType::DropShadow => {
            shapes.extend(crate::draw::box_shadow(
                rect,
                color,
                effect.params.blur,
                effect.params.spread,
                crate::draw::ShadowOffset::new(effect.params.x, effect.params.y),
            ));
        }
        EffectType::OuterGlow => {
            shapes.extend(crate::blur::soft_shadow(
                rect,
                color,
                effect.params.blur,
                0.0,
                crate::draw::ShadowOffset::zero(),
                crate::blur::BlurQuality::Medium,
            ));
        }
        EffectType::GaussianBlur | EffectType::Feather => {
            shapes.extend(crate::blur::soft_shadow(
                rect,
                color,
                effect.params.blur.max(effect.params.radius),
                0.0,
                crate::draw::ShadowOffset::zero(),
                crate::blur::BlurQuality::High,
            ));
        }
        EffectType::InnerShadow | EffectType::InnerGlow => {
            shapes.extend(crate::draw::inner_shadow(rect, color, effect.params.blur));
        }
        _ => {}
    }
    shapes
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SceneBlurEffectContract {
    pub(crate) solid_rect_source: bool,
    pub(crate) shaped_source: bool,
    pub(crate) normal_blend: bool,
    pub(crate) gpu_resources_ready: bool,
}

impl SceneBlurEffectContract {
    #[allow(dead_code)]
    pub(crate) fn exact_solid_rect_source() -> Self {
        Self {
            solid_rect_source: true,
            shaped_source: false,
            normal_blend: true,
            gpu_resources_ready: true,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn exact_shaped_source() -> Self {
        Self {
            solid_rect_source: false,
            shaped_source: true,
            normal_blend: true,
            gpu_resources_ready: true,
        }
    }
}

#[allow(dead_code)]
pub(crate) fn scene_blur_effect_report(
    effect_type: EffectType,
    capabilities: &crate::render::RenderCapabilities,
    request: crate::render::OffscreenRequest,
    contract: SceneBlurEffectContract,
) -> crate::render::RenderReport {
    use crate::render::{
        RenderBackendKind, RenderFeature, RenderIssue, RenderIssueKind, RenderQuality, RenderReport,
    };

    let mut report = RenderReport::new(capabilities.backend, request.requested_quality);
    if !matches!(effect_type, EffectType::GaussianBlur | EffectType::Feather) {
        report.add_issue(RenderIssue::new(
            RenderFeature::Blur,
            RenderIssueKind::UnsupportedFeature,
            request.requested_quality,
            RenderQuality::Unsupported,
            "scene blur report only covers GaussianBlur and Feather effects",
        ));
        return report;
    }

    if (!contract.solid_rect_source && !contract.shaped_source)
        || !contract.normal_blend
        || !contract.gpu_resources_ready
    {
        report.add_issue(RenderIssue::new(
            RenderFeature::Blur,
            RenderIssueKind::UnsupportedFeature,
            request.requested_quality,
            RenderQuality::Approximate,
            "exact scene blur requires initialized GPU resources, normal blend, and an approved library-owned RGBA source layer; falling back to soft_shadow approximation",
        ));
        return report;
    }

    let exceeds_phase9a_axis_budget = request.width == 0
        || request.height == 0
        || request.width > 4_096
        || request.height > 4_096;
    if exceeds_phase9a_axis_budget {
        report.add_issue(RenderIssue::new(
            RenderFeature::Blur,
            RenderIssueKind::SizeBudgetExceeded,
            request.requested_quality,
            RenderQuality::Unsupported,
            "scene source-layer blur exceeds the Phase 9A per-axis 4096 pixel budget",
        ));
        return report;
    }

    if matches!(
        capabilities.backend,
        RenderBackendKind::EguiWgpuCallback | RenderBackendKind::WgpuOffscreen
    ) && capabilities.exact_large_blur
        && request.fits(capabilities)
    {
        return report;
    }

    let (kind, quality, message) = if !request.fits(capabilities) {
        (
            RenderIssueKind::SizeBudgetExceeded,
            RenderQuality::Unsupported,
            "scene blur request exceeds the offscreen pixel budget",
        )
    } else {
        (
            RenderIssueKind::ApproximateFallback,
            RenderQuality::Approximate,
            "non-WGPU scene GaussianBlur/Feather uses egui soft_shadow approximation",
        )
    };
    report.add_issue(RenderIssue::new(
        RenderFeature::Blur,
        kind,
        request.requested_quality,
        quality,
        message,
    ));
    report
}

#[allow(dead_code)]
pub(crate) fn scene_shadow_effect_report(
    effect_type: EffectType,
    requested_radius: f32,
    capabilities: &crate::render::RenderCapabilities,
    request: crate::render::OffscreenRequest,
    contract: SceneBlurEffectContract,
) -> crate::render::RenderReport {
    use crate::render::{
        RenderBackendKind, RenderFeature, RenderIssue, RenderIssueKind, RenderQuality, RenderReport,
    };

    let mut report = RenderReport::new(capabilities.backend, request.requested_quality);
    if !matches!(effect_type, EffectType::DropShadow | EffectType::OuterGlow) {
        report.add_issue(RenderIssue::new(
            RenderFeature::Shadow,
            RenderIssueKind::UnsupportedFeature,
            request.requested_quality,
            RenderQuality::Unsupported,
            "scene shadow report only covers DropShadow and OuterGlow effects",
        ));
        return report;
    }

    if requested_radius < 1.0 {
        report.add_issue(RenderIssue::new(
            RenderFeature::Shadow,
            RenderIssueKind::ApproximateFallback,
            request.requested_quality,
            RenderQuality::Approximate,
            "exact scene shadow requires requested blur/radius >= 1.0; falling back to box_shadow/soft_shadow approximation",
        ));
        return report;
    }

    if (!contract.solid_rect_source && !contract.shaped_source)
        || !contract.normal_blend
        || !contract.gpu_resources_ready
    {
        report.add_issue(RenderIssue::new(
            RenderFeature::Shadow,
            RenderIssueKind::UnsupportedFeature,
            request.requested_quality,
            RenderQuality::Approximate,
            "exact scene shadow requires initialized GPU resources, normal blend, and an approved library-owned RGBA source layer; falling back to box_shadow/soft_shadow approximation",
        ));
        return report;
    }

    let exceeds_phase9b_axis_budget = request.width == 0
        || request.height == 0
        || request.width > 4_096
        || request.height > 4_096;
    if exceeds_phase9b_axis_budget {
        report.add_issue(RenderIssue::new(
            RenderFeature::Shadow,
            RenderIssueKind::SizeBudgetExceeded,
            request.requested_quality,
            RenderQuality::Unsupported,
            "scene source-layer shadow exceeds the Phase 9B per-axis 4096 pixel budget",
        ));
        return report;
    }

    if matches!(
        capabilities.backend,
        RenderBackendKind::EguiWgpuCallback | RenderBackendKind::WgpuOffscreen
    ) && capabilities.exact_large_blur
        && request.fits(capabilities)
    {
        return report;
    }

    let (kind, quality, message) = if !request.fits(capabilities) {
        (
            RenderIssueKind::SizeBudgetExceeded,
            RenderQuality::Unsupported,
            "scene shadow request exceeds the offscreen pixel budget",
        )
    } else {
        (
            RenderIssueKind::ApproximateFallback,
            RenderQuality::Approximate,
            "non-WGPU scene DropShadow/OuterGlow uses bounded box_shadow/soft_shadow approximation",
        )
    };
    report.add_issue(RenderIssue::new(
        RenderFeature::Shadow,
        kind,
        request.requested_quality,
        quality,
        message,
    ));
    report
}

fn requested_effect_blend_mode<'a>(
    effect: &'a EffectLayer,
    node_blend_mode: &'a BlendMode,
) -> &'a BlendMode {
    if effect.blend_mode != BlendMode::Normal {
        &effect.blend_mode
    } else {
        node_blend_mode
    }
}

#[cfg(feature = "wgpu")]
fn paint_exact_source_layer_effect(
    ui: &egui::Ui,
    origin: egui::Vec2,
    geometry: &Geometry,
    effect: &EffectLayer,
    node_opacity: f32,
    node_blend_mode: &BlendMode,
    _force_normal_blend: bool,
) -> Option<Vec<egui::Shape>> {
    use std::hash::{Hash, Hasher};

    if !matches!(
        effect.effect_type,
        EffectType::GaussianBlur
            | EffectType::Feather
            | EffectType::DropShadow
            | EffectType::OuterGlow
    ) {
        return None;
    }

    let requested_blend_mode = requested_effect_blend_mode(effect, node_blend_mode);
    if *requested_blend_mode != BlendMode::Normal {
        return None;
    }

    if !crate::gpu::gpu_effects_initialized_for_context(ui.ctx()) {
        return None;
    }

    let opacity = effect.opacity * node_opacity;
    let color = resolve_color(ui, effect.params.color, opacity, &BlendMode::Normal);
    let spec = source_layer_effect_spec(effect, geometry, origin, color)?;
    let width_px = spec.output_rect.width().ceil();
    let height_px = spec.output_rect.height().ceil();
    if !(1.0..=4096.0).contains(&width_px) || !(1.0..=4096.0).contains(&height_px) {
        return None;
    }
    let width = width_px as u32;
    let height = height_px as u32;
    let capabilities = crate::render::RenderCapabilities::egui_wgpu_callback(4096 * 4096);
    let request = crate::render::OffscreenRequest {
        feature: spec.feature,
        width,
        height,
        requested_quality: crate::render::RenderQuality::Exact,
    };
    let report = crate::gpu::wgpu_source_layer_effect_report(
        &capabilities,
        request,
        crate::gpu::GpuEffectSource::LibraryOwnedSourceLayer,
    );
    if !report.is_exact() {
        return None;
    }

    let rgba = source_layer_rgba(width, height, &spec, color)?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    "phase9b-scene-source-layer-effect".hash(&mut hasher);
    effect.effect_type_hash().hash(&mut hasher);
    spec.output_rect.min.x.to_bits().hash(&mut hasher);
    spec.output_rect.min.y.to_bits().hash(&mut hasher);
    spec.source_rect.min.x.to_bits().hash(&mut hasher);
    spec.source_rect.min.y.to_bits().hash(&mut hasher);
    width.hash(&mut hasher);
    height.hash(&mut hasher);
    spec.radius.to_bits().hash(&mut hasher);
    color.to_array().hash(&mut hasher);
    rgba.hash(&mut hasher);
    let callback = egui_wgpu::Callback::new_paint_callback(
        spec.output_rect,
        crate::gpu::GpuSourceLayerEffectCallback::new_blur(
            hasher.finish(),
            [width, height],
            rgba,
            spec.radius,
        ),
    );
    Some(vec![egui::Shape::Callback(callback)])
}

#[cfg(feature = "wgpu")]
struct SourceLayerEffectSpec {
    feature: crate::render::RenderFeature,
    output_rect: egui::Rect,
    source_rect: egui::Rect,
    radius: f32,
    source: SourceLayerPixels,
}

#[cfg(feature = "wgpu")]
enum SourceLayerPixels {
    SolidRect,
    RasterizedShape(egui::Shape),
}

#[cfg(feature = "wgpu")]
struct SourceLayerSource {
    rect: egui::Rect,
    pixels: SourceLayerPixels,
}

#[cfg(feature = "wgpu")]
fn source_layer_effect_spec(
    effect: &EffectLayer,
    geometry: &Geometry,
    origin: egui::Vec2,
    color: egui::Color32,
) -> Option<SourceLayerEffectSpec> {
    let requested_radius = effect.params.blur.max(effect.params.radius);
    match effect.effect_type {
        EffectType::GaussianBlur | EffectType::Feather => {
            let source = source_layer_source(geometry, origin, color)?;
            Some(SourceLayerEffectSpec {
                feature: crate::render::RenderFeature::Blur,
                output_rect: source.rect,
                source_rect: source.rect,
                radius: requested_radius.max(1.0),
                source: source.pixels,
            })
        }
        EffectType::DropShadow | EffectType::OuterGlow => {
            if requested_radius < 1.0 {
                return None;
            }
            if effect.params.spread < 0.0 {
                return None;
            }
            let offset = if effect.effect_type == EffectType::DropShadow {
                egui::vec2(effect.params.x, effect.params.y)
            } else {
                egui::Vec2::ZERO
            };
            let source = source_layer_source(geometry, origin + offset, color)?;
            let is_solid_rect = matches!(source.pixels, SourceLayerPixels::SolidRect);
            if !is_solid_rect && effect.params.spread != 0.0 {
                return None;
            }
            let source_rect = if is_solid_rect {
                source.rect.expand(effect.params.spread.max(0.0))
            } else {
                source.rect
            };
            let output_rect = source_rect.expand(requested_radius.ceil());
            if !source_rect.is_finite()
                || !source_rect.is_positive()
                || !output_rect.is_finite()
                || !output_rect.is_positive()
            {
                return None;
            }
            Some(SourceLayerEffectSpec {
                feature: crate::render::RenderFeature::Shadow,
                output_rect,
                source_rect,
                radius: requested_radius,
                source: if is_solid_rect {
                    SourceLayerPixels::SolidRect
                } else {
                    source.pixels
                },
            })
        }
        _ => None,
    }
}

#[cfg(feature = "wgpu")]
fn source_layer_source(
    geometry: &Geometry,
    origin: egui::Vec2,
    color: egui::Color32,
) -> Option<SourceLayerSource> {
    let shape = match geometry {
        Geometry::Rect {
            rect,
            corner_radius,
        } if *corner_radius == 0.0 => {
            let rect = offset_rect(*rect, origin);
            if !rect.is_finite() || !rect.is_positive() {
                return None;
            }
            return Some(SourceLayerSource {
                rect,
                pixels: SourceLayerPixels::SolidRect,
            });
        }
        Geometry::Rect {
            rect,
            corner_radius,
        } => egui::Shape::rect_filled(offset_rect(*rect, origin), *corner_radius, color),
        Geometry::Ellipse { rect } => {
            let rect = offset_rect(*rect, origin);
            egui::Shape::ellipse_filled(
                rect.center(),
                egui::vec2(rect.width() * 0.5, rect.height() * 0.5),
                color,
            )
        }
        Geometry::Path { points, closed } if *closed && points.len() >= 3 => {
            egui::Shape::Path(egui::epaint::PathShape {
                points: offset_points(points, origin),
                closed: true,
                fill: color,
                stroke: egui::Stroke::NONE.into(),
            })
        }
        _ => return None,
    };
    let rect = crate::draw::shape_bounds(&shape)?;
    if !rect.is_finite() || !rect.is_positive() {
        return None;
    }
    Some(SourceLayerSource {
        rect,
        pixels: SourceLayerPixels::RasterizedShape(shape),
    })
}

#[cfg(feature = "wgpu")]
fn source_layer_rgba(
    width: u32,
    height: u32,
    spec: &SourceLayerEffectSpec,
    color: egui::Color32,
) -> Option<Vec<u8>> {
    match &spec.source {
        SourceLayerPixels::SolidRect => Some(source_rect_rgba(
            width,
            height,
            spec.output_rect,
            spec.source_rect,
            color,
        )),
        SourceLayerPixels::RasterizedShape(shape) => {
            let mut pixels = vec![egui::Color32::TRANSPARENT; (width * height) as usize];
            let mut unhandled = Vec::new();
            crate::draw::rasterize_shape(
                shape,
                spec.output_rect.min,
                width,
                height,
                &mut pixels,
                &mut unhandled,
            );
            if !unhandled.is_empty() {
                return None;
            }
            Some(crate::draw::pixels_to_rgba(&pixels))
        }
    }
}

#[cfg(feature = "wgpu")]
fn source_rect_rgba(
    width: u32,
    height: u32,
    output_rect: egui::Rect,
    source_rect: egui::Rect,
    color: egui::Color32,
) -> Vec<u8> {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    let mut rgba = vec![0; (width * height * 4) as usize];
    let min_x = (source_rect.min.x - output_rect.min.x)
        .floor()
        .clamp(0.0, width as f32) as u32;
    let min_y = (source_rect.min.y - output_rect.min.y)
        .floor()
        .clamp(0.0, height as f32) as u32;
    let max_x = (source_rect.max.x - output_rect.min.x)
        .ceil()
        .clamp(0.0, width as f32) as u32;
    let max_y = (source_rect.max.y - output_rect.min.y)
        .ceil()
        .clamp(0.0, height as f32) as u32;
    for y in min_y..max_y {
        for x in min_x..max_x {
            let idx = ((y * width + x) * 4) as usize;
            rgba[idx..idx + 4].copy_from_slice(&[r, g, b, a]);
        }
    }
    rgba
}

#[cfg(feature = "wgpu")]
trait EffectTypeHash {
    fn effect_type_hash(&self) -> u8;
}

#[cfg(feature = "wgpu")]
impl EffectTypeHash for EffectLayer {
    fn effect_type_hash(&self) -> u8 {
        match self.effect_type {
            EffectType::GaussianBlur => 1,
            EffectType::Feather => 2,
            EffectType::DropShadow => 3,
            EffectType::OuterGlow => 4,
            _ => 0,
        }
    }
}

pub(crate) fn resolve_color(
    ui: &egui::Ui,
    color: egui::Color32,
    opacity: f32,
    blend_mode: &BlendMode,
) -> egui::Color32 {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    let color = egui::Color32::from_rgba_unmultiplied(
        r,
        g,
        b,
        (a as f32 * opacity).clamp(0.0, 255.0) as u8,
    );
    if *blend_mode == BlendMode::Normal {
        color
    } else {
        crate::draw::blend_color(color, ui.visuals().window_fill(), blend_mode.clone())
    }
}

pub(crate) fn gradient_stops(
    gradient: &GradientDef,
    opacity: f32,
    ui: &egui::Ui,
    blend_mode: &BlendMode,
) -> Vec<(f32, egui::Color32)> {
    gradient
        .stops
        .iter()
        .map(|stop| {
            (
                stop.position,
                resolve_color(ui, stop.color, opacity, blend_mode),
            )
        })
        .collect()
}

pub(crate) fn sample_layout_path(
    points: &[crate::codegen::PathPoint],
    closed: bool,
) -> Vec<egui::Pos2> {
    if points.is_empty() {
        return Vec::new();
    }
    if points.len() == 1 {
        return vec![egui::pos2(points[0].anchor[0], points[0].anchor[1])];
    }
    let mut sampled = Vec::new();
    let segment_count = if closed {
        points.len()
    } else {
        points.len() - 1
    };
    for idx in 0..segment_count {
        let next_idx = (idx + 1) % points.len();
        let current = &points[idx];
        let next = &points[next_idx];
        let p0 = egui::pos2(current.anchor[0], current.anchor[1]);
        let p1 = egui::pos2(current.right_ctrl[0], current.right_ctrl[1]);
        let p2 = egui::pos2(next.left_ctrl[0], next.left_ctrl[1]);
        let p3 = egui::pos2(next.anchor[0], next.anchor[1]);
        if sampled.is_empty() {
            sampled.push(p0);
        }
        let is_line = p0.distance(p1) < 0.01 && p2.distance(p3) < 0.01;
        let steps = if is_line { 1 } else { 12 };
        for step in 1..=steps {
            let t = step as f32 / steps as f32;
            sampled.push(cubic_bezier(p0, p1, p2, p3, t));
        }
    }
    sampled
}

pub(crate) fn cubic_bezier(
    p0: egui::Pos2,
    p1: egui::Pos2,
    p2: egui::Pos2,
    p3: egui::Pos2,
    t: f32,
) -> egui::Pos2 {
    let mt = 1.0 - t;
    let v = p0.to_vec2() * (mt * mt * mt)
        + p1.to_vec2() * (3.0 * mt * mt * t)
        + p2.to_vec2() * (3.0 * mt * t * t)
        + p3.to_vec2() * (t * t * t);
    egui::pos2(v.x, v.y)
}

pub(crate) fn offset_rect(rect: egui::Rect, origin: egui::Vec2) -> egui::Rect {
    rect.translate(origin)
}

pub(crate) fn offset_points(points: &[egui::Pos2], origin: egui::Vec2) -> Vec<egui::Pos2> {
    points.iter().map(|p| *p + origin).collect()
}

pub(crate) fn offset_transform(matrix: [f32; 6], origin: egui::Vec2) -> crate::draw::Transform2D {
    let [a, b, c, d, e, f] = matrix;
    crate::draw::Transform2D {
        a,
        b,
        c,
        d,
        e: origin.x + e - a * origin.x - c * origin.y,
        f: origin.y + f - b * origin.x - d * origin.y,
    }
}

pub(crate) fn rotate_geometry(geometry: &Geometry, angle_deg: f32) -> Geometry {
    let transform = crate::draw::Transform2D::rotate_around(angle_deg, geometry.bounds().center());
    match geometry {
        Geometry::Group { bounds } => Geometry::Group {
            bounds: transform.apply_to_rect(*bounds),
        },
        Geometry::Rect {
            rect,
            corner_radius,
        } => Geometry::Path {
            points: crate::draw::rounded_rect_path(*rect, *corner_radius)
                .into_iter()
                .map(|point| transform.apply(point))
                .collect(),
            closed: true,
        },
        Geometry::Ellipse { rect } => Geometry::Path {
            points: ellipse_points(*rect, 48)
                .into_iter()
                .map(|point| transform.apply(point))
                .collect(),
            closed: true,
        },
        Geometry::Path { points, closed } => Geometry::Path {
            points: points.iter().map(|point| transform.apply(*point)).collect(),
            closed: *closed,
        },
        Geometry::MeshPatch {
            corners,
            colors,
            subdivisions,
        } => Geometry::MeshPatch {
            corners: corners.map(|point| transform.apply(point)),
            colors: *colors,
            subdivisions: *subdivisions,
        },
    }
}

pub(crate) fn ellipse_points(rect: egui::Rect, segments: usize) -> Vec<egui::Pos2> {
    let center = rect.center();
    let rx = rect.width() * 0.5;
    let ry = rect.height() * 0.5;
    let segments = segments.max(adaptive_ellipse_segments(rect));
    (0..segments.max(3))
        .map(|idx| {
            let angle = std::f32::consts::TAU * idx as f32 / segments.max(3) as f32;
            center + egui::vec2(angle.cos() * rx, angle.sin() * ry)
        })
        .collect()
}

pub(crate) fn adaptive_ellipse_segments(rect: egui::Rect) -> usize {
    let rx = rect.width().abs() * 0.5;
    let ry = rect.height().abs() * 0.5;
    let perimeter_estimate =
        std::f32::consts::PI * (3.0 * (rx + ry) - ((3.0 * rx + ry) * (rx + 3.0 * ry)).sqrt());
    (perimeter_estimate / 4.0).ceil().clamp(48.0, 160.0) as usize
}

pub(crate) fn geometry_to_polygon(geometry: &Geometry, origin: egui::Vec2) -> Vec<egui::Pos2> {
    match geometry {
        Geometry::Rect {
            rect,
            corner_radius,
        } if *corner_radius > 0.001 => crate::draw::rounded_rect_path(*rect, *corner_radius)
            .into_iter()
            .map(|point| point + origin)
            .collect(),
        Geometry::Group { bounds } | Geometry::Rect { rect: bounds, .. } => {
            let r = offset_rect(*bounds, origin);
            vec![
                r.min,
                egui::pos2(r.max.x, r.min.y),
                r.max,
                egui::pos2(r.min.x, r.max.y),
            ]
        }
        Geometry::Ellipse { rect } => ellipse_points(offset_rect(*rect, origin), 48),
        Geometry::Path { points, .. } => offset_points(points, origin),
        Geometry::MeshPatch { corners, .. } => corners.map(|p| p + origin).to_vec(),
    }
}

pub(crate) fn bounds_for_points(points: &[egui::Pos2; 4]) -> egui::Rect {
    bounds_for_slice(points)
}

pub(crate) fn bounds_for_slice(points: &[egui::Pos2]) -> egui::Rect {
    if points.is_empty() {
        return egui::Rect::NOTHING;
    }
    let mut min = points[0];
    let mut max = points[0];
    for p in points.iter().skip(1) {
        min.x = min.x.min(p.x);
        min.y = min.y.min(p.y);
        max.x = max.x.max(p.x);
        max.y = max.y.max(p.y);
    }
    egui::Rect::from_min_max(min, max)
}
