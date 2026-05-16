use super::{
    color::blend_color,
    composite_hash::{blend_layers_hash, clip_mask_hash, pixels_to_rgba, polygon_hash},
    composite_masks::{apply_clip_mask, apply_polygon_alpha_mask},
    layout::ClipMask,
};
use egui::epaint::*;
use egui::*;

/// A layer of shapes to be composited with a specific blend mode and opacity.
///
/// Used with [`composite_layers`] to combine multiple layers using
/// Photoshop/Illustrator-style blend modes via CPU-side compositing.
pub struct BlendLayer {
    /// Shapes to render in this layer.
    pub shapes: Vec<egui::Shape>,
    /// Optional polygon masks applied to this layer before it is blended.
    /// Each polygon is tested with simple point-in-polygon (AND-combined for nesting).
    pub clip_polygons: Vec<Vec<egui::Pos2>>,
    /// Optional compound clip masks with fill rule (supports holes via EvenOdd).
    /// Applied after `clip_polygons`; multiple masks are AND-combined for nesting.
    pub clip_masks: Vec<ClipMask>,
    /// Blend mode for compositing this layer over the layers below it.
    pub blend_mode: crate::codegen::BlendMode,
    /// Overall opacity of this layer (0.0–1.0).
    pub opacity: f32,
}

type RasterizedBlendGroup = (egui::Rect, [u32; 2], Vec<egui::Color32>, Vec<egui::Shape>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RasterizeBlendError {
    UnsupportedContent,
    LayerTooLarge { width: u32, height: u32, max: u32 },
}

impl BlendLayer {
    /// Create a new blend layer with Normal blend mode and full opacity.
    pub fn new(shapes: Vec<egui::Shape>) -> Self {
        Self {
            shapes,
            clip_polygons: Vec::new(),
            clip_masks: Vec::new(),
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
        if polygon.len() >= 3 {
            self.clip_polygons.push(polygon);
        }
        self
    }

    /// Apply a compound clip mask (with fill rule support) to this layer.
    pub fn clip_mask(mut self, mask: ClipMask) -> Self {
        self.clip_masks.push(mask);
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
    if layers.is_empty() {
        return;
    }
    let (rect, size, pixels, unhandled) = match rasterize_composited_layers(&layers) {
        Ok(result) => result,
        Err(error) => {
            paint_blend_rasterization_failure(ui, &layers, error);
            return;
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
}

/// Composite layers through an egui-wgpu [`PaintCallback`] when the `wgpu`
/// feature is enabled. Call [`crate::init_gpu_effects`] once during app startup
/// before using this path. Without `wgpu`, this falls back to [`composite_layers`].
#[cfg(feature = "wgpu")]
pub fn composite_layers_gpu(ui: &mut egui::Ui, layers: Vec<BlendLayer>) {
    let (rect, size, pixels, unhandled) = match rasterize_composited_layers(&layers) {
        Ok(result) => result,
        Err(_) => {
            composite_layers(ui, layers);
            return;
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
}

#[cfg(not(feature = "wgpu"))]
pub fn composite_layers_gpu(ui: &mut egui::Ui, layers: Vec<BlendLayer>) {
    composite_layers(ui, layers)
}

/// Composite layers and apply an arbitrary polygon mask before painting.
///
/// This is the vector-export friendly clipping path: supplied [`BlendLayer`]s are
/// rasterized into a single per-pixel layer group, every pixel outside
/// `clip_polygon` is made transparent, and the result is painted as one texture.
/// With the `wgpu` feature enabled it is presented through the egui-wgpu callback
/// pipeline; otherwise it uses egui's texture painter as a CPU fallback.
pub fn clipped_layers_gpu(ui: &mut egui::Ui, clip_polygon: &[egui::Pos2], layers: Vec<BlendLayer>) {
    if clip_polygon.len() < 3 {
        composite_layers_gpu(ui, layers);
        return;
    }
    let (rect, size, mut pixels, unhandled) = match rasterize_composited_layers(&layers) {
        Ok(result) => result,
        Err(error) => {
            paint_blend_rasterization_failure(ui, &layers, error);
            return;
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
}

/// Composite layers and apply an arbitrary clip mask (with fill rule support).
///
/// This extends [`clipped_layers_gpu`] by supporting compound clip masks with
/// even-odd fill rules for holes. The mask is applied per-pixel to the composited
/// layer group before presentation.
pub fn clipped_layers_mask(ui: &mut egui::Ui, mask: &ClipMask, layers: Vec<BlendLayer>) {
    if mask.contours.is_empty() || mask.contours.iter().all(|c| c.len() < 3) {
        composite_layers_gpu(ui, layers);
        return;
    }
    let (rect, size, mut pixels, unhandled) = match rasterize_composited_layers(&layers) {
        Ok(result) => result,
        Err(error) => {
            paint_blend_rasterization_failure(ui, &layers, error);
            return;
        }
    };
    apply_clip_mask(&mut pixels, size[0], size[1], rect.min, mask);

    #[cfg(feature = "wgpu")]
    {
        let rgba = pixels_to_rgba(&pixels);
        let id = blend_layers_hash(&layers, &pixels) ^ clip_mask_hash(mask);
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
    }

    for shape in unhandled {
        ui.painter().add(shape);
    }
}

pub(crate) fn rasterize_composited_layers(
    layers: &[BlendLayer],
) -> Result<RasterizedBlendGroup, RasterizeBlendError> {
    let rect = layers_bounds(layers).ok_or(RasterizeBlendError::UnsupportedContent)?;
    let width = rect.width().ceil() as u32;
    let height = rect.height().ceil() as u32;
    if width > 4096 || height > 4096 {
        return Err(RasterizeBlendError::LayerTooLarge {
            width,
            height,
            max: 4096,
        });
    }
    let width = width.clamp(1, 4096);
    let height = height.clamp(1, 4096);
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
        for polygon in &layer.clip_polygons {
            apply_polygon_alpha_mask(&mut layer_pixels, width, height, rect.min, polygon);
        }
        for mask in &layer.clip_masks {
            apply_clip_mask(&mut layer_pixels, width, height, rect.min, mask);
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

fn paint_blend_rasterization_failure(
    ui: &mut egui::Ui,
    layers: &[BlendLayer],
    error: RasterizeBlendError,
) {
    let bounds = layers_bounds(layers).unwrap_or_else(|| ui.clip_rect());
    let rect = bounds.intersect(ui.clip_rect()).expand(4.0);
    let stroke = Stroke::new(1.5, Color32::from_rgb(220, 60, 60));
    ui.painter()
        .rect_stroke(rect, 0.0, stroke, StrokeKind::Outside);
    let message = match error {
        RasterizeBlendError::UnsupportedContent => "unsupported blend rasterization".to_owned(),
        RasterizeBlendError::LayerTooLarge { width, height, max } => format!(
            "unsupported blend rasterization\n{}×{} exceeds {}px limit",
            width, height, max
        ),
    };
    ui.painter().text(
        rect.left_top() + Vec2::new(4.0, 4.0),
        Align2::LEFT_TOP,
        message,
        FontId::monospace(11.0),
        Color32::from_rgb(220, 60, 60),
    );
}

fn layers_bounds(layers: &[BlendLayer]) -> Option<egui::Rect> {
    layers
        .iter()
        .flat_map(|layer| layer.shapes.iter())
        .filter_map(shape_bounds)
        .reduce(|a, b| a.union(b))
}

fn shape_bounds(shape: &egui::Shape) -> Option<egui::Rect> {
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

fn valid_bounds(rect: egui::Rect) -> Option<egui::Rect> {
    if rect.is_finite() && rect.is_positive() {
        Some(rect)
    } else {
        None
    }
}

fn path_stroke_outset(stroke: &egui::epaint::PathStroke, closed: bool) -> f32 {
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

fn bounds_from_points(points: &[egui::Pos2]) -> Option<egui::Rect> {
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

fn rasterize_shape(
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

fn path_stroke_color(stroke: &egui::epaint::PathStroke) -> Option<egui::Color32> {
    if stroke.width <= 0.0 {
        return None;
    }
    match stroke.color {
        egui::epaint::ColorMode::Solid(color) if color != egui::Color32::TRANSPARENT => Some(color),
        _ => None,
    }
}
