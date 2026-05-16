use super::*;

const MAX_RASTER_DIMENSION: u32 = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RasterizeBlendError {
    InvalidBounds,
    InvalidClipMask,
    UnsupportedShapes { count: usize },
    LayerTooLarge { width: u32, height: u32, max: u32 },
}

impl RasterizeBlendError {
    pub(crate) fn to_render_issue(
        self,
        feature: crate::render::RenderFeature,
    ) -> crate::render::RenderIssue {
        match self {
            Self::InvalidBounds => crate::render::RenderIssue::new(
                feature,
                crate::render::RenderIssueKind::InvalidBounds,
                crate::render::RenderQuality::Exact,
                crate::render::RenderQuality::Approximate,
                "blend group has no finite drawable bounds; painted original shapes without per-pixel compositing",
            ),
            Self::InvalidClipMask => crate::render::RenderIssue::new(
                feature,
                crate::render::RenderIssueKind::InvalidBounds,
                crate::render::RenderQuality::Exact,
                crate::render::RenderQuality::Approximate,
                "clip polygon is empty, non-finite, or degenerate; painted original shapes without exact mask compositing",
            ),
            Self::UnsupportedShapes { count } => crate::render::RenderIssue::new(
                feature,
                crate::render::RenderIssueKind::UnsupportedShape,
                crate::render::RenderQuality::Exact,
                crate::render::RenderQuality::Approximate,
                format!(
                    "{count} shape(s) are not supported by the CPU blend rasterizer; painted original shapes without exact group compositing"
                ),
            ),
            Self::LayerTooLarge { width, height, max } => crate::render::RenderIssue::new(
                feature,
                crate::render::RenderIssueKind::SizeBudgetExceeded,
                crate::render::RenderQuality::Exact,
                crate::render::RenderQuality::Approximate,
                format!(
                    "blend group {width}×{height}px exceeds the {max}px per-axis offscreen budget; painted original shapes without exact group compositing"
                ),
            ),
        }
    }
}

pub(crate) fn rasterize_composited_layers_result(
    layers: &[BlendLayer],
) -> Result<RasterizedBlendGroup, RasterizeBlendError> {
    let rect = layers_bounds(layers).ok_or(RasterizeBlendError::InvalidBounds)?;
    let width = rect.width().ceil() as u32;
    let height = rect.height().ceil() as u32;
    if width > MAX_RASTER_DIMENSION || height > MAX_RASTER_DIMENSION {
        return Err(RasterizeBlendError::LayerTooLarge {
            width,
            height,
            max: MAX_RASTER_DIMENSION,
        });
    }
    let width = width.clamp(1, MAX_RASTER_DIMENSION);
    let height = height.clamp(1, MAX_RASTER_DIMENSION);
    let mut composited = vec![egui::Color32::TRANSPARENT; (width * height) as usize];
    let mut unhandled = Vec::new();

    for layer in layers {
        let mut layer_pixels = vec![egui::Color32::TRANSPARENT; composited.len()];
        for shape in &layer.shapes {
            rasterize_shape(
                shape,
                rect.min,
                width,
                height,
                &mut layer_pixels,
                &mut unhandled,
            );
        }
        if !unhandled.is_empty() {
            return Err(RasterizeBlendError::UnsupportedShapes {
                count: unhandled.len(),
            });
        }
        for polygon in &layer.clip_polygons {
            if !ClipMask::from_polygon(polygon.clone()).is_valid() {
                return Err(RasterizeBlendError::InvalidClipMask);
            }
            apply_polygon_alpha_mask(&mut layer_pixels, width, height, rect.min, polygon);
        }
        for (dst, src) in composited.iter_mut().zip(layer_pixels) {
            let src = color_with_opacity(src, layer.opacity);
            if src == egui::Color32::TRANSPARENT {
                continue;
            }
            *dst = blend_color(src, *dst, layer.blend_mode.clone());
        }
    }

    Ok((rect, [width, height], composited, unhandled))
}

pub(crate) fn layers_bounds(layers: &[BlendLayer]) -> Option<egui::Rect> {
    layers
        .iter()
        .flat_map(|layer| layer.shapes.iter())
        .filter_map(shape_bounds)
        .reduce(|a, b| a.union(b))
}

pub(crate) fn shape_bounds(shape: &egui::Shape) -> Option<egui::Rect> {
    match shape {
        egui::Shape::Rect(r) => valid_bounds(r.visual_bounding_rect()),
        egui::Shape::Circle(c) => valid_bounds(c.visual_bounding_rect()),
        egui::Shape::Ellipse(e) => valid_bounds(e.visual_bounding_rect()),
        egui::Shape::Path(p) => bounds_from_points(&p.points)
            .map(|rect| rect.expand(path_stroke_outset(&p.stroke, p.closed))),
        egui::Shape::LineSegment { points, stroke } => {
            bounds_from_points(points).map(|r| r.expand(stroke.width.max(1.0) * 0.5))
        }
        egui::Shape::Mesh(mesh) => mesh
            .vertices
            .iter()
            .map(|vertex| egui::Rect::from_min_max(vertex.pos, vertex.pos))
            .reduce(|a, b| a.union(b)),
        egui::Shape::Vec(shapes) => shapes
            .iter()
            .filter_map(shape_bounds)
            .reduce(|a, b| a.union(b)),
        _ => None,
    }
}

pub(crate) fn valid_bounds(rect: egui::Rect) -> Option<egui::Rect> {
    if rect.is_finite() && rect.is_positive() {
        Some(rect)
    } else {
        None
    }
}

pub(crate) fn path_stroke_outset(stroke: &egui::epaint::PathStroke, closed: bool) -> f32 {
    if stroke.is_empty() {
        return 0.0;
    }
    if !closed {
        return stroke.width.max(1.0) * 0.5;
    }
    match stroke.kind {
        egui::StrokeKind::Inside => 0.0,
        egui::StrokeKind::Middle => stroke.width.max(1.0) * 0.5,
        egui::StrokeKind::Outside => stroke.width.max(1.0),
    }
}

pub(crate) fn bounds_from_points(points: &[egui::Pos2]) -> Option<egui::Rect> {
    let first = points.first()?;
    let mut min = *first;
    let mut max = *first;
    for p in &points[1..] {
        min.x = min.x.min(p.x);
        min.y = min.y.min(p.y);
        max.x = max.x.max(p.x);
        max.y = max.y.max(p.y);
    }
    Some(egui::Rect::from_min_max(min, max))
}

pub(crate) fn rasterize_shape(
    shape: &egui::Shape,
    origin: egui::Pos2,
    width: u32,
    height: u32,
    pixels: &mut [egui::Color32],
    unhandled: &mut Vec<egui::Shape>,
) {
    match shape {
        egui::Shape::Rect(r) => {
            fill_rect_shape_pixels(r, origin, width, height, pixels);
            if r.stroke.width > 0.0 && r.stroke.color != egui::Color32::TRANSPARENT {
                stroke_rect_shape_pixels(r, origin, width, height, pixels);
            }
        }
        egui::Shape::Circle(c) => {
            fill_circle_pixels(c.center, c.radius, origin, width, height, c.fill, pixels);
            if c.stroke.width > 0.0 && c.stroke.color != egui::Color32::TRANSPARENT {
                stroke_circle_pixels(
                    c.center,
                    c.radius,
                    origin,
                    width,
                    height,
                    c.stroke.width,
                    c.stroke.color,
                    pixels,
                );
            }
        }
        egui::Shape::Ellipse(e) => {
            fill_ellipse_pixels(
                e.center, e.radius, e.angle, origin, width, height, e.fill, pixels,
            );
            if e.stroke.width > 0.0 && e.stroke.color != egui::Color32::TRANSPARENT {
                stroke_ellipse_pixels(
                    e.center,
                    e.radius,
                    e.angle,
                    origin,
                    width,
                    height,
                    e.stroke.width,
                    e.stroke.color,
                    pixels,
                );
            }
        }
        egui::Shape::Path(p) if p.closed => {
            fill_polygon_pixels(&p.points, origin, width, height, p.fill, pixels);
            if let Some(color) = path_stroke_color(&p.stroke) {
                stroke_polyline_pixels(
                    &p.points,
                    true,
                    origin,
                    width,
                    height,
                    p.stroke.width,
                    color,
                    pixels,
                );
            }
        }
        egui::Shape::Path(p) => {
            if let Some(color) = path_stroke_color(&p.stroke) {
                stroke_polyline_pixels(
                    &p.points,
                    false,
                    origin,
                    width,
                    height,
                    p.stroke.width,
                    color,
                    pixels,
                );
            }
        }
        egui::Shape::LineSegment { points, stroke } => {
            stroke_line_pixels(
                points[0],
                points[1],
                origin,
                width,
                height,
                stroke.width,
                stroke.color,
                pixels,
            );
        }
        egui::Shape::Mesh(mesh) => rasterize_mesh_pixels(mesh, origin, width, height, pixels),
        egui::Shape::Vec(shapes) => {
            for s in shapes {
                rasterize_shape(s, origin, width, height, pixels, unhandled);
            }
        }
        _ => {
            unhandled.push(shape.clone());
        }
    }
}
