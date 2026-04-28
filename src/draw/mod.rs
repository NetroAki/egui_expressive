//! Layered painter helpers and fluent shape builders for egui.

use egui::{
    epaint::{PathShape, PathStroke, RectShape, StrokeKind},
    Color32, CornerRadius, Id, LayerId, Order, Pos2, Rect, Shape, Stroke,
};

pub fn with_clip_path(painter: &egui::Painter, path: Vec<egui::Pos2>) -> egui::Painter {
    let Some(first) = path.first().copied() else {
        return painter.clone();
    };

    let bounds = path
        .iter()
        .skip(1)
        .fold(Rect::from_min_max(first, first), |rect, point| {
            rect.union(Rect::from_min_max(*point, *point))
        });

    // egui Painter supports rectangular clipping only. Arbitrary polygon masks are
    // handled by `clipped_layers_gpu`; this scoped helper still applies the tight
    // path bounds so generated code is clipped instead of silently unbounded.
    painter.with_clip_rect(painter.clip_rect().intersect(bounds))
}

pub fn with_blend_mode(painter: &egui::Painter, _mode: crate::codegen::BlendMode) -> egui::Painter {
    // egui doesn't support blend modes on painters yet
    painter.clone()
}

// ---------------------------------------------------------------------------
// LayeredPainter
// ---------------------------------------------------------------------------

/// Wraps an egui context to provide named layer painters with inherited clip rect.
pub struct LayeredPainter<'a> {
    ctx: &'a egui::Context,
    clip_rect: Rect,
    default_id: Id,
}

impl<'a> LayeredPainter<'a> {
    /// Create a new `LayeredPainter` from a [`Ui`].
    #[inline]
    pub fn from_ui(ui: &'a egui::Ui) -> Self {
        Self {
            ctx: ui.ctx(),
            clip_rect: ui.clip_rect(),
            default_id: ui.id(),
        }
    }

    /// Painter on the `Background` layer.
    #[inline]
    pub fn background(&self) -> egui::Painter {
        self.layer_painter(LayerId::new(Order::Background, self.default_id))
    }

    /// Painter on the main layer (between background and foreground).
    #[inline]
    pub fn main(&self) -> egui::Painter {
        self.layer_painter(LayerId::new(Order::Middle, self.default_id))
    }

    /// Painter on the `Foreground` layer.
    #[inline]
    pub fn foreground(&self) -> egui::Painter {
        self.layer_painter(LayerId::new(Order::Foreground, self.default_id))
    }

    /// Painter on the `Tooltip` layer.
    #[inline]
    pub fn tooltip(&self) -> egui::Painter {
        self.layer_painter(LayerId::new(Order::Tooltip, self.default_id))
    }

    /// Painter on a custom named layer.
    #[inline]
    pub fn layer(&self, name: &str) -> egui::Painter {
        self.layer_painter(LayerId::new(Order::Middle, Id::new(name)))
    }

    /// Returns a painter clipped to the given sub-rectangle.
    #[inline]
    pub fn clipped(&self, rect: Rect) -> egui::Painter {
        self.main().with_clip_rect(rect)
    }

    fn layer_painter(&self, layer_id: LayerId) -> egui::Painter {
        self.ctx
            .layer_painter(layer_id)
            .with_clip_rect(self.clip_rect)
    }
}

// ---------------------------------------------------------------------------
// ShapeBuilder
// ---------------------------------------------------------------------------

/// Static entry point for fluent shape builders.
pub struct ShapeBuilder;

impl ShapeBuilder {
    /// Begin building a filled rectangle.
    #[inline]
    pub fn rect(rect: Rect) -> RectBuilder {
        RectBuilder {
            rect,
            fill: Color32::TRANSPARENT,
            stroke: Stroke::NONE,
            rounding: CornerRadius::ZERO,
        }
    }

    /// Begin building a circle.
    #[inline]
    pub fn circle(center: Pos2, radius: f32) -> CircleBuilder {
        CircleBuilder {
            center,
            radius,
            fill: Color32::TRANSPARENT,
            stroke: Stroke::NONE,
        }
    }

    /// Begin building a path.
    #[inline]
    pub fn path(points: Vec<Pos2>) -> PathBuilder {
        PathBuilder {
            points,
            closed: false,
            fill: Color32::TRANSPARENT,
            stroke: PathStroke::NONE,
        }
    }

    /// Shortcut: a simple line segment.
    #[inline]
    pub fn line(a: Pos2, b: Pos2, stroke: Stroke) -> Shape {
        Shape::LineSegment {
            points: [a, b],
            stroke,
        }
    }

    /// A diamond (rhombus) shape centered at `center`.
    #[inline]
    pub fn diamond(center: Pos2, size: f32, fill: Color32, stroke: Stroke) -> Shape {
        let s = size;
        let half = s / 2.0;
        Shape::Path(PathShape {
            points: vec![
                Pos2::new(center.x, center.y - half), // top
                Pos2::new(center.x + half, center.y), // right
                Pos2::new(center.x, center.y + half), // bottom
                Pos2::new(center.x - half, center.y), // left
            ],
            closed: true,
            fill,
            stroke: PathStroke::new(stroke.width, stroke.color),
        })
    }
}

// ---------------------------------------------------------------------------
// RectBuilder
// ---------------------------------------------------------------------------

/// Builder for `egui::Shape::Rect`.
#[derive(Debug, Clone)]
pub struct RectBuilder {
    rect: Rect,
    fill: Color32,
    stroke: Stroke,
    rounding: CornerRadius,
}

impl RectBuilder {
    /// Set the fill color.
    #[inline]
    pub fn fill(mut self, color: Color32) -> Self {
        self.fill = color;
        self
    }

    /// Set the stroke.
    #[inline]
    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = stroke;
        self
    }

    /// Set the corner rounding.
    #[inline]
    pub fn rounding(mut self, r: impl Into<CornerRadius>) -> Self {
        self.rounding = r.into();
        self
    }

    /// Build the final [`Shape`].
    #[inline]
    pub fn build(self) -> Shape {
        Shape::Rect(RectShape::new(
            self.rect,
            self.rounding,
            self.fill,
            self.stroke,
            StrokeKind::Outside,
        ))
    }
}

// ---------------------------------------------------------------------------
// CircleBuilder
// ---------------------------------------------------------------------------

/// Builder for `egui::Shape::Circle`.
#[derive(Debug, Clone)]
pub struct CircleBuilder {
    center: Pos2,
    radius: f32,
    fill: Color32,
    stroke: Stroke,
}

impl CircleBuilder {
    /// Set the fill color.
    #[inline]
    pub fn fill(mut self, color: Color32) -> Self {
        self.fill = color;
        self
    }

    /// Set the stroke.
    #[inline]
    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = stroke;
        self
    }

    /// Build the final [`Shape`].
    #[inline]
    pub fn build(self) -> Shape {
        Shape::Circle(egui::epaint::CircleShape {
            center: self.center,
            radius: self.radius,
            fill: self.fill,
            stroke: self.stroke,
        })
    }
}

// ---------------------------------------------------------------------------
// PathBuilder
// ---------------------------------------------------------------------------

/// Builder for `egui::Shape::Path`.
#[derive(Debug, Clone)]
pub struct PathBuilder {
    points: Vec<Pos2>,
    closed: bool,
    fill: Color32,
    stroke: PathStroke,
}

impl PathBuilder {
    /// Mark the path as closed (filled).
    #[inline]
    pub fn closed(mut self) -> Self {
        self.closed = true;
        self
    }

    /// Set the fill color (only meaningful when closed).
    #[inline]
    pub fn fill(mut self, color: Color32) -> Self {
        self.fill = color;
        self
    }

    /// Set the stroke.
    #[inline]
    pub fn stroke(mut self, stroke: PathStroke) -> Self {
        self.stroke = stroke;
        self
    }

    /// Build the final [`Shape`].
    #[inline]
    pub fn build(self) -> Shape {
        Shape::Path(PathShape {
            points: self.points,
            closed: self.closed,
            fill: self.fill,
            stroke: self.stroke,
        })
    }
}

// ─── Shadow & Glow ────────────────────────────────────────────────────────────

/// Direction of a shadow offset.
#[derive(Clone, Copy, Debug)]
pub struct ShadowOffset {
    pub x: f32,
    pub y: f32,
}

impl ShadowOffset {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
    pub fn drop(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Approximate a CSS box-shadow with multiple semi-transparent rects.
/// Returns a `Vec<Shape>` to be added to a painter.
pub fn box_shadow(
    rect: egui::Rect,
    color: egui::Color32,
    blur_radius: f32,
    spread: f32,
    offset: ShadowOffset,
) -> Vec<egui::Shape> {
    let steps = (blur_radius.ceil() as usize).clamp(1, 12);
    let mut shapes = Vec::with_capacity(steps);
    let base_alpha = color.a() as f32 / steps as f32;

    for i in 0..steps {
        let t = i as f32 / steps as f32;
        let expansion = spread + blur_radius * t;
        let alpha = (base_alpha * (1.0 - t * 0.5)) as u8;
        let shadow_color =
            egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha);
        let shadow_rect = egui::Rect::from_min_max(
            egui::Pos2::new(
                rect.min.x - expansion + offset.x,
                rect.min.y - expansion + offset.y,
            ),
            egui::Pos2::new(
                rect.max.x + expansion + offset.x,
                rect.max.y + expansion + offset.y,
            ),
        );
        let rounding = egui::CornerRadius::same((expansion * 0.5).round() as u8);
        shapes.push(egui::Shape::Rect(egui::epaint::RectShape::filled(
            shadow_rect,
            rounding,
            shadow_color,
        )));
    }
    shapes
}

/// Symmetric glow around a rect (no offset, equal spread on all sides).
pub fn glow(rect: egui::Rect, color: egui::Color32, radius: f32) -> Vec<egui::Shape> {
    box_shadow(rect, color, radius, 0.0, ShadowOffset::zero())
}

/// Inner shadow (inset) approximated by drawing a border with gradient-like alpha.
pub fn inner_shadow(rect: egui::Rect, color: egui::Color32, blur_radius: f32) -> Vec<egui::Shape> {
    let steps = (blur_radius.ceil() as usize).clamp(1, 8);
    let mut shapes = Vec::with_capacity(steps * 4);
    let base_alpha = color.a() as f32 / steps as f32;

    for i in 0..steps {
        let t = i as f32 / steps as f32;
        let inset = blur_radius * t;
        let alpha = (base_alpha * (1.0 - t)) as u8;
        let c = egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha);
        let stroke = egui::Stroke::new(1.0, c);
        let inner = egui::Rect::from_min_max(
            egui::Pos2::new(rect.min.x + inset, rect.min.y + inset),
            egui::Pos2::new(rect.max.x - inset, rect.max.y - inset),
        );
        if inner.width() > 0.0 && inner.height() > 0.0 {
            shapes.push(egui::Shape::Rect(egui::epaint::RectShape::stroke(
                inner,
                egui::CornerRadius::ZERO,
                stroke,
                egui::epaint::StrokeKind::Inside,
            )));
        }
    }
    shapes
}

/// Load an image file at runtime and paint it into `rect`.
///
/// This is intended for generated Illustrator preview code where linked raster
/// assets are known only at export time. It returns `false` when the file cannot
/// be read or decoded so generated code can draw a visible fallback instead.
pub fn paint_image_from_path(
    ui: &egui::Ui,
    painter: &egui::Painter,
    rect: egui::Rect,
    path: &str,
    texture_id: &str,
    tint: egui::Color32,
) -> bool {
    let cache_id = egui::Id::new(("egui_expressive_image_texture", texture_id, path));
    let texture = if let Some(texture) = ui
        .ctx()
        .data(|data| data.get_temp::<egui::TextureHandle>(cache_id))
    {
        texture
    } else {
        let path_obj = std::path::Path::new(path);
        let mut bytes = std::fs::read(path_obj);
        if bytes.is_err() {
            if let Some(file_name) = path_obj.file_name() {
                bytes = std::fs::read(file_name);
                if bytes.is_err() {
                    bytes = std::fs::read(std::path::Path::new("generated").join(file_name));
                }
                if bytes.is_err() {
                    bytes = std::fs::read(std::path::Path::new("assets").join(file_name));
                }
                if bytes.is_err() {
                    bytes = std::fs::read(
                        std::path::Path::new("generated")
                            .join("assets")
                            .join(file_name),
                    );
                }
            }
        }
        let Ok(bytes) = bytes else {
            return false;
        };
        let Ok(dynamic_image) = image::load_from_memory(&bytes) else {
            return false;
        };
        let rgba = dynamic_image.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
        let texture = ui
            .ctx()
            .load_texture(texture_id, color_image, egui::TextureOptions::LINEAR);
        ui.ctx()
            .data_mut(|data| data.insert_temp(cache_id, texture.clone()));
        texture
    };

    painter.image(
        texture.id(),
        rect,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        tint,
    );
    true
}

/// Paint a reusable placeholder slot for assets or Illustrator primitives that
/// are intentionally unavailable at runtime.
///
/// This keeps generated exporters and hand-authored egui_expressive code on the
/// same visible fallback primitive instead of duplicating ad-hoc red rectangles
/// in generated files.
pub fn paint_placeholder_slot(
    painter: &egui::Painter,
    rect: egui::Rect,
    fill: egui::Color32,
    stroke: egui::Stroke,
    label: impl AsRef<str>,
) {
    painter.rect_filled(rect, 0.0, fill);
    painter.rect_stroke(rect, 0.0, stroke, egui::StrokeKind::Outside);
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label.as_ref(),
        egui::FontId::proportional(12.0),
        stroke.color,
    );
}

/// Paint an optional image path and draw a shared placeholder when it cannot be
/// loaded.
///
/// Returns `true` when the image was decoded and painted, `false` when the
/// fallback slot was painted. Use this from generated Illustrator code and from
/// code-first egui_expressive UIs that accept user-provided assets.
pub fn paint_image_slot(
    ui: &egui::Ui,
    painter: &egui::Painter,
    rect: egui::Rect,
    path: Option<&str>,
    texture_id: &str,
    tint: egui::Color32,
    fallback_label: &str,
) -> bool {
    if let Some(path) = path.filter(|p| !p.trim().is_empty()) {
        if paint_image_from_path(ui, painter, rect, path, texture_id, tint) {
            return true;
        }
    }

    let alpha = tint.a();
    paint_placeholder_slot(
        painter,
        rect,
        egui::Color32::from_rgba_unmultiplied(255, 0, 0, (30_u16 * alpha as u16 / 255) as u8),
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 0, 0, alpha)),
        fallback_label,
    );
    false
}

// ─── Gradients ────────────────────────────────────────────────────────────────

/// Direction for linear gradients.
#[derive(Clone, Copy, Debug)]
pub enum GradientDir {
    TopToBottom,
    BottomToTop,
    LeftToRight,
    RightToLeft,
    /// Gradient at an arbitrary angle (degrees, CSS convention: 0° = left to right, 90° = top to bottom).
    Angle(f32),
}

/// Build a linear gradient rect as an `egui::Shape::Mesh`.
/// `stops` is a slice of `(position 0.0–1.0, Color32)` pairs.
pub fn linear_gradient_rect(
    rect: egui::Rect,
    stops: &[(f32, egui::Color32)],
    dir: GradientDir,
) -> egui::Shape {
    use egui::epaint::{Mesh, Vertex};

    if stops.is_empty() {
        return egui::Shape::Noop;
    }

    let mut mesh = Mesh::default();

    // Compute diagonal length for angle-based gradients
    let diag_len = (rect.width().powi(2) + rect.height().powi(2)).sqrt();

    // We build a quad strip along the gradient axis.
    // For each stop, we add 2 vertices (one on each side of the rect).
    let n = stops.len();
    for (i, &(t, color)) in stops.iter().enumerate() {
        let t = t.clamp(0.0, 1.0);
        let (p0, p1) = match dir {
            GradientDir::TopToBottom => (
                egui::Pos2::new(rect.min.x, rect.min.y + rect.height() * t),
                egui::Pos2::new(rect.max.x, rect.min.y + rect.height() * t),
            ),
            GradientDir::BottomToTop => (
                egui::Pos2::new(rect.min.x, rect.max.y - rect.height() * t),
                egui::Pos2::new(rect.max.x, rect.max.y - rect.height() * t),
            ),
            GradientDir::LeftToRight => (
                egui::Pos2::new(rect.min.x + rect.width() * t, rect.min.y),
                egui::Pos2::new(rect.min.x + rect.width() * t, rect.max.y),
            ),
            GradientDir::RightToLeft => (
                egui::Pos2::new(rect.max.x - rect.width() * t, rect.min.y),
                egui::Pos2::new(rect.max.x - rect.width() * t, rect.max.y),
            ),
            GradientDir::Angle(deg) => {
                // CSS convention: 0° = left to right, 90° = top to bottom
                let rad = deg.to_radians();
                let (sin_t, cos_t) = rad.sin_cos();

                // Start point at center minus half diagonal in the opposite direction
                let dx = cos_t * diag_len * 0.5;
                let dy = sin_t * diag_len * 0.5;
                let center = rect.center();
                let start = egui::Pos2::new(center.x - dx, center.y - dy);
                let end = egui::Pos2::new(center.x + dx, center.y + dy);

                // Interpolate along the line
                let curr = egui::Pos2::new(
                    start.x + (end.x - start.x) * t,
                    start.y + (end.y - start.y) * t,
                );

                // Perpendicular direction for the strip width
                let perp_dx = -sin_t;
                let perp_dy = cos_t;
                let half_w = diag_len * 0.5;

                (
                    egui::Pos2::new(curr.x - perp_dx * half_w, curr.y - perp_dy * half_w),
                    egui::Pos2::new(curr.x + perp_dx * half_w, curr.y + perp_dy * half_w),
                )
            }
        };
        let uv = egui::epaint::WHITE_UV;
        mesh.vertices.push(Vertex { pos: p0, uv, color });
        mesh.vertices.push(Vertex { pos: p1, uv, color });

        // Add two triangles for each segment (except the last stop)
        if i + 1 < n {
            let base = (i * 2) as u32;
            // Triangle 1: top-left, top-right, bottom-left
            mesh.indices.push(base);
            mesh.indices.push(base + 1);
            mesh.indices.push(base + 2);
            // Triangle 2: top-right, bottom-right, bottom-left
            mesh.indices.push(base + 1);
            mesh.indices.push(base + 3);
            mesh.indices.push(base + 2);
        }
    }

    egui::Shape::Mesh(mesh.into())
}

/// Convenience: two-stop gradient from `top` to `bottom` color.
pub fn gradient_rect(rect: egui::Rect, top: egui::Color32, bottom: egui::Color32) -> egui::Shape {
    linear_gradient_rect(rect, &[(0.0, top), (1.0, bottom)], GradientDir::TopToBottom)
}

/// Build a gradient mesh clipped to a sampled closed path.
///
/// This supports editable Illustrator-style gradient fills for circles, ellipses, and arbitrary
/// closed paths without raster snapshots. Concave polygons are triangulated with ear clipping.
pub fn gradient_path_mesh(
    points: &[egui::Pos2],
    stops: &[(f32, egui::Color32)],
    angle_deg: f32,
    radial: bool,
) -> Option<egui::Shape> {
    gradient_path_mesh_with_geometry(points, stops, angle_deg, radial, None, None, None)
}

/// Build a gradient mesh clipped to a sampled closed path with explicit radial geometry.
///
/// For radial gradients, `center`, `focal_point`, and `radius` preserve Illustrator gradient
/// geometry when available. Missing values fall back to the clipped path bounds.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GradientPathGeometry {
    pub center: Option<egui::Pos2>,
    pub focal_point: Option<egui::Pos2>,
    pub radius: Option<f32>,
    pub transform: Option<Transform2D>,
}

pub fn gradient_path_mesh_with_geometry(
    points: &[egui::Pos2],
    stops: &[(f32, egui::Color32)],
    angle_deg: f32,
    radial: bool,
    center: Option<egui::Pos2>,
    focal_point: Option<egui::Pos2>,
    radius: Option<f32>,
) -> Option<egui::Shape> {
    gradient_path_mesh_with_transform(
        points,
        stops,
        angle_deg,
        radial,
        GradientPathGeometry {
            center,
            focal_point,
            radius,
            transform: None,
        },
    )
}

/// Build a gradient mesh clipped to a sampled closed path with optional gradient transform.
pub fn gradient_path_mesh_with_transform(
    points: &[egui::Pos2],
    stops: &[(f32, egui::Color32)],
    angle_deg: f32,
    radial: bool,
    geometry: GradientPathGeometry,
) -> Option<egui::Shape> {
    if points.len() < 3 || stops.is_empty() {
        return None;
    }

    let mut polygon = points.to_vec();
    if polygon.len() > 3 && polygon.first() == polygon.last() {
        polygon.pop();
    }
    if polygon.len() < 3 {
        return None;
    }

    let inverse_transform = geometry.transform.and_then(|t| t.inverse());
    let sample_point =
        |point: egui::Pos2| inverse_transform.map(|t| t.apply(point)).unwrap_or(point);
    let sample_polygon: Vec<egui::Pos2> = polygon.iter().copied().map(sample_point).collect();

    let display_bounds = bounds_for_points(&polygon);
    let sample_bounds = bounds_for_points(&sample_polygon);
    let display_center = geometry.center.unwrap_or_else(|| display_bounds.center());
    let display_focal_point = geometry.focal_point.unwrap_or(display_center);
    let sample_center = geometry
        .center
        .map(sample_point)
        .unwrap_or_else(|| sample_bounds.center());
    let sample_focal_point = geometry
        .focal_point
        .map(sample_point)
        .unwrap_or(sample_center);
    let angle = angle_deg.to_radians();
    let dir = egui::vec2(angle.cos(), angle.sin());
    let projections: Vec<f32> = sample_polygon
        .iter()
        .map(|p| p.to_vec2().dot(dir))
        .collect();
    let min_proj = projections.iter().copied().fold(f32::INFINITY, f32::min);
    let max_proj = projections
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
    let proj_span = (max_proj - min_proj).max(0.001);
    let max_radius = geometry
        .radius
        .unwrap_or_else(|| {
            polygon
                .iter()
                .map(|p| (sample_point(*p) - sample_center).length())
                .fold(0.0f32, f32::max)
        })
        .max(0.001);
    let indices = triangulate_polygon(&polygon)?;

    let mut mesh = egui::epaint::Mesh::default();
    if radial {
        for triangle in indices.chunks_exact(3) {
            let a = polygon[triangle[0] as usize];
            let b = polygon[triangle[1] as usize];
            let c = polygon[triangle[2] as usize];
            let inner = if point_in_triangle(display_focal_point, a, b, c) {
                display_focal_point
            } else if point_in_triangle(display_center, a, b, c) {
                display_center
            } else {
                egui::pos2((a.x + b.x + c.x) / 3.0, (a.y + b.y + c.y) / 3.0)
            };
            let base = mesh.vertices.len() as u32;
            for point in [a, b, c, inner] {
                mesh.vertices.push(egui::epaint::Vertex {
                    pos: point,
                    uv: egui::epaint::WHITE_UV,
                    color: sample_gradient_color(
                        stops,
                        radial_gradient_t(
                            sample_point(point),
                            sample_center,
                            sample_focal_point,
                            max_radius,
                        ),
                    ),
                });
            }
            mesh.indices.extend_from_slice(&[
                base,
                base + 1,
                base + 3,
                base + 1,
                base + 2,
                base + 3,
                base + 2,
                base,
                base + 3,
            ]);
        }
        return Some(egui::Shape::mesh(mesh));
    }

    for (idx, point) in polygon.iter().enumerate() {
        let t = (projections[idx] - min_proj) / proj_span;
        mesh.vertices.push(egui::epaint::Vertex {
            pos: *point,
            uv: egui::epaint::WHITE_UV,
            color: sample_gradient_color(stops, t),
        });
    }
    mesh.indices = indices;
    Some(egui::Shape::mesh(mesh))
}

fn bounds_for_points(points: &[egui::Pos2]) -> egui::Rect {
    let mut min = points[0];
    let mut max = points[0];
    for point in points.iter().skip(1) {
        min.x = min.x.min(point.x);
        min.y = min.y.min(point.y);
        max.x = max.x.max(point.x);
        max.y = max.y.max(point.y);
    }
    egui::Rect::from_min_max(min, max)
}

fn triangulate_polygon(points: &[egui::Pos2]) -> Option<Vec<u32>> {
    if points.len() < 3 {
        return None;
    }
    let ccw = signed_area(points) > 0.0;
    let mut remaining: Vec<usize> = (0..points.len()).collect();
    let mut indices = Vec::with_capacity((points.len() - 2) * 3);
    let mut guard = 0usize;
    while remaining.len() > 3 && guard < points.len() * points.len() {
        guard += 1;
        let mut clipped = false;
        for i in 0..remaining.len() {
            let prev = remaining[(i + remaining.len() - 1) % remaining.len()];
            let curr = remaining[i];
            let next = remaining[(i + 1) % remaining.len()];
            if !is_convex(points[prev], points[curr], points[next], ccw) {
                continue;
            }
            let contains_point = remaining.iter().any(|&idx| {
                idx != prev
                    && idx != curr
                    && idx != next
                    && point_in_triangle(points[idx], points[prev], points[curr], points[next])
            });
            if contains_point {
                continue;
            }
            if ccw {
                indices.extend_from_slice(&[prev as u32, curr as u32, next as u32]);
            } else {
                indices.extend_from_slice(&[prev as u32, next as u32, curr as u32]);
            }
            remaining.remove(i);
            clipped = true;
            break;
        }
        if !clipped {
            return None;
        }
    }
    if remaining.len() == 3 {
        if ccw {
            indices.extend_from_slice(&[
                remaining[0] as u32,
                remaining[1] as u32,
                remaining[2] as u32,
            ]);
        } else {
            indices.extend_from_slice(&[
                remaining[0] as u32,
                remaining[2] as u32,
                remaining[1] as u32,
            ]);
        }
    }
    Some(indices)
}

fn signed_area(points: &[egui::Pos2]) -> f32 {
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
        .map(|(a, b)| a.x * b.y - b.x * a.y)
        .sum::<f32>()
        * 0.5
}

fn is_convex(a: egui::Pos2, b: egui::Pos2, c: egui::Pos2, ccw: bool) -> bool {
    let cross = cross2(b - a, c - b);
    if ccw {
        cross > 0.0
    } else {
        cross < 0.0
    }
}

fn point_in_triangle(p: egui::Pos2, a: egui::Pos2, b: egui::Pos2, c: egui::Pos2) -> bool {
    let area = |p1: egui::Pos2, p2: egui::Pos2, p3: egui::Pos2| cross2(p2 - p1, p3 - p1);
    let d1 = area(p, a, b);
    let d2 = area(p, b, c);
    let d3 = area(p, c, a);
    let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(has_neg && has_pos)
}

fn cross2(a: egui::Vec2, b: egui::Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

fn radial_gradient_t(
    point: egui::Pos2,
    center: egui::Pos2,
    focal_point: egui::Pos2,
    radius: f32,
) -> f32 {
    let sample = point - focal_point;
    let sample_distance = sample.length();
    if sample_distance <= 0.001 {
        return 0.0;
    }

    let direction = sample / sample_distance;
    let focal_to_center = focal_point - center;
    let b = focal_to_center.dot(direction);
    let c = focal_to_center.dot(focal_to_center) - radius * radius;
    let discriminant = b * b - c;
    if discriminant <= 0.0 {
        return sample_distance / radius.max(0.001);
    }

    let outer_distance = -b + discriminant.sqrt();
    sample_distance / outer_distance.max(0.001)
}

fn sample_gradient_color(stops: &[(f32, egui::Color32)], t: f32) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    let mut sorted = stops.to_vec();
    sorted.sort_by(|a, b| a.0.total_cmp(&b.0));
    if t <= sorted[0].0 {
        return sorted[0].1;
    }
    for pair in sorted.windows(2) {
        let (a_pos, a_color) = pair[0];
        let (b_pos, b_color) = pair[1];
        if t <= b_pos {
            let span = (b_pos - a_pos).max(0.001);
            let local = ((t - a_pos) / span).clamp(0.0, 1.0);
            let [ar, ag, ab, aa] = a_color.to_srgba_unmultiplied();
            let [br, bg, bb, ba] = b_color.to_srgba_unmultiplied();
            let lerp = |x: u8, y: u8| x as f32 + (y as f32 - x as f32) * local;
            return egui::Color32::from_rgba_unmultiplied(
                lerp(ar, br).round() as u8,
                lerp(ag, bg).round() as u8,
                lerp(ab, bb).round() as u8,
                lerp(aa, ba).round() as u8,
            );
        }
    }
    sorted.last().map(|(_, color)| *color).unwrap_or_default()
}

/// Build a bilinear gradient mesh patch as an `egui::Shape::Mesh`.
///
/// This is the code-output primitive for Illustrator-style gradient mesh cells. The corner order is
/// top-left, top-right, bottom-right, bottom-left. Complex Illustrator meshes should be emitted as a
/// sequence of these patches, keeping the call immediate-mode and avoiding raster snapshots.
pub fn mesh_gradient_patch(
    corners: [egui::Pos2; 4],
    colors: [egui::Color32; 4],
    subdivisions: usize,
) -> egui::Shape {
    use egui::epaint::Mesh;

    let subdivisions = subdivisions.clamp(1, 64);
    let stride = subdivisions + 1;
    let mut mesh = Mesh::default();

    for y in 0..=subdivisions {
        let v = y as f32 / subdivisions as f32;
        for x in 0..=subdivisions {
            let u = x as f32 / subdivisions as f32;
            mesh.colored_vertex(bilerp_pos(corners, u, v), bilerp_color(colors, u, v));
        }
    }

    for y in 0..subdivisions {
        for x in 0..subdivisions {
            let a = (y * stride + x) as u32;
            let b = a + 1;
            let c = ((y + 1) * stride + x) as u32;
            let d = c + 1;
            mesh.add_triangle(a, b, c);
            mesh.add_triangle(b, d, c);
        }
    }

    egui::Shape::Mesh(mesh.into())
}

/// Build an editable procedural pattern fill clipped to a sampled path.
///
/// Illustrator pattern swatch internals are not exposed consistently through CEP/UXP, so exporters
/// pass the swatch name as a stable seed and keep the result as vector shapes instead of rasterizing
/// the artboard. The returned shapes are deterministic, editable, and clipped by point sampling to
/// the provided polygon.
pub fn pattern_fill_path(
    points: &[egui::Pos2],
    seed: u32,
    foreground: egui::Color32,
    background: egui::Color32,
    cell_size: f32,
    mark_size: f32,
) -> Vec<egui::Shape> {
    if points.len() < 3 {
        return Vec::new();
    }

    let mut polygon = points.to_vec();
    if polygon.len() > 3 && polygon.first() == polygon.last() {
        polygon.pop();
    }
    if polygon.len() < 3 {
        return Vec::new();
    }

    let bounds = bounds_for_points(&polygon);
    if bounds.is_negative() || bounds.width() <= 0.0 || bounds.height() <= 0.0 {
        return Vec::new();
    }

    let cell_size = cell_size.max(2.0);
    let mark_size = mark_size.clamp(0.5, cell_size * 0.45);
    let stroke = Stroke::new(mark_size.max(0.5), foreground);
    let cols = (bounds.width() / cell_size).ceil() as u32 + 1;
    let rows = (bounds.height() / cell_size).ceil() as u32 + 1;
    let mut shapes = Vec::with_capacity((cols * rows) as usize + 1);

    if background != egui::Color32::TRANSPARENT {
        shapes.push(egui::Shape::Path(PathShape {
            points: polygon.clone(),
            closed: true,
            fill: background,
            stroke: PathStroke::NONE,
        }));
    }

    for row in 0..rows {
        for col in 0..cols {
            let jitter = hash_noise(seed, col, row) as f32 / 255.0 - 0.5;
            let center = egui::pos2(
                bounds.min.x + (col as f32 + 0.5 + jitter * 0.18) * cell_size,
                bounds.min.y + (row as f32 + 0.5 - jitter * 0.18) * cell_size,
            );
            if !point_in_polygon(center, &polygon) {
                continue;
            }

            let half = cell_size * 0.28;
            match (seed.wrapping_add(row).wrapping_add(col)) % 3 {
                0 => {
                    let a = center + egui::vec2(-half, -half);
                    let b = center + egui::vec2(half, half);
                    if point_in_polygon(a, &polygon) && point_in_polygon(b, &polygon) {
                        shapes.push(egui::Shape::line_segment([a, b], stroke));
                    }
                }
                1 => {
                    let a = center + egui::vec2(-half, half);
                    let b = center + egui::vec2(half, -half);
                    if point_in_polygon(a, &polygon) && point_in_polygon(b, &polygon) {
                        shapes.push(egui::Shape::line_segment([a, b], stroke));
                    }
                }
                _ => {
                    let radius = mark_size.max(1.0);
                    let inside = [
                        center + egui::vec2(radius, 0.0),
                        center + egui::vec2(-radius, 0.0),
                        center + egui::vec2(0.0, radius),
                        center + egui::vec2(0.0, -radius),
                    ]
                    .into_iter()
                    .all(|point| point_in_polygon(point, &polygon));
                    if inside {
                        shapes.push(egui::Shape::circle_filled(center, radius, foreground));
                    }
                }
            }
        }
    }

    shapes
}

fn bilerp_pos(corners: [egui::Pos2; 4], u: f32, v: f32) -> egui::Pos2 {
    let top = egui::pos2(
        corners[0].x + (corners[1].x - corners[0].x) * u,
        corners[0].y + (corners[1].y - corners[0].y) * u,
    );
    let bottom = egui::pos2(
        corners[3].x + (corners[2].x - corners[3].x) * u,
        corners[3].y + (corners[2].y - corners[3].y) * u,
    );
    egui::pos2(
        top.x + (bottom.x - top.x) * v,
        top.y + (bottom.y - top.y) * v,
    )
}

fn bilerp_color(colors: [egui::Color32; 4], u: f32, v: f32) -> egui::Color32 {
    let [tl, tr, br, bl] = colors.map(|c| c.to_srgba_unmultiplied());
    let channel = |idx: usize| {
        let top = tl[idx] as f32 + (tr[idx] as f32 - tl[idx] as f32) * u;
        let bottom = bl[idx] as f32 + (br[idx] as f32 - bl[idx] as f32) * u;
        (top + (bottom - top) * v).round().clamp(0.0, 255.0) as u8
    };
    egui::Color32::from_rgba_unmultiplied(channel(0), channel(1), channel(2), channel(3))
}

/// Deterministic procedural noise overlay for code-only Illustrator grain/noise effects.
///
/// This emits tiny translucent rectangles instead of loading a raster texture. It is intended as
/// the CPU/immediate-mode fallback for Illustrator appearance stacks; GPU builds can replace the
/// same semantic primitive with a shader pass.
pub fn noise_rect(rect: egui::Rect, seed: u32, cell_size: f32, opacity: f32) -> Vec<egui::Shape> {
    let cell_size = cell_size.max(1.0);
    let opacity = opacity.clamp(0.0, 1.0);
    if rect.is_negative() || opacity <= 0.0 {
        return Vec::new();
    }

    let cols = (rect.width() / cell_size).ceil().max(1.0) as u32;
    let rows = (rect.height() / cell_size).ceil().max(1.0) as u32;
    let mut shapes = Vec::with_capacity((cols * rows) as usize);

    for row in 0..rows {
        for col in 0..cols {
            let x = rect.min.x + col as f32 * cell_size;
            let y = rect.min.y + row as f32 * cell_size;
            let r = egui::Rect::from_min_max(
                egui::pos2(x, y),
                egui::pos2(
                    (x + cell_size).min(rect.max.x),
                    (y + cell_size).min(rect.max.y),
                ),
            );
            let value = hash_noise(seed, col, row);
            let alpha = (value as f32 / 255.0 * opacity * 255.0).round() as u8;
            shapes.push(egui::Shape::rect_filled(
                r,
                0.0,
                egui::Color32::from_white_alpha(alpha),
            ));
        }
    }

    shapes
}

fn hash_noise(seed: u32, x: u32, y: u32) -> u8 {
    let mut n = seed ^ x.wrapping_mul(0x9E37_79B9) ^ y.wrapping_mul(0x85EB_CA6B);
    n ^= n >> 16;
    n = n.wrapping_mul(0x7FEB_352D);
    n ^= n >> 15;
    n = n.wrapping_mul(0x846C_A68B);
    n ^= n >> 16;
    (n & 0xFF) as u8
}

/// Convert RGB (0.0–1.0) to HSL (hue 0.0–360.0, saturation 0.0–1.0, lightness 0.0–1.0).
fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) * 0.5;
    if (max - min).abs() < 1e-6 {
        return (0.0, 0.0, l);
    }
    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if (max - r).abs() < 1e-6 {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if (max - g).abs() < 1e-6 {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    (h * 60.0, s, l)
}

/// Convert HSL (hue 0.0–360.0, saturation 0.0–1.0, lightness 0.0–1.0) to RGB (0.0–1.0).
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s < 1e-6 {
        return (l, l, l);
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let hue_to_rgb = |p: f32, q: f32, mut t: f32| -> f32 {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            return p + (q - p) * 6.0 * t;
        }
        if t < 0.5 {
            return q;
        }
        if t < 2.0 / 3.0 {
            return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
        }
        p
    };
    let h = h / 360.0;
    (
        hue_to_rgb(p, q, h + 1.0 / 3.0),
        hue_to_rgb(p, q, h),
        hue_to_rgb(p, q, h - 1.0 / 3.0),
    )
}

/// Blend two colors using the specified blend mode.
pub fn blend_color(
    fg: egui::Color32,
    bg: egui::Color32,
    mode: crate::codegen::BlendMode,
) -> egui::Color32 {
    // Unpack as straight (unmultiplied) RGBA so blend math operates on true color values.
    // Color32 stores premultiplied bytes; to_srgba_unmultiplied() reverses that.
    let fg_arr = fg.to_srgba_unmultiplied();
    let bg_arr = bg.to_srgba_unmultiplied();
    let fg = (
        fg_arr[0] as f32 / 255.0,
        fg_arr[1] as f32 / 255.0,
        fg_arr[2] as f32 / 255.0,
        fg_arr[3] as f32 / 255.0,
    );
    let bg = (
        bg_arr[0] as f32 / 255.0,
        bg_arr[1] as f32 / 255.0,
        bg_arr[2] as f32 / 255.0,
        bg_arr[3] as f32 / 255.0,
    );

    let (r, g, b) = match mode {
        crate::codegen::BlendMode::Normal => (fg.0, fg.1, fg.2),
        crate::codegen::BlendMode::Multiply => (bg.0 * fg.0, bg.1 * fg.1, bg.2 * fg.2),
        crate::codegen::BlendMode::Screen => (
            1.0 - (1.0 - bg.0) * (1.0 - fg.0),
            1.0 - (1.0 - bg.1) * (1.0 - fg.1),
            1.0 - (1.0 - bg.2) * (1.0 - fg.2),
        ),
        crate::codegen::BlendMode::Overlay => {
            let blend = |bg: f32, fg: f32| {
                if bg < 0.5 {
                    2.0 * bg * fg
                } else {
                    1.0 - 2.0 * (1.0 - bg) * (1.0 - fg)
                }
            };
            (blend(bg.0, fg.0), blend(bg.1, fg.1), blend(bg.2, fg.2))
        }
        crate::codegen::BlendMode::Darken => (bg.0.min(fg.0), bg.1.min(fg.1), bg.2.min(fg.2)),
        crate::codegen::BlendMode::Lighten => (bg.0.max(fg.0), bg.1.max(fg.1), bg.2.max(fg.2)),
        // Advanced blend modes
        crate::codegen::BlendMode::ColorDodge => (
            if fg.0 >= 1.0 {
                1.0
            } else {
                (bg.0 / (1.0 - fg.0)).min(1.0)
            },
            if fg.1 >= 1.0 {
                1.0
            } else {
                (bg.1 / (1.0 - fg.1)).min(1.0)
            },
            if fg.2 >= 1.0 {
                1.0
            } else {
                (bg.2 / (1.0 - fg.2)).min(1.0)
            },
        ),
        crate::codegen::BlendMode::ColorBurn => (
            if fg.0 <= 0.0 {
                0.0
            } else {
                1.0 - ((1.0 - bg.0) / fg.0).min(1.0)
            },
            if fg.1 <= 0.0 {
                0.0
            } else {
                1.0 - ((1.0 - bg.1) / fg.1).min(1.0)
            },
            if fg.2 <= 0.0 {
                0.0
            } else {
                1.0 - ((1.0 - bg.2) / fg.2).min(1.0)
            },
        ),
        crate::codegen::BlendMode::HardLight => {
            // HardLight = Overlay with fg and bg swapped
            let blend = |fg: f32, bg: f32| {
                if fg < 0.5 {
                    2.0 * fg * bg
                } else {
                    1.0 - 2.0 * (1.0 - fg) * (1.0 - bg)
                }
            };
            (blend(fg.0, bg.0), blend(fg.1, bg.1), blend(fg.2, bg.2))
        }
        crate::codegen::BlendMode::SoftLight => {
            // W3C SoftLight formula
            let blend = |bg: f32, fg: f32| {
                if fg <= 0.5 {
                    bg - (1.0 - 2.0 * fg) * bg * (1.0 - bg)
                } else {
                    let d = if bg <= 0.25 {
                        ((16.0 * bg - 12.0) * bg + 4.0) * bg
                    } else {
                        bg.sqrt()
                    };
                    bg + (2.0 * fg - 1.0) * (d - bg)
                }
            };
            (blend(bg.0, fg.0), blend(bg.1, fg.1), blend(bg.2, fg.2))
        }
        crate::codegen::BlendMode::Difference => (
            (bg.0 - fg.0).abs(),
            (bg.1 - fg.1).abs(),
            (bg.2 - fg.2).abs(),
        ),
        crate::codegen::BlendMode::Exclusion => (
            bg.0 + fg.0 - 2.0 * bg.0 * fg.0,
            bg.1 + fg.1 - 2.0 * bg.1 * fg.1,
            bg.2 + fg.2 - 2.0 * bg.2 * fg.2,
        ),
        crate::codegen::BlendMode::Hue => {
            // Set hue of bg to hue of fg, keep bg saturation and luminosity
            let (fh, _fs, _fl) = rgb_to_hsl(fg.0, fg.1, fg.2);
            let (_bh, bs, bl) = rgb_to_hsl(bg.0, bg.1, bg.2);
            hsl_to_rgb(fh, bs, bl)
        }
        crate::codegen::BlendMode::Saturation => {
            // Set saturation of bg to saturation of fg, keep bg hue and luminosity
            let (_fh, fs, _fl) = rgb_to_hsl(fg.0, fg.1, fg.2);
            let (bh, _bs, bl) = rgb_to_hsl(bg.0, bg.1, bg.2);
            hsl_to_rgb(bh, fs, bl)
        }
        crate::codegen::BlendMode::Color => {
            // Set hue+saturation of bg to fg, keep bg luminosity
            let (fh, fs, _fl) = rgb_to_hsl(fg.0, fg.1, fg.2);
            let (_bh, _bs, bl) = rgb_to_hsl(bg.0, bg.1, bg.2);
            hsl_to_rgb(fh, fs, bl)
        }
        crate::codegen::BlendMode::Luminosity => {
            // Set luminosity of bg to luminosity of fg, keep bg hue+saturation
            let (_fh, _fs, fl) = rgb_to_hsl(fg.0, fg.1, fg.2);
            let (bh, bs, _bl) = rgb_to_hsl(bg.0, bg.1, bg.2);
            hsl_to_rgb(bh, bs, fl)
        }
    };

    // Full W3C Porter-Duff "source over" compositing in straight-alpha space:
    //   co = cs·αs·(1−αb) + αs·αb·B(cb,cs) + cb·αb·(1−αs)
    // where B(cb,cs) = r/g/b from the blend mode above.
    let out_a = fg.3 + bg.3 * (1.0 - fg.3);
    let (r, g, b) = if out_a > 1e-6 {
        let compose = |cs: f32, blend: f32, cb: f32| {
            (cs * fg.3 * (1.0 - bg.3) + fg.3 * bg.3 * blend + cb * bg.3 * (1.0 - fg.3)) / out_a
        };
        (
            compose(fg.0, r, bg.0),
            compose(fg.1, g, bg.1),
            compose(fg.2, b, bg.2),
        )
    } else {
        (0.0, 0.0, 0.0)
    };

    // Convert back to u8
    let r = (r.clamp(0.0, 1.0) * 255.0) as u8;
    let g = (g.clamp(0.0, 1.0) * 255.0) as u8;
    let b = (b.clamp(0.0, 1.0) * 255.0) as u8;
    let a = (out_a.clamp(0.0, 1.0) * 255.0) as u8;

    egui::Color32::from_rgba_unmultiplied(r, g, b, a)
}

// ─── Icon Rendering ───────────────────────────────────────────────────────────

/// Render a single glyph from an icon font (e.g., Phosphor Icons) at `pos`.
///
/// # Usage
/// 1. Load your icon font via `egui::FontDefinitions` and give it a family name.
/// 2. Call `icon(painter, pos, '\u{E000}', 16.0, color, "PhosphorIcons")`.
pub fn icon(
    painter: &egui::Painter,
    pos: egui::Pos2,
    codepoint: char,
    size: f32,
    color: egui::Color32,
    font_family: &str,
) {
    let font_id = egui::FontId::new(size, egui::FontFamily::Name(font_family.into()));
    painter.text(
        pos,
        egui::Align2::CENTER_CENTER,
        codepoint.to_string(),
        font_id,
        color,
    );
}

/// Render a Phosphor-style icon using a built-in path approximation.
/// This works without loading an icon font — uses PathBuilder to draw common shapes.
pub fn icon_play(painter: &egui::Painter, center: egui::Pos2, size: f32, color: egui::Color32) {
    let r = size * 0.4;
    let points = vec![
        egui::Pos2::new(center.x - r * 0.5, center.y - r),
        egui::Pos2::new(center.x + r, center.y),
        egui::Pos2::new(center.x - r * 0.5, center.y + r),
    ];
    painter.add(egui::Shape::convex_polygon(
        points,
        color,
        egui::Stroke::NONE,
    ));
}

pub fn icon_stop(painter: &egui::Painter, center: egui::Pos2, size: f32, color: egui::Color32) {
    let r = size * 0.35;
    let rect = egui::Rect::from_center_size(center, egui::Vec2::splat(r * 2.0));
    painter.add(egui::Shape::Rect(egui::epaint::RectShape::filled(
        rect,
        egui::CornerRadius::ZERO,
        color,
    )));
}

pub fn icon_record(painter: &egui::Painter, center: egui::Pos2, size: f32, color: egui::Color32) {
    painter.circle_filled(center, size * 0.35, color);
}

pub fn icon_loop(painter: &egui::Painter, center: egui::Pos2, size: f32, color: egui::Color32) {
    // Two arrows forming a loop — simplified as two arcs
    let r = size * 0.35;
    let stroke = egui::Stroke::new(size * 0.1, color);
    painter.circle_stroke(center, r, stroke);
}

// ─── Radial Gradient ─────────────────────────────────────────────────────────

/// Direction for radial gradient — center-out or outside-in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RadialGradientDir {
    /// Color at center, fades to edge color.
    CenterOut,
    /// Color at edge, fades to center color.
    EdgeIn,
}

/// Render a radial gradient as a `Shape::Mesh`.
///
/// Approximates a radial gradient using a triangle fan from the center.
/// `segments` controls smoothness (32 is good, 64 is high quality).
pub fn radial_gradient(
    center: egui::Pos2,
    radius: f32,
    inner_color: egui::Color32,
    outer_color: egui::Color32,
    segments: u32,
) -> egui::Shape {
    use egui::{epaint::Mesh, Vec2};
    let mut mesh = Mesh::default();

    // Center vertex
    mesh.colored_vertex(center, inner_color);

    // Ring vertices
    let n = segments.max(8);
    for i in 0..=n {
        let angle = (i as f32 / n as f32) * std::f32::consts::TAU;
        let pos = center + Vec2::new(angle.cos(), angle.sin()) * radius;
        mesh.colored_vertex(pos, outer_color);
    }

    // Triangles: center (0) + consecutive ring pairs
    for i in 0..n {
        mesh.add_triangle(0, i + 1, i + 2);
    }

    egui::Shape::Mesh(std::sync::Arc::new(mesh))
}

/// Radial gradient clipped to a rectangle (elliptical).
pub fn radial_gradient_rect(
    rect: egui::Rect,
    inner_color: egui::Color32,
    outer_color: egui::Color32,
    segments: u32,
) -> egui::Shape {
    let center = rect.center();
    let rx = rect.width() * 0.5;
    let ry = rect.height() * 0.5;
    use egui::epaint::Mesh;
    let mut mesh = Mesh::default();

    mesh.colored_vertex(center, inner_color);

    let n = segments.max(8);
    for i in 0..=n {
        let angle = (i as f32 / n as f32) * std::f32::consts::TAU;
        let pos = center + egui::Vec2::new(angle.cos() * rx, angle.sin() * ry);
        mesh.colored_vertex(pos, outer_color);
    }

    for i in 0..n {
        mesh.add_triangle(0, i + 1, i + 2);
    }

    egui::Shape::Mesh(std::sync::Arc::new(mesh))
}

/// Multi-stop radial gradient clipped to a rectangle (elliptical).
///
/// Unlike [`radial_gradient_rect`], this preserves all Illustrator radial-gradient stops by
/// emitting concentric mesh rings. Stop positions are clamped to `0.0..=1.0`; missing stops produce
/// [`egui::Shape::Noop`].
pub fn radial_gradient_rect_stops(
    rect: egui::Rect,
    stops: &[(f32, egui::Color32)],
    segments: u32,
) -> egui::Shape {
    use egui::epaint::Mesh;

    if stops.is_empty() {
        return egui::Shape::Noop;
    }

    let mut stops = stops.to_vec();
    stops.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    for (pos, _) in &mut stops {
        *pos = pos.clamp(0.0, 1.0);
    }

    let ring_count = stops.len().max(2);
    let segments = segments.max(8);
    let center = rect.center();
    let rx = rect.width() * 0.5;
    let ry = rect.height() * 0.5;
    let mut mesh = Mesh::default();

    for ring in 0..ring_count {
        let t = if ring_count == 1 {
            0.0
        } else {
            ring as f32 / (ring_count - 1) as f32
        };
        let color = sample_stops(&stops, t);

        if ring == 0 {
            mesh.colored_vertex(center, color);
        } else {
            for i in 0..=segments {
                let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
                mesh.colored_vertex(
                    center + egui::vec2(angle.cos() * rx * t, angle.sin() * ry * t),
                    color,
                );
            }
        }
    }

    // Center fan.
    for i in 0..segments {
        mesh.add_triangle(0, i + 1, i + 2);
    }

    // Ring strips.
    let ring_stride = segments + 1;
    for ring in 1..(ring_count - 1) as u32 {
        let inner_start = 1 + (ring - 1) * ring_stride;
        let outer_start = 1 + ring * ring_stride;
        for i in 0..segments {
            let a = inner_start + i;
            let b = inner_start + i + 1;
            let c = outer_start + i;
            let d = outer_start + i + 1;
            mesh.add_triangle(a, b, c);
            mesh.add_triangle(b, d, c);
        }
    }

    egui::Shape::Mesh(std::sync::Arc::new(mesh))
}

fn sample_stops(stops: &[(f32, egui::Color32)], t: f32) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    if stops.len() == 1 || t <= stops[0].0 {
        return stops[0].1;
    }
    for pair in stops.windows(2) {
        let (a_t, a) = pair[0];
        let (b_t, b) = pair[1];
        if t <= b_t {
            let local = if (b_t - a_t).abs() < f32::EPSILON {
                0.0
            } else {
                (t - a_t) / (b_t - a_t)
            };
            return lerp_color(a, b, local);
        }
    }
    stops
        .last()
        .map(|(_, c)| *c)
        .unwrap_or(egui::Color32::TRANSPARENT)
}

fn lerp_color(a: egui::Color32, b: egui::Color32, t: f32) -> egui::Color32 {
    let a = a.to_srgba_unmultiplied();
    let b = b.to_srgba_unmultiplied();
    let channel = |idx: usize| (a[idx] as f32 + (b[idx] as f32 - a[idx] as f32) * t).round() as u8;
    egui::Color32::from_rgba_unmultiplied(channel(0), channel(1), channel(2), channel(3))
}

// ─── Scan Lines & Overlays ───────────────────────────────────────────────────

/// Render a CRT-style scan line overlay over a rect.
///
/// Draws alternating semi-transparent horizontal lines.
/// `line_height` is the height of each scan line pair (default 2.0).
/// `alpha` controls darkness (0.0 = invisible, 1.0 = fully black lines).
pub fn scan_lines(rect: egui::Rect, line_height: f32, alpha: f32) -> Vec<egui::Shape> {
    let color = egui::Color32::from_black_alpha((alpha * 80.0).clamp(0.0, 255.0) as u8);
    let lh = line_height.max(1.0);
    let mut shapes = Vec::new();
    let mut y = rect.min.y;
    while y < rect.max.y {
        let line_rect = egui::Rect::from_min_max(
            egui::Pos2::new(rect.min.x, y),
            egui::Pos2::new(rect.max.x, (y + lh * 0.5).min(rect.max.y)),
        );
        shapes.push(egui::Shape::rect_filled(line_rect, 0.0, color));
        y += lh;
    }
    shapes
}

/// Render a dot-matrix / halftone overlay over a rect.
///
/// Draws a grid of small semi-transparent dots.
pub fn dot_matrix(
    rect: egui::Rect,
    dot_spacing: f32,
    dot_radius: f32,
    color: egui::Color32,
) -> Vec<egui::Shape> {
    let spacing = dot_spacing.max(2.0);
    let mut shapes = Vec::new();
    let mut y = rect.min.y + spacing * 0.5;
    while y < rect.max.y {
        let mut x = rect.min.x + spacing * 0.5;
        while x < rect.max.x {
            shapes.push(egui::Shape::circle_filled(
                egui::Pos2::new(x, y),
                dot_radius,
                color,
            ));
            x += spacing;
        }
        y += spacing;
    }
    shapes
}

/// Render a vignette effect (dark edges, bright center) over a rect.
pub fn vignette(rect: egui::Rect, color: egui::Color32, strength: f32) -> egui::Shape {
    // Approximate with a radial gradient from transparent center to colored edge
    let alpha = (strength * 200.0).clamp(0.0, 255.0) as u8;
    let edge_color = egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha);
    radial_gradient_rect(rect, egui::Color32::TRANSPARENT, edge_color, 48)
}

// ─── Rich Stroke & Dashed Paths ───────────────────────────────────────────────

/// Stroke cap style.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StrokeCap {
    Butt,
    Round,
    Square,
}

/// Stroke join style.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StrokeJoin {
    Miter,
    Round,
    Bevel,
}

/// Dash pattern for dashed strokes.
#[derive(Clone, Debug)]
pub struct DashPattern {
    pub dashes: Vec<f32>,
    pub offset: f32,
}

/// Rich stroke with support for dashes, caps, and joins.
#[derive(Clone, Debug)]
pub struct RichStroke {
    pub width: f32,
    pub color: egui::Color32,
    pub dash: Option<DashPattern>,
    pub cap: StrokeCap,
    pub join: StrokeJoin,
}

impl RichStroke {
    /// Create a solid (non-dashed) stroke.
    pub fn solid(width: f32, color: egui::Color32) -> Self {
        Self {
            width,
            color,
            dash: None,
            cap: StrokeCap::Butt,
            join: StrokeJoin::Miter,
        }
    }

    /// Create a dashed stroke with equal dash and gap lengths.
    pub fn dashed(width: f32, color: egui::Color32, dash: f32, gap: f32) -> Self {
        Self {
            width,
            color,
            dash: Some(DashPattern {
                dashes: vec![dash, gap],
                offset: 0.0,
            }),
            cap: StrokeCap::Butt,
            join: StrokeJoin::Miter,
        }
    }
}

/// Render a path with a rich stroke (supports dashes).
pub fn dashed_path(painter: &egui::Painter, points: &[Pos2], stroke: &RichStroke) {
    for shape in dashed_path_shapes(points, stroke) {
        painter.add(shape);
    }
}

pub fn dashed_path_shapes(points: &[Pos2], stroke: &RichStroke) -> Vec<egui::Shape> {
    let mut shapes = Vec::new();
    if points.len() < 2 {
        return shapes;
    }
    match &stroke.dash {
        None => {
            draw_dash(&mut shapes, points, stroke);
        }
        Some(pattern) => {
            let total_len: f32 = points.windows(2).map(|w| (w[1] - w[0]).length()).sum();
            if total_len <= 0.0 {
                return shapes;
            }
            let cycle_len: f32 = pattern.dashes.iter().sum();
            if cycle_len <= 0.0 {
                return shapes;
            }

            let mut dist = pattern.offset % cycle_len;
            let mut phase = 0usize;
            let mut drawing = true;

            let mut d = dist;
            while d >= pattern.dashes[phase] {
                d -= pattern.dashes[phase];
                phase = (phase + 1) % pattern.dashes.len();
                drawing = !drawing;
            }
            dist = d;

            let mut current_dash = Vec::new();
            let mut current_pos = points[0];

            if drawing {
                current_dash.push(current_pos);
            }

            for i in 0..points.len() - 1 {
                let seg_vec = points[i + 1] - points[i];
                let seg_len = seg_vec.length();
                if seg_len <= 0.0 {
                    continue;
                }
                let seg_dir = seg_vec / seg_len;

                let mut walked = 0.0f32;
                while walked < seg_len {
                    let remaining_in_phase = pattern.dashes[phase] - dist;
                    let step = remaining_in_phase.min(seg_len - walked);
                    let next_pos = points[i] + seg_dir * (walked + step);

                    current_pos = next_pos;
                    walked += step;
                    dist += step;

                    if dist >= pattern.dashes[phase] {
                        if drawing {
                            current_dash.push(current_pos);
                            draw_dash(&mut shapes, &current_dash, stroke);
                            current_dash.clear();
                        }
                        dist = 0.0;
                        phase = (phase + 1) % pattern.dashes.len();
                        drawing = !drawing;
                        if drawing {
                            current_dash.push(current_pos);
                        }
                    } else if walked >= seg_len && drawing {
                        current_dash.push(current_pos);
                    }
                }
            }
            if drawing && current_dash.len() > 1 {
                draw_dash(&mut shapes, &current_dash, stroke);
            }
        }
    }
    shapes
}

fn draw_dash(shapes: &mut Vec<egui::Shape>, dash_points: &[Pos2], stroke: &RichStroke) {
    if dash_points.len() < 2 {
        return;
    }
    if stroke.join == StrokeJoin::Bevel && dash_points.len() > 2 {
        for pair in dash_points.windows(2) {
            shapes.push(egui::Shape::line_segment(
                [pair[0], pair[1]],
                Stroke::new(stroke.width, stroke.color),
            ));
        }
    } else {
        shapes.push(egui::Shape::line(
            dash_points.to_vec(),
            Stroke::new(stroke.width, stroke.color),
        ));
    }

    if stroke.cap == StrokeCap::Round {
        shapes.push(egui::Shape::circle_filled(
            dash_points[0],
            stroke.width * 0.5,
            stroke.color,
        ));
        shapes.push(egui::Shape::circle_filled(
            *dash_points.last().unwrap(),
            stroke.width * 0.5,
            stroke.color,
        ));
    } else if stroke.cap == StrokeCap::Square {
        let d0 = dash_points[1] - dash_points[0];
        let len0 = d0.length();
        if len0 > 0.0 {
            let dir0 = d0 / len0;
            let p0 = dash_points[0] - dir0 * (stroke.width * 0.5);
            shapes.push(egui::Shape::line_segment(
                [p0, dash_points[0]],
                Stroke::new(stroke.width, stroke.color),
            ));
        }
        let n = dash_points.len();
        let d1 = dash_points[n - 1] - dash_points[n - 2];
        let len1 = d1.length();
        if len1 > 0.0 {
            let dir1 = d1 / len1;
            let p1 = dash_points[n - 1] + dir1 * (stroke.width * 0.5);
            shapes.push(egui::Shape::line_segment(
                [dash_points[n - 1], p1],
                Stroke::new(stroke.width, stroke.color),
            ));
        }
    }

    if stroke.join == StrokeJoin::Round {
        for &p in &dash_points[1..dash_points.len() - 1] {
            shapes.push(egui::Shape::circle_filled(
                p,
                stroke.width * 0.5,
                stroke.color,
            ));
        }
    }
}

// ─── 2D Transform ─────────────────────────────────────────────────────────────

pub fn rounded_rect_path(rect: egui::Rect, rounding: f32) -> Vec<egui::Pos2> {
    let mut points = Vec::new();
    let r = rounding.min(rect.width() * 0.5).min(rect.height() * 0.5);
    if r <= 0.0 {
        return vec![
            rect.min,
            egui::pos2(rect.max.x, rect.min.y),
            rect.max,
            egui::pos2(rect.min.x, rect.max.y),
        ];
    }
    let n = adaptive_arc_segments(r);
    let c = egui::pos2(rect.max.x - r, rect.min.y + r);
    for i in 0..=n {
        let a = std::f32::consts::FRAC_PI_2 * (i as f32 / n as f32);
        points.push(c + egui::vec2(a.sin() * r, -a.cos() * r));
    }
    let c = egui::pos2(rect.max.x - r, rect.max.y - r);
    for i in 0..=n {
        let a = std::f32::consts::FRAC_PI_2 * (i as f32 / n as f32);
        points.push(c + egui::vec2(a.cos() * r, a.sin() * r));
    }
    let c = egui::pos2(rect.min.x + r, rect.max.y - r);
    for i in 0..=n {
        let a = std::f32::consts::FRAC_PI_2 * (i as f32 / n as f32);
        points.push(c + egui::vec2(-a.sin() * r, a.cos() * r));
    }
    let c = egui::pos2(rect.min.x + r, rect.min.y + r);
    for i in 0..=n {
        let a = std::f32::consts::FRAC_PI_2 * (i as f32 / n as f32);
        points.push(c + egui::vec2(-a.cos() * r, -a.sin() * r));
    }
    points
}

fn adaptive_arc_segments(radius: f32) -> usize {
    ((radius * std::f32::consts::FRAC_PI_2) / 3.0)
        .ceil()
        .clamp(8.0, 32.0) as usize
}

/// 2D affine transform (SVG matrix convention: [a, b, c, d, e, f]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform2D {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl Transform2D {
    /// Identity transform.
    pub fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    /// Create a translation transform.
    pub fn translate(x: f32, y: f32) -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: x,
            f: y,
        }
    }

    /// Create a rotation transform (angle in degrees).
    pub fn rotate(angle_deg: f32) -> Self {
        let r = angle_deg.to_radians();
        let (s, c) = r.sin_cos();
        Self {
            a: c,
            b: s,
            c: -s,
            d: c,
            e: 0.0,
            f: 0.0,
        }
    }

    /// Create a rotation around a center point (angle in degrees).
    pub fn rotate_around(angle_deg: f32, center: Pos2) -> Self {
        Self::translate(-center.x, -center.y)
            .then(Self::rotate(angle_deg))
            .then(Self::translate(center.x, center.y))
    }

    /// Create a scale transform.
    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            a: sx,
            b: 0.0,
            c: 0.0,
            d: sy,
            e: 0.0,
            f: 0.0,
        }
    }

    /// Compose two transforms (first apply self, then other).
    pub fn then(self, other: Self) -> Self {
        Self {
            a: other.a * self.a + other.c * self.b,
            b: other.b * self.a + other.d * self.b,
            c: other.a * self.c + other.c * self.d,
            d: other.b * self.c + other.d * self.d,
            e: other.a * self.e + other.c * self.f + other.e,
            f: other.b * self.e + other.d * self.f + other.f,
        }
    }

    /// Invert the transform, returning `None` for singular matrices.
    pub fn inverse(self) -> Option<Self> {
        let det = self.a * self.d - self.b * self.c;
        if det.abs() <= 0.000001 {
            return None;
        }
        let inv_det = 1.0 / det;
        Some(Self {
            a: self.d * inv_det,
            b: -self.b * inv_det,
            c: -self.c * inv_det,
            d: self.a * inv_det,
            e: (self.c * self.f - self.d * self.e) * inv_det,
            f: (self.b * self.e - self.a * self.f) * inv_det,
        })
    }

    /// Apply transform to a point.
    pub fn apply(&self, p: Pos2) -> Pos2 {
        Pos2::new(
            self.a * p.x + self.c * p.y + self.e,
            self.b * p.x + self.d * p.y + self.f,
        )
    }

    /// Apply transform to a shape.
    pub fn apply_to_shape(&self, shape: Shape) -> Shape {
        transform_shape(shape, self)
    }

    /// Apply transform to a rect, returning the axis-aligned bounding box of the transformed corners.
    pub fn apply_to_rect(&self, rect: egui::Rect) -> egui::Rect {
        let corners = [
            self.apply(rect.min),
            self.apply(egui::Pos2::new(rect.max.x, rect.min.y)),
            self.apply(rect.max),
            self.apply(egui::Pos2::new(rect.min.x, rect.max.y)),
        ];
        let min = corners.iter().fold(corners[0], |a, &b| {
            egui::Pos2::new(a.x.min(b.x), a.y.min(b.y))
        });
        let max = corners.iter().fold(corners[0], |a, &b| {
            egui::Pos2::new(a.x.max(b.x), a.y.max(b.y))
        });
        egui::Rect::from_min_max(min, max)
    }
}

/// Apply a 2D affine transform to all points in a shape.
pub fn transform_shape(shape: Shape, t: &Transform2D) -> Shape {
    match shape {
        Shape::Vec(shapes) => {
            Shape::Vec(shapes.into_iter().map(|s| transform_shape(s, t)).collect())
        }
        Shape::Path(mut p) => {
            p.points = p.points.into_iter().map(|pt| t.apply(pt)).collect();
            Shape::Path(p)
        }
        Shape::Circle(mut c) => {
            c.center = t.apply(c.center);
            Shape::Circle(c)
        }
        Shape::Rect(mut r) => {
            // Transform all 4 corners, take bounding box
            let corners = [
                t.apply(r.rect.min),
                t.apply(Pos2::new(r.rect.max.x, r.rect.min.y)),
                t.apply(r.rect.max),
                t.apply(Pos2::new(r.rect.min.x, r.rect.max.y)),
            ];
            let min_x = corners.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
            let min_y = corners.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
            let max_x = corners
                .iter()
                .map(|p| p.x)
                .fold(f32::NEG_INFINITY, f32::max);
            let max_y = corners
                .iter()
                .map(|p| p.y)
                .fold(f32::NEG_INFINITY, f32::max);
            r.rect = Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y));
            Shape::Rect(r)
        }
        Shape::LineSegment {
            points: [a, b],
            stroke,
        } => Shape::LineSegment {
            points: [t.apply(a), t.apply(b)],
            stroke,
        },
        Shape::Mesh(mut m) => {
            let m_mut = std::sync::Arc::make_mut(&mut m);
            for v in &mut m_mut.vertices {
                v.pos = t.apply(v.pos);
            }
            Shape::Mesh(m)
        }
        other => other,
    }
}

// ─── Clipped Rounded Rect ─────────────────────────────────────────────────────

/// Render content clipped to a rounded rectangle.
///
/// Note: egui's native clip system only supports rectangular clip rects, so
/// true rounded clipping isn't possible. The `rounding` parameter is accepted
/// for API compatibility but has no effect on the clipping shape. Content is
/// clipped to the rectangular intersection of the clip rect and the given rect.
pub fn clipped_to_bounding_rect(
    ui: &mut egui::Ui,
    rect: Rect,
    _rounding: f32,
    content: impl FnOnce(&mut egui::Ui),
) {
    let clip = ui.painter().clip_rect().intersect(rect);
    let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(rect));
    child_ui.set_clip_rect(clip);
    content(&mut child_ui);
}

pub fn clipped_rounded_rect(
    ui: &mut egui::Ui,
    rect: Rect,
    rounding: f32,
    content: impl FnOnce(&mut egui::Ui),
) {
    clipped_to_bounding_rect(ui, rect, rounding, content)
}

// ---------------------------------------------------------------------------
// ClipScope — clip children to shape bounds/approximations
// ---------------------------------------------------------------------------

/// Shape to clip content to.
#[derive(Clone, Debug)]
pub enum ClipShape {
    /// Axis-aligned rectangle.
    Rect(egui::Rect),
    /// Rounded rectangle.
    RoundedRect(egui::Rect, egui::CornerRadius),
    /// Circle.
    Circle(egui::Pos2, f32),
}

impl ClipShape {
    /// Convert to an axis-aligned bounding rect for egui's clip system.
    pub fn bounding_rect(&self) -> egui::Rect {
        match self {
            ClipShape::Rect(r) => *r,
            ClipShape::RoundedRect(r, _) => *r,
            ClipShape::Circle(center, radius) => {
                egui::Rect::from_center_size(*center, egui::Vec2::splat(radius * 2.0))
            }
        }
    }
}

/// Clip all content drawn inside the closure to the given shape.
///
/// For `Rect` and `RoundedRect`, uses egui's native clip rect.
/// For `Circle`, clips to the bounding rect (egui limitation — true circular
/// clipping requires GPU stencil which egui doesn't expose).
///
/// # Example
/// ```rust,ignore
/// // Clip an image to a circle (approximate — clips to bounding square)
/// clip_to_bounding_rect(ui, ClipShape::Circle(center, 32.0), |ui| {
///     ui.image(texture_id, Vec2::splat(64.0));
/// });
///
/// // Clip content to a rounded card
/// clip_to_bounding_rect(ui, ClipShape::RoundedRect(card_rect, CornerRadius::same(8)), |ui| {
///     ui.label("Clipped content");
/// });
/// ```
pub fn clip_to_bounding_rect(
    ui: &mut egui::Ui,
    shape: ClipShape,
    content: impl FnOnce(&mut egui::Ui),
) -> egui::Response {
    let clip_rect = shape.bounding_rect();
    let old_clip = ui.clip_rect();
    let new_clip = old_clip.intersect(clip_rect);
    ui.set_clip_rect(new_clip);
    let response = ui.scope(content).response;
    ui.set_clip_rect(old_clip);
    response
}

pub fn clip_to(
    ui: &mut egui::Ui,
    shape: ClipShape,
    content: impl FnOnce(&mut egui::Ui),
) -> egui::Response {
    clip_to_bounding_rect(ui, shape, content)
}

// ---------------------------------------------------------------------------
// OpacityScope — fade an entire subtree
// ---------------------------------------------------------------------------

/// Apply opacity to all content drawn inside the closure.
///
/// Opacity is approximated by multiplying the alpha channel of all shapes
/// painted during the closure. This works for most cases but does not
/// affect textures/images (egui limitation).
///
/// # Example
/// ```rust,ignore
/// // Fade out a disabled panel
/// with_opacity(ui, if enabled { 1.0 } else { 0.4 }, |ui| {
///     ui.label("This content fades when disabled");
///     ui.button("Faded button");
/// });
/// ```
pub fn with_opacity(
    ui: &mut egui::Ui,
    opacity: f32,
    content: impl FnOnce(&mut egui::Ui),
) -> egui::Response {
    if (opacity - 1.0).abs() < 0.001 {
        return ui.scope(content).response;
    }
    let opacity = opacity.clamp(0.0, 1.0);
    ui.scope(|ui| {
        ui.multiply_opacity(opacity);
        content(ui);
    })
    .response
}

// ---------------------------------------------------------------------------
// ZStack — overlapping layout (simple, correct version)
// ---------------------------------------------------------------------------

/// A layout container where children overlap, sized to the largest child.
///
/// Children are drawn in order (first = bottom, last = top).
/// The ZStack allocates space equal to its largest child.
///
/// # Example
/// ```rust,ignore
/// // Badge on an icon
/// zstack(ui, |ui| {
///     ui.image(icon, Vec2::splat(32.0));
/// }, |ui| {
///     // This overlaps the icon
///     ui.label("3");
/// });
/// ```
///
/// For more layers, nest zstack calls or use `put()` directly.
pub fn zstack<R>(
    ui: &mut egui::Ui,
    background: impl FnOnce(&mut egui::Ui) -> R,
    foreground: impl FnOnce(&mut egui::Ui),
) -> R {
    // Paint background layer, record its rect
    let bg_response = ui.scope(|ui| background(ui));
    let rect = bg_response.response.rect;

    // Paint foreground layer at the same position using put()
    ui.put(rect, |ui: &mut egui::Ui| {
        foreground(ui);
        ui.allocate_rect(rect, egui::Sense::hover())
    });

    bg_response.inner
}

/// A boxed layer closure for use with [`zstack_layers`].
pub type LayerFn = Box<dyn FnOnce(&mut egui::Ui)>;

/// Stack multiple layers at the same position.
/// Each layer is a closure that receives a `&mut Ui` positioned at `rect`.
/// The rect is determined by the first layer.
pub fn zstack_layers(ui: &mut egui::Ui, layers: Vec<LayerFn>) -> egui::Response {
    if layers.is_empty() {
        return ui.allocate_rect(egui::Rect::NOTHING, egui::Sense::hover());
    }

    let mut layers = layers;
    let first = layers.remove(0);
    let bg = ui.scope(first);
    let rect = bg.response.rect;

    for layer in layers {
        ui.put(rect, |ui: &mut egui::Ui| {
            layer(ui);
            ui.allocate_rect(rect, egui::Sense::hover())
        });
    }

    bg.response
}

// ============================================================================
// Blend Layer Compositing
// ============================================================================

/// A layer of shapes to be composited with a specific blend mode and opacity.
///
/// Used with [`composite_layers`] to combine multiple layers using
/// Photoshop/Illustrator-style blend modes via CPU-side compositing.
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

type RasterizedBlendGroup = (egui::Rect, [u32; 2], Vec<egui::Color32>, Vec<egui::Shape>);

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
        if polygon.len() >= 3 {
            self.clip_polygons.push(polygon);
        }
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
    let Some((rect, size, pixels, unhandled)) = rasterize_composited_layers(&layers) else {
        for layer in layers {
            for shape in layer.shapes {
                ui.painter().add(shape);
            }
        }
        return;
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
    let Some((rect, size, pixels, unhandled)) = rasterize_composited_layers(&layers) else {
        composite_layers(ui, layers);
        return;
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
    let Some((rect, size, mut pixels, unhandled)) = rasterize_composited_layers(&layers) else {
        return;
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

fn rasterize_composited_layers(layers: &[BlendLayer]) -> Option<RasterizedBlendGroup> {
    let rect = layers_bounds(layers)?;
    let width = (rect.width().ceil() as u32).clamp(1, 4096);
    let height = (rect.height().ceil() as u32).clamp(1, 4096);
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
            return None;
        }
        for polygon in &layer.clip_polygons {
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

    Some((rect, [width, height], composited, unhandled))
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

fn fill_rect_shape_pixels(
    rect_shape: &egui::epaint::RectShape,
    origin: egui::Pos2,
    width: u32,
    height: u32,
    pixels: &mut [egui::Color32],
) {
    if rect_shape.fill == egui::Color32::TRANSPARENT {
        return;
    }
    if rect_shape.corner_radius == egui::CornerRadius::ZERO && rect_shape.angle.abs() <= 0.0001 {
        fill_rect_pixels(
            rect_shape.rect,
            origin,
            width,
            height,
            rect_shape.fill,
            pixels,
        );
        return;
    }
    let points =
        rounded_rect_shape_path(rect_shape.rect, rect_shape.corner_radius, rect_shape.angle);
    fill_polygon_pixels(&points, origin, width, height, rect_shape.fill, pixels);
}

fn stroke_rect_shape_pixels(
    rect_shape: &egui::epaint::RectShape,
    origin: egui::Pos2,
    width: u32,
    height: u32,
    pixels: &mut [egui::Color32],
) {
    if rect_shape.stroke.is_empty() {
        return;
    }
    let stroke_width = rect_shape.stroke.width.max(1.0);
    let center_outset = rect_stroke_center_outset(rect_shape.stroke_kind, stroke_width);
    let rect = rect_shape.rect.expand(center_outset);
    if !rect.is_positive() {
        return;
    }
    let radius = (rect_shape.corner_radius.average() + center_outset).max(0.0);
    let mut points = rounded_rect_path(rect, radius);
    if rect_shape.angle.abs() > 0.0001 {
        let transform =
            Transform2D::rotate_around(rect_shape.angle.to_degrees(), rect_shape.rect.center());
        for point in &mut points {
            *point = transform.apply(*point);
        }
    }
    stroke_polyline_pixels(
        &points,
        true,
        origin,
        width,
        height,
        rect_shape.stroke.width,
        rect_shape.stroke.color,
        pixels,
    );
}

fn rounded_rect_shape_path(
    rect: egui::Rect,
    corner_radius: egui::CornerRadius,
    angle_rad: f32,
) -> Vec<egui::Pos2> {
    let mut points = rounded_rect_path(rect, corner_radius.average());
    if angle_rad.abs() > 0.0001 {
        let transform = Transform2D::rotate_around(angle_rad.to_degrees(), rect.center());
        for point in &mut points {
            *point = transform.apply(*point);
        }
    }
    points
}

fn rect_stroke_center_outset(stroke_kind: egui::StrokeKind, stroke_width: f32) -> f32 {
    match stroke_kind {
        egui::StrokeKind::Inside => -stroke_width * 0.5,
        egui::StrokeKind::Middle => 0.0,
        egui::StrokeKind::Outside => stroke_width * 0.5,
    }
}

fn fill_rect_pixels(
    rect: egui::Rect,
    origin: egui::Pos2,
    width: u32,
    height: u32,
    color: egui::Color32,
    pixels: &mut [egui::Color32],
) {
    if color == egui::Color32::TRANSPARENT {
        return;
    }
    let x0 = (rect.min.x - origin.x).floor().max(0.0) as u32;
    let y0 = (rect.min.y - origin.y).floor().max(0.0) as u32;
    let x1 = (rect.max.x - origin.x).ceil().min(width as f32) as u32;
    let y1 = (rect.max.y - origin.y).ceil().min(height as f32) as u32;
    for y in y0..y1 {
        for x in x0..x1 {
            pixels[(y * width + x) as usize] = blend_color(
                color,
                pixels[(y * width + x) as usize],
                crate::codegen::BlendMode::Normal,
            );
        }
    }
}

fn fill_circle_pixels(
    center: egui::Pos2,
    radius: f32,
    origin: egui::Pos2,
    width: u32,
    height: u32,
    color: egui::Color32,
    pixels: &mut [egui::Color32],
) {
    if color == egui::Color32::TRANSPARENT || radius <= 0.0 {
        return;
    }
    let r2 = radius * radius;
    let rect = egui::Rect::from_center_size(center, egui::vec2(radius * 2.0, radius * 2.0));
    let x0 = (rect.min.x - origin.x).floor().max(0.0) as u32;
    let y0 = (rect.min.y - origin.y).floor().max(0.0) as u32;
    let x1 = (rect.max.x - origin.x).ceil().min(width as f32) as u32;
    let y1 = (rect.max.y - origin.y).ceil().min(height as f32) as u32;
    for y in y0..y1 {
        for x in x0..x1 {
            let p = egui::pos2(origin.x + x as f32 + 0.5, origin.y + y as f32 + 0.5);
            if p.distance_sq(center) <= r2 {
                let idx = (y * width + x) as usize;
                pixels[idx] = blend_color(color, pixels[idx], crate::codegen::BlendMode::Normal);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn stroke_circle_pixels(
    center: egui::Pos2,
    radius: f32,
    origin: egui::Pos2,
    width: u32,
    height: u32,
    stroke_width: f32,
    color: egui::Color32,
    pixels: &mut [egui::Color32],
) {
    if color == egui::Color32::TRANSPARENT || radius <= 0.0 || stroke_width <= 0.0 {
        return;
    }
    let half = stroke_width.max(1.0) * 0.5;
    let outer = radius + half;
    let inner = (radius - half).max(0.0);
    let outer2 = outer * outer;
    let inner2 = inner * inner;
    let rect = egui::Rect::from_center_size(center, egui::vec2(outer * 2.0, outer * 2.0));
    let x0 = (rect.min.x - origin.x).floor().max(0.0) as u32;
    let y0 = (rect.min.y - origin.y).floor().max(0.0) as u32;
    let x1 = (rect.max.x - origin.x).ceil().min(width as f32) as u32;
    let y1 = (rect.max.y - origin.y).ceil().min(height as f32) as u32;
    for y in y0..y1 {
        for x in x0..x1 {
            let p = egui::pos2(origin.x + x as f32 + 0.5, origin.y + y as f32 + 0.5);
            let d2 = p.distance_sq(center);
            if d2 <= outer2 && d2 >= inner2 {
                let idx = (y * width + x) as usize;
                pixels[idx] = blend_color(color, pixels[idx], crate::codegen::BlendMode::Normal);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn fill_ellipse_pixels(
    center: egui::Pos2,
    radius: egui::Vec2,
    angle: f32,
    origin: egui::Pos2,
    width: u32,
    height: u32,
    color: egui::Color32,
    pixels: &mut [egui::Color32],
) {
    if color == egui::Color32::TRANSPARENT || radius.x <= 0.0 || radius.y <= 0.0 {
        return;
    }
    let Some(bounds) = ellipse_bounds(center, radius, angle, 0.0) else {
        return;
    };
    let x0 = (bounds.min.x - origin.x).floor().max(0.0) as u32;
    let y0 = (bounds.min.y - origin.y).floor().max(0.0) as u32;
    let x1 = (bounds.max.x - origin.x).ceil().min(width as f32) as u32;
    let y1 = (bounds.max.y - origin.y).ceil().min(height as f32) as u32;
    for y in y0..y1 {
        for x in x0..x1 {
            let p = egui::pos2(origin.x + x as f32 + 0.5, origin.y + y as f32 + 0.5);
            if point_in_ellipse(p, center, radius, angle) {
                let idx = (y * width + x) as usize;
                pixels[idx] = blend_color(color, pixels[idx], crate::codegen::BlendMode::Normal);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn stroke_ellipse_pixels(
    center: egui::Pos2,
    radius: egui::Vec2,
    angle: f32,
    origin: egui::Pos2,
    width: u32,
    height: u32,
    stroke_width: f32,
    color: egui::Color32,
    pixels: &mut [egui::Color32],
) {
    if color == egui::Color32::TRANSPARENT
        || radius.x <= 0.0
        || radius.y <= 0.0
        || stroke_width <= 0.0
    {
        return;
    }
    let half = stroke_width.max(1.0) * 0.5;
    let outer_radius = egui::vec2(radius.x + half, radius.y + half);
    let inner_radius = egui::vec2((radius.x - half).max(0.0), (radius.y - half).max(0.0));
    let Some(bounds) = ellipse_bounds(center, outer_radius, angle, 0.0) else {
        return;
    };
    let x0 = (bounds.min.x - origin.x).floor().max(0.0) as u32;
    let y0 = (bounds.min.y - origin.y).floor().max(0.0) as u32;
    let x1 = (bounds.max.x - origin.x).ceil().min(width as f32) as u32;
    let y1 = (bounds.max.y - origin.y).ceil().min(height as f32) as u32;
    for y in y0..y1 {
        for x in x0..x1 {
            let p = egui::pos2(origin.x + x as f32 + 0.5, origin.y + y as f32 + 0.5);
            if point_in_ellipse(p, center, outer_radius, angle)
                && !point_in_ellipse(p, center, inner_radius, angle)
            {
                let idx = (y * width + x) as usize;
                pixels[idx] = blend_color(color, pixels[idx], crate::codegen::BlendMode::Normal);
            }
        }
    }
}

fn ellipse_bounds(
    center: egui::Pos2,
    radius: egui::Vec2,
    angle: f32,
    outset: f32,
) -> Option<egui::Rect> {
    let rx = radius.x.abs() + outset;
    let ry = radius.y.abs() + outset;
    if rx <= 0.0 || ry <= 0.0 {
        return None;
    }
    let mut points = Vec::with_capacity(64);
    let (sin, cos) = angle.sin_cos();
    for idx in 0..64 {
        let theta = std::f32::consts::TAU * idx as f32 / 64.0;
        let local = egui::vec2(theta.cos() * rx, theta.sin() * ry);
        points.push(
            center + egui::vec2(cos * local.x - sin * local.y, sin * local.x + cos * local.y),
        );
    }
    bounds_from_points(&points)
}

fn point_in_ellipse(point: egui::Pos2, center: egui::Pos2, radius: egui::Vec2, angle: f32) -> bool {
    let rx = radius.x.abs();
    let ry = radius.y.abs();
    if rx <= 0.0001 || ry <= 0.0001 {
        return false;
    }
    let v = point - center;
    let (sin, cos) = (-angle).sin_cos();
    let local = egui::vec2(cos * v.x - sin * v.y, sin * v.x + cos * v.y);
    let nx = local.x / rx;
    let ny = local.y / ry;
    nx * nx + ny * ny <= 1.0
}

#[allow(clippy::too_many_arguments)]
fn stroke_polyline_pixels(
    points: &[egui::Pos2],
    closed: bool,
    origin: egui::Pos2,
    width: u32,
    height: u32,
    stroke_width: f32,
    color: egui::Color32,
    pixels: &mut [egui::Color32],
) {
    if points.len() < 2 {
        return;
    }
    for segment in points.windows(2) {
        stroke_line_pixels(
            segment[0],
            segment[1],
            origin,
            width,
            height,
            stroke_width,
            color,
            pixels,
        );
    }
    if closed {
        stroke_line_pixels(
            *points.last().unwrap(),
            points[0],
            origin,
            width,
            height,
            stroke_width,
            color,
            pixels,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn stroke_line_pixels(
    a: egui::Pos2,
    b: egui::Pos2,
    origin: egui::Pos2,
    width: u32,
    height: u32,
    stroke_width: f32,
    color: egui::Color32,
    pixels: &mut [egui::Color32],
) {
    if color == egui::Color32::TRANSPARENT || stroke_width <= 0.0 {
        return;
    }
    let Some(bounds) = bounds_from_points(&[a, b]).map(|r| r.expand(stroke_width.max(1.0))) else {
        return;
    };
    let x0 = (bounds.min.x - origin.x).floor().max(0.0) as u32;
    let y0 = (bounds.min.y - origin.y).floor().max(0.0) as u32;
    let x1 = (bounds.max.x - origin.x).ceil().min(width as f32) as u32;
    let y1 = (bounds.max.y - origin.y).ceil().min(height as f32) as u32;
    let ab = b - a;
    let len2 = ab.length_sq();
    if len2 <= 0.0001 {
        fill_circle_pixels(a, stroke_width * 0.5, origin, width, height, color, pixels);
        return;
    }
    let radius = stroke_width.max(1.0) * 0.5;
    let radius2 = radius * radius;
    for y in y0..y1 {
        for x in x0..x1 {
            let p = egui::pos2(origin.x + x as f32 + 0.5, origin.y + y as f32 + 0.5);
            let t = ((p - a).dot(ab) / len2).clamp(0.0, 1.0);
            let closest = a + ab * t;
            if p.distance_sq(closest) <= radius2 {
                let idx = (y * width + x) as usize;
                pixels[idx] = blend_color(color, pixels[idx], crate::codegen::BlendMode::Normal);
            }
        }
    }
}

fn fill_polygon_pixels(
    points: &[egui::Pos2],
    origin: egui::Pos2,
    width: u32,
    height: u32,
    color: egui::Color32,
    pixels: &mut [egui::Color32],
) {
    if points.len() < 3 || color == egui::Color32::TRANSPARENT {
        return;
    }
    let Some(bounds) = bounds_from_points(points) else {
        return;
    };
    let x0 = (bounds.min.x - origin.x).floor().max(0.0) as u32;
    let y0 = (bounds.min.y - origin.y).floor().max(0.0) as u32;
    let x1 = (bounds.max.x - origin.x).ceil().min(width as f32) as u32;
    let y1 = (bounds.max.y - origin.y).ceil().min(height as f32) as u32;
    for y in y0..y1 {
        for x in x0..x1 {
            let p = egui::pos2(origin.x + x as f32 + 0.5, origin.y + y as f32 + 0.5);
            if point_in_polygon(p, points) {
                let idx = (y * width + x) as usize;
                pixels[idx] = blend_color(color, pixels[idx], crate::codegen::BlendMode::Normal);
            }
        }
    }
}

fn rasterize_mesh_pixels(
    mesh: &egui::epaint::Mesh,
    origin: egui::Pos2,
    width: u32,
    height: u32,
    pixels: &mut [egui::Color32],
) {
    for tri in mesh.indices.chunks_exact(3) {
        let a = &mesh.vertices[tri[0] as usize];
        let b = &mesh.vertices[tri[1] as usize];
        let c = &mesh.vertices[tri[2] as usize];
        rasterize_triangle_pixels(
            [a.pos, b.pos, c.pos],
            [a.color, b.color, c.color],
            origin,
            width,
            height,
            pixels,
        );
    }
}

fn rasterize_triangle_pixels(
    points: [egui::Pos2; 3],
    colors: [egui::Color32; 3],
    origin: egui::Pos2,
    width: u32,
    height: u32,
    pixels: &mut [egui::Color32],
) {
    let Some(bounds) = bounds_from_points(&points) else {
        return;
    };
    let x0 = (bounds.min.x - origin.x).floor().max(0.0) as u32;
    let y0 = (bounds.min.y - origin.y).floor().max(0.0) as u32;
    let x1 = (bounds.max.x - origin.x).ceil().min(width as f32) as u32;
    let y1 = (bounds.max.y - origin.y).ceil().min(height as f32) as u32;
    let denom = cross2(points[1] - points[0], points[2] - points[0]);
    if denom.abs() <= 0.0001 {
        return;
    }
    let channels = colors.map(|color| color.to_srgba_unmultiplied());
    for y in y0..y1 {
        for x in x0..x1 {
            let p = egui::pos2(origin.x + x as f32 + 0.5, origin.y + y as f32 + 0.5);
            let w0 = cross2(points[1] - p, points[2] - p) / denom;
            let w1 = cross2(points[2] - p, points[0] - p) / denom;
            let w2 = 1.0 - w0 - w1;
            if w0 >= -0.001 && w1 >= -0.001 && w2 >= -0.001 {
                let channel = |idx: usize| {
                    (channels[0][idx] as f32 * w0
                        + channels[1][idx] as f32 * w1
                        + channels[2][idx] as f32 * w2)
                        .round()
                        .clamp(0.0, 255.0) as u8
                };
                let color = egui::Color32::from_rgba_unmultiplied(
                    channel(0),
                    channel(1),
                    channel(2),
                    channel(3),
                );
                let idx = (y * width + x) as usize;
                pixels[idx] = blend_color(color, pixels[idx], crate::codegen::BlendMode::Normal);
            }
        }
    }
}

fn point_in_polygon(p: egui::Pos2, polygon: &[egui::Pos2]) -> bool {
    let mut inside = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        let pi = polygon[i];
        let pj = polygon[j];
        if ((pi.y > p.y) != (pj.y > p.y))
            && (p.x < (pj.x - pi.x) * (p.y - pi.y) / (pj.y - pi.y) + pi.x)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn apply_polygon_alpha_mask(
    pixels: &mut [egui::Color32],
    width: u32,
    height: u32,
    origin: egui::Pos2,
    polygon: &[egui::Pos2],
) {
    for y in 0..height {
        for x in 0..width {
            let p = egui::pos2(origin.x + x as f32 + 0.5, origin.y + y as f32 + 0.5);
            if !point_in_polygon(p, polygon) {
                pixels[(y * width + x) as usize] = egui::Color32::TRANSPARENT;
            }
        }
    }
}

fn color_with_opacity(color: egui::Color32, opacity: f32) -> egui::Color32 {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    egui::Color32::from_rgba_unmultiplied(
        r,
        g,
        b,
        (a as f32 * opacity.clamp(0.0, 1.0)).round() as u8,
    )
}

#[cfg(feature = "wgpu")]
fn pixels_to_rgba(pixels: &[egui::Color32]) -> Vec<u8> {
    pixels
        .iter()
        .flat_map(|p| p.to_srgba_unmultiplied())
        .collect()
}

fn blend_layers_hash(layers: &[BlendLayer], pixels: &[egui::Color32]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    layers.len().hash(&mut hasher);
    for layer in layers {
        layer.blend_mode.hash(&mut hasher);
        layer.opacity.to_bits().hash(&mut hasher);
        layer.shapes.len().hash(&mut hasher);
        layer.clip_polygons.len().hash(&mut hasher);
        for polygon in &layer.clip_polygons {
            for point in polygon {
                point.x.to_bits().hash(&mut hasher);
                point.y.to_bits().hash(&mut hasher);
            }
        }
    }
    for p in pixels {
        p.hash(&mut hasher);
    }
    hasher.finish()
}

fn polygon_hash(points: &[egui::Pos2]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for p in points {
        p.x.to_bits().hash(&mut hasher);
        p.y.to_bits().hash(&mut hasher);
    }
    hasher.finish()
}

// ============================================================================
// Clipping Mask Support
// ============================================================================

/// Paint content clipped to a convex polygon using a bounding-box scissor approximation.
///
/// This function clips content to the **axis-aligned bounding box** of the polygon,
/// then paints background-colored triangles over the corners to approximate the
/// polygon boundary. This approach is correct only when:
/// - The polygon is convex
/// - The background behind the clip region is a uniform color matching `ui.visuals().window_fill()`
///
/// For non-uniform backgrounds, concave polygons, or layered scenes, prefer
/// [`clipped_layers_gpu`], which masks the composited layer group per pixel and
/// presents it through the egui-wgpu callback path when available.
///
/// # Arguments
/// * `ui` — the egui UI context
/// * `clip_polygon` — convex polygon vertices (clockwise or counter-clockwise)
/// * `content` — closure that paints the clipped content
pub fn clipped_shape_approx(
    ui: &mut egui::Ui,
    clip_polygon: &[egui::Pos2],
    _clip_closed: bool,
    content: impl FnOnce(&mut egui::Ui),
) {
    if clip_polygon.len() < 3 {
        // Degenerate: just paint without clipping
        content(ui);
        return;
    }
    // Compute bounding box of the clip polygon
    let min_x = clip_polygon
        .iter()
        .map(|p| p.x)
        .fold(f32::INFINITY, f32::min);
    let min_y = clip_polygon
        .iter()
        .map(|p| p.y)
        .fold(f32::INFINITY, f32::min);
    let max_x = clip_polygon
        .iter()
        .map(|p| p.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let max_y = clip_polygon
        .iter()
        .map(|p| p.y)
        .fold(f32::NEG_INFINITY, f32::max);

    if min_x >= max_x || min_y >= max_y {
        return;
    }

    let clip_rect = egui::Rect::from_min_max(egui::pos2(min_x, min_y), egui::pos2(max_x, max_y));

    // Use rectangular scissor for the bounding box
    let painter = ui.painter().with_clip_rect(clip_rect);
    let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(clip_rect));
    child_ui.set_clip_rect(clip_rect);

    // Paint the content (clipped to bbox of polygon)
    content(&mut child_ui);

    // Paint corner-covering triangles in the background color to approximate
    // the polygon clip for convex shapes (e.g. rounded rects, hexagons)
    let bg = ui.visuals().window_fill();
    let n = clip_polygon.len();
    for i in 0..n {
        let a = clip_polygon[i];
        let b = clip_polygon[(i + 1) % n];
        // For each edge, paint the "outside" triangle between the edge and the bbox corner
        // This is a best-effort approximation for convex polygons
        let corner = nearest_bbox_corner(a, b, clip_rect);
        if let Some(corner_pt) = corner {
            painter.add(egui::Shape::convex_polygon(
                vec![a, b, corner_pt],
                bg,
                egui::Stroke::NONE,
            ));
        }
    }
}

pub fn clipped_shape(
    ui: &mut egui::Ui,
    clip_polygon: &[egui::Pos2],
    clip_closed: bool,
    content: impl FnOnce(&mut egui::Ui),
) {
    clipped_shape_approx(ui, clip_polygon, clip_closed, content)
}

/// Paint content clipped to a polygon-shaped region using a background-dependent alpha-mask approximation.
///
/// Requires the `clip-mask` feature flag (`tiny-skia` dependency).
/// When the feature is not enabled, falls back to `clipped_shape_approx` (bbox approximation).
///
/// # Limitations
/// This is **not** true arbitrary clipping. It works by painting an inverted mask overlay
/// using the current `ui.visuals().window_fill()` color. It will look incorrect if the
/// background behind the clipped shape is not a solid color matching `window_fill()`,
/// or if placed over layered/non-flat scenes.
///
/// # How it works
/// 1. Builds a tiny-skia alpha mask from the clip polygon
/// 2. Clips content to the bounding box via `set_clip_rect`
/// 3. Overlays the inverted mask (in background color) to hide regions outside the polygon
#[cfg(feature = "clip-mask")]
pub fn clipped_shape_cpu(
    ui: &mut egui::Ui,
    clip_polygon: &[egui::Pos2],
    content: impl FnOnce(&mut egui::Ui),
) {
    use tiny_skia::{FillRule, Paint as SkPaint, PathBuilder, Pixmap, Transform as SkTransform};

    if clip_polygon.len() < 3 {
        content(ui);
        return;
    }

    let min_x = clip_polygon
        .iter()
        .map(|p| p.x)
        .fold(f32::INFINITY, f32::min);
    let min_y = clip_polygon
        .iter()
        .map(|p| p.y)
        .fold(f32::INFINITY, f32::min);
    let max_x = clip_polygon
        .iter()
        .map(|p| p.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let max_y = clip_polygon
        .iter()
        .map(|p| p.y)
        .fold(f32::NEG_INFINITY, f32::max);

    if min_x >= max_x || min_y >= max_y {
        return;
    }

    let w = (max_x - min_x).ceil() as u32;
    let h = (max_y - min_y).ceil() as u32;
    if w == 0 || h == 0 {
        return;
    }

    // Build tiny-skia mask pixmap
    let mut mask_pixmap = match Pixmap::new(w, h) {
        Some(p) => p,
        None => {
            clipped_shape_approx(ui, clip_polygon, true, content);
            return;
        }
    };

    let mut pb = PathBuilder::new();
    let first = clip_polygon[0];
    pb.move_to(first.x - min_x, first.y - min_y);
    for pt in &clip_polygon[1..] {
        pb.line_to(pt.x - min_x, pt.y - min_y);
    }
    pb.close();

    if let Some(path) = pb.finish() {
        let mut paint = SkPaint::default();
        paint.set_color_rgba8(255, 255, 255, 255);
        mask_pixmap.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            SkTransform::identity(),
            None,
        );
    }

    // Paint content clipped to bbox
    let clip_rect = egui::Rect::from_min_max(egui::pos2(min_x, min_y), egui::pos2(max_x, max_y));
    let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(clip_rect));
    child_ui.set_clip_rect(clip_rect);
    content(&mut child_ui);

    // Build inverted mask overlay: background color where polygon is absent
    let bg = ui.visuals().window_fill();
    let mask_pixels: Vec<egui::Color32> = mask_pixmap
        .pixels()
        .iter()
        .map(|px| {
            let a = px.alpha();
            egui::Color32::from_rgba_unmultiplied(bg.r(), bg.g(), bg.b(), 255u8.saturating_sub(a))
        })
        .collect();

    let mask_image = egui::ColorImage {
        size: [w as usize, h as usize],
        pixels: mask_pixels,
        source_size: egui::Vec2::new(w as f32, h as f32),
    };

    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for p in clip_polygon {
        p.x.to_bits().hash(&mut hasher);
        p.y.to_bits().hash(&mut hasher);
    }
    let hash = hasher.finish();
    let texture_name = format!("__egui_expressive_clip_mask_{:x}_{}x{}", hash, w, h);

    let texture = ui
        .ctx()
        .load_texture(texture_name, mask_image, egui::TextureOptions::NEAREST);

    let painter = ui.painter().with_clip_rect(clip_rect);
    painter.image(
        texture.id(),
        clip_rect,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );
}

/// Find the nearest bounding box corner to the midpoint of edge (a, b),
/// on the outside of the polygon. Returns None if the edge is axis-aligned
/// (no corner masking needed).
fn nearest_bbox_corner(a: egui::Pos2, b: egui::Pos2, bbox: egui::Rect) -> Option<egui::Pos2> {
    let mid = egui::pos2((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
    // Determine which bbox corner is farthest from the midpoint (outside the polygon)
    let corners = [
        bbox.min,
        egui::pos2(bbox.max.x, bbox.min.y),
        bbox.max,
        egui::pos2(bbox.min.x, bbox.max.y),
    ];
    // Only return a corner if it's clearly outside the edge
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    // Edge normal (pointing outward for CW polygon)
    let nx = dy;
    let ny = -dx;
    corners
        .iter()
        .filter(|&&c| {
            // Corner is on the outside (positive normal side)
            let dot = (c.x - mid.x) * nx + (c.y - mid.y) * ny;
            dot > 1.0
        })
        .copied()
        .next()
}

/// Alignment for stacked/layered content within a bounding rect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StackAlign {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl StackAlign {
    /// Convert to egui's `Align2`.
    pub fn to_align2(self) -> egui::Align2 {
        match self {
            Self::TopLeft => egui::Align2::LEFT_TOP,
            Self::TopCenter => egui::Align2::CENTER_TOP,
            Self::TopRight => egui::Align2::RIGHT_TOP,
            Self::CenterLeft => egui::Align2::LEFT_CENTER,
            Self::Center => egui::Align2::CENTER_CENTER,
            Self::CenterRight => egui::Align2::RIGHT_CENTER,
            Self::BottomLeft => egui::Align2::LEFT_BOTTOM,
            Self::BottomCenter => egui::Align2::CENTER_BOTTOM,
            Self::BottomRight => egui::Align2::RIGHT_BOTTOM,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::BlendMode;

    fn opaque(r: u8, g: u8, b: u8) -> egui::Color32 {
        egui::Color32::from_rgb(r, g, b)
    }

    #[test]
    fn test_mesh_gradient_patch_generates_subdivided_mesh() {
        let shape = mesh_gradient_patch(
            [
                egui::pos2(0.0, 0.0),
                egui::pos2(10.0, 0.0),
                egui::pos2(10.0, 10.0),
                egui::pos2(0.0, 10.0),
            ],
            [
                egui::Color32::RED,
                egui::Color32::GREEN,
                egui::Color32::BLUE,
                egui::Color32::WHITE,
            ],
            2,
        );

        let egui::Shape::Mesh(mesh) = shape else {
            panic!("expected mesh shape");
        };
        assert_eq!(mesh.vertices.len(), 9);
        assert_eq!(mesh.indices.len(), 24);
        assert_eq!(mesh.vertices[0].pos, egui::pos2(0.0, 0.0));
        assert_eq!(mesh.vertices[8].pos, egui::pos2(10.0, 10.0));
    }

    #[test]
    fn test_noise_rect_is_deterministic_and_subdivided() {
        let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(4.0, 4.0));
        let a = noise_rect(rect, 42, 2.0, 0.5);
        let b = noise_rect(rect, 42, 2.0, 0.5);
        assert_eq!(a.len(), 4);
        assert_eq!(format!("{:?}", a), format!("{:?}", b));
    }

    #[test]
    fn test_radial_gradient_rect_stops_preserves_multiple_rings() {
        let shape = radial_gradient_rect_stops(
            egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(10.0, 10.0)),
            &[
                (0.0, egui::Color32::RED),
                (0.5, egui::Color32::GREEN),
                (1.0, egui::Color32::BLUE),
            ],
            8,
        );
        let egui::Shape::Mesh(mesh) = shape else {
            panic!("expected mesh shape");
        };
        assert!(mesh.vertices.len() > 10);
        assert!(mesh.indices.len() > 24);
    }

    #[test]
    fn test_radial_gradient_path_mesh_has_inner_stop_vertex() {
        let points = vec![
            egui::pos2(0.0, 0.0),
            egui::pos2(10.0, 0.0),
            egui::pos2(10.0, 10.0),
            egui::pos2(0.0, 10.0),
        ];
        let shape = gradient_path_mesh(
            &points,
            &[(0.0, egui::Color32::RED), (1.0, egui::Color32::BLUE)],
            0.0,
            true,
        )
        .expect("radial path mesh");
        let egui::Shape::Mesh(mesh) = shape else {
            panic!("expected mesh shape");
        };
        assert!(mesh.vertices.len() > points.len());
        assert!(mesh
            .vertices
            .iter()
            .any(|v| v.pos == egui::pos2(5.0, 5.0) && v.color == egui::Color32::RED));
    }

    #[test]
    fn test_radial_gradient_path_mesh_uses_explicit_focal_point_and_radius() {
        let points = vec![
            egui::pos2(0.0, 0.0),
            egui::pos2(10.0, 0.0),
            egui::pos2(10.0, 10.0),
            egui::pos2(0.0, 10.0),
        ];
        let focal = egui::pos2(3.0, 4.0);
        let shape = gradient_path_mesh_with_geometry(
            &points,
            &[(0.0, egui::Color32::RED), (1.0, egui::Color32::BLUE)],
            0.0,
            true,
            Some(egui::pos2(2.0, 2.0)),
            Some(focal),
            Some(20.0),
        )
        .expect("radial path mesh");
        let egui::Shape::Mesh(mesh) = shape else {
            panic!("expected mesh shape");
        };
        assert!(mesh
            .vertices
            .iter()
            .any(|v| v.pos == focal && v.color == egui::Color32::RED));
    }

    #[test]
    fn test_radial_gradient_t_uses_centered_outer_circle() {
        let t = radial_gradient_t(
            egui::pos2(11.0, 5.0),
            egui::pos2(5.0, 5.0),
            egui::pos2(7.0, 5.0),
            10.0,
        );
        assert!((t - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_transform_inverse_roundtrip() {
        let transform = Transform2D::translate(3.0, -2.0).then(Transform2D::scale(2.0, 4.0));
        let inverse = transform.inverse().expect("invertible transform");
        let point = egui::pos2(7.0, 11.0);
        let roundtrip = inverse.apply(transform.apply(point));
        assert!((roundtrip.x - point.x).abs() < 0.001);
        assert!((roundtrip.y - point.y).abs() < 0.001);
    }

    #[test]
    fn test_blend_color_normal() {
        // Normal: result is fg (fully opaque)
        let result = blend_color(opaque(200, 100, 50), opaque(50, 50, 50), BlendMode::Normal);
        let [r, g, b, _] = result.to_srgba_unmultiplied();
        assert_eq!(r, 200);
        assert_eq!(g, 100);
        assert_eq!(b, 50);
    }

    #[test]
    fn test_blend_color_multiply() {
        // Multiply: white * white = white
        let result = blend_color(
            opaque(255, 255, 255),
            opaque(255, 255, 255),
            BlendMode::Multiply,
        );
        let [r, _g, _b, _] = result.to_srgba_unmultiplied();
        assert_eq!(r, 255);
        // Multiply: black * anything = black
        let result2 = blend_color(opaque(0, 0, 0), opaque(200, 100, 50), BlendMode::Multiply);
        let [r2, g2, b2, _] = result2.to_srgba_unmultiplied();
        assert_eq!(r2, 0);
        assert_eq!(g2, 0);
        assert_eq!(b2, 0);
    }

    #[test]
    fn test_blend_color_screen() {
        // Screen: black screen anything = anything
        let result = blend_color(opaque(0, 0, 0), opaque(200, 100, 50), BlendMode::Screen);
        let [r, g, _b, _] = result.to_srgba_unmultiplied();
        assert!((r as i32 - 200).abs() <= 2, "r={}", r);
        assert!((g as i32 - 100).abs() <= 2, "g={}", g);
        // Screen: white screen anything = white
        let result2 = blend_color(
            opaque(255, 255, 255),
            opaque(100, 100, 100),
            BlendMode::Screen,
        );
        let [r2, _, _, _] = result2.to_srgba_unmultiplied();
        assert_eq!(r2, 255);
    }

    #[test]
    fn test_blend_color_difference() {
        // Difference: same color = black
        let result = blend_color(
            opaque(100, 100, 100),
            opaque(100, 100, 100),
            BlendMode::Difference,
        );
        let [r, g, b, _] = result.to_srgba_unmultiplied();
        assert!(
            r <= 2 && g <= 2 && b <= 2,
            "expected near-black, got ({},{},{})",
            r,
            g,
            b
        );
    }

    #[test]
    fn test_blend_color_exclusion() {
        // Exclusion: same color = near-black (2*c*(1-c) subtracted)
        let result = blend_color(
            opaque(128, 128, 128),
            opaque(128, 128, 128),
            BlendMode::Exclusion,
        );
        let [r, _, _, _] = result.to_srgba_unmultiplied();
        // 0.5 + 0.5 - 2*0.5*0.5 = 0.5 → ~128
        assert!((r as i32 - 128).abs() <= 3, "r={}", r);
    }

    #[test]
    fn test_blend_color_hsl_modes_no_panic() {
        // HSL modes should not panic for any input
        for mode in [
            BlendMode::Hue,
            BlendMode::Saturation,
            BlendMode::Color,
            BlendMode::Luminosity,
        ] {
            let _ = blend_color(opaque(200, 100, 50), opaque(50, 150, 200), mode);
        }
    }

    #[test]
    fn test_blend_color_color_dodge_white_fg() {
        // ColorDodge: white fg → white result
        let result = blend_color(
            opaque(255, 255, 255),
            opaque(100, 100, 100),
            BlendMode::ColorDodge,
        );
        let [r, g, b, _] = result.to_srgba_unmultiplied();
        assert_eq!(r, 255);
        assert_eq!(g, 255);
        assert_eq!(b, 255);
    }

    #[test]
    fn test_blend_color_hard_light() {
        // HardLight with black fg → black result (2*0*bg = 0)
        let result = blend_color(opaque(0, 0, 0), opaque(200, 100, 50), BlendMode::HardLight);
        let [r, g, b, _] = result.to_srgba_unmultiplied();
        assert!(
            r <= 2 && g <= 2 && b <= 2,
            "expected near-black, got ({},{},{})",
            r,
            g,
            b
        );
    }

    #[test]
    fn test_composite_layers_empty() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                composite_layers(ui, vec![]);
            });
        });
    }

    #[test]
    fn test_composite_layers_behavior() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let shape1 = egui::Shape::Rect(egui::epaint::RectShape::filled(
                    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(10.0)),
                    egui::CornerRadius::ZERO,
                    egui::Color32::RED,
                ));
                let shape2 = egui::Shape::Rect(egui::epaint::RectShape::filled(
                    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(10.0)),
                    egui::CornerRadius::ZERO,
                    egui::Color32::BLUE,
                ));
                let layer1 = BlendLayer::new(vec![shape1])
                    .blend_mode(BlendMode::Normal)
                    .opacity(1.0);
                let layer2 = BlendLayer::new(vec![shape2])
                    .blend_mode(BlendMode::Multiply)
                    .opacity(0.5);

                composite_layers(ui, vec![layer1, layer2]);
            });
        });
    }

    #[test]
    fn test_rasterize_composited_layers_per_pixel_blend() {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(2.0));
        let red = egui::Shape::Rect(egui::epaint::RectShape::filled(
            rect,
            egui::CornerRadius::ZERO,
            egui::Color32::RED,
        ));
        let blue = egui::Shape::Rect(egui::epaint::RectShape::filled(
            rect,
            egui::CornerRadius::ZERO,
            egui::Color32::BLUE,
        ));
        let (_, size, pixels, unhandled) = rasterize_composited_layers(&[
            BlendLayer::new(vec![red]),
            BlendLayer::new(vec![blue]).blend_mode(BlendMode::Multiply),
        ])
        .expect("layers rasterize");
        assert_eq!(size, [2, 2]);
        assert!(unhandled.is_empty());
        let [r, g, b, a] = pixels[0].to_srgba_unmultiplied();
        assert_eq!((r, g, b, a), (0, 0, 0, 255));
    }

    #[test]
    fn test_rasterize_composited_layers_handles_ellipse() {
        let ellipse = egui::Shape::ellipse_filled(
            egui::pos2(5.0, 3.0),
            egui::vec2(5.0, 3.0),
            egui::Color32::RED,
        );
        let (_, size, pixels, unhandled) =
            rasterize_composited_layers(&[BlendLayer::new(vec![ellipse])])
                .expect("ellipse rasterizes");
        assert_eq!(size, [10, 6]);
        assert!(unhandled.is_empty());
        assert_ne!(
            pixels[(3 * size[0] + 5) as usize],
            egui::Color32::TRANSPARENT
        );
    }

    #[test]
    fn test_rasterize_composited_layers_preserves_rounded_rect_corners() {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(10.0));
        let rounded = egui::Shape::Rect(egui::epaint::RectShape::filled(
            rect,
            egui::CornerRadius::same(5),
            egui::Color32::RED,
        ));
        let (_, size, pixels, unhandled) =
            rasterize_composited_layers(&[BlendLayer::new(vec![rounded])])
                .expect("rect rasterizes");
        assert_eq!(size, [10, 10]);
        assert!(unhandled.is_empty());
        assert_eq!(pixels[0], egui::Color32::TRANSPARENT);
        assert_ne!(
            pixels[(5 * size[0] + 5) as usize],
            egui::Color32::TRANSPARENT
        );
    }

    #[test]
    fn test_rasterize_composited_layers_uses_stroke_aware_bounds() {
        let circle = egui::Shape::circle_stroke(
            egui::pos2(5.0, 5.0),
            5.0,
            egui::Stroke::new(4.0, egui::Color32::RED),
        );
        let (_, size, _, unhandled) = rasterize_composited_layers(&[BlendLayer::new(vec![circle])])
            .expect("stroke rasterizes");
        assert_eq!(size, [14, 14]);
        assert!(unhandled.is_empty());
    }

    #[test]
    fn test_polygon_alpha_mask_clears_outside_pixels() {
        let mut pixels = vec![egui::Color32::WHITE; 4];
        apply_polygon_alpha_mask(
            &mut pixels,
            2,
            2,
            egui::Pos2::ZERO,
            &[
                egui::pos2(0.0, 0.0),
                egui::pos2(1.0, 0.0),
                egui::pos2(1.0, 1.0),
                egui::pos2(0.0, 1.0),
            ],
        );
        assert!(pixels.contains(&egui::Color32::TRANSPARENT));
        assert!(pixels.contains(&egui::Color32::WHITE));
    }

    #[test]
    fn test_clipped_shape_approx_empty() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                clipped_shape_approx(ui, &[], true, |_| {});
            });
        });
    }

    #[cfg(feature = "clip-mask")]
    #[test]
    fn test_clipped_shape_cpu_empty() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                clipped_shape_cpu(ui, &[], |_| {});
            });
        });
    }

    #[cfg(feature = "clip-mask")]
    #[test]
    fn test_clipped_shape_cpu_behavior() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let polygon1 = vec![
                    egui::pos2(0.0, 0.0),
                    egui::pos2(10.0, 0.0),
                    egui::pos2(10.0, 10.0),
                ];
                let polygon2 = vec![
                    egui::pos2(10.0, 10.0),
                    egui::pos2(20.0, 10.0),
                    egui::pos2(20.0, 20.0),
                ];
                clipped_shape_cpu(ui, &polygon1, |ui| {
                    ui.label("Clipped 1");
                });
                clipped_shape_cpu(ui, &polygon2, |ui| {
                    ui.label("Clipped 2");
                });
            });
        });
    }

    #[test]
    fn test_paint_image_from_path_missing() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(10.0));
                let success = paint_image_from_path(
                    ui,
                    ui.painter(),
                    rect,
                    "nonexistent.png",
                    "test_id",
                    egui::Color32::WHITE,
                );
                assert!(!success);
            });
        });
    }

    #[test]
    fn test_bevel_join_emits_segmented_geometry() {
        let points = vec![
            egui::pos2(0.0, 0.0),
            egui::pos2(10.0, 0.0),
            egui::pos2(10.0, 10.0),
        ];
        let stroke = RichStroke {
            width: 2.0,
            color: egui::Color32::WHITE,
            dash: None,
            cap: StrokeCap::Butt,
            join: StrokeJoin::Bevel,
        };

        let shapes = dashed_path_shapes(&points, &stroke);
        assert_eq!(shapes.len(), 2);
    }

    #[test]
    fn test_layered_painter_from_ui_and_layers() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let layered = LayeredPainter::from_ui(ui);
                let clip = ui.clip_rect();
                assert_eq!(layered.background().clip_rect(), clip);
                assert_eq!(layered.main().clip_rect(), clip);
                assert_eq!(layered.foreground().clip_rect(), clip);
            });
        });
    }

    #[test]
    fn test_transform_apply_to_shape_and_rect() {
        let transform = Transform2D::translate(10.0, 5.0).then(Transform2D::scale(2.0, 3.0));
        let rect = egui::Rect::from_min_size(egui::pos2(1.0, 2.0), egui::vec2(3.0, 4.0));
        let transformed_rect = transform.apply_to_rect(rect);
        assert_eq!(transformed_rect.min, egui::pos2(22.0, 21.0));
        assert_eq!(transformed_rect.max, egui::pos2(28.0, 33.0));

        let shape = Shape::LineSegment {
            points: [egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)],
            stroke: egui::Stroke::new(1.0, egui::Color32::WHITE),
        };
        match transform.apply_to_shape(shape) {
            Shape::LineSegment { points, .. } => {
                assert_eq!(points[0], egui::pos2(20.0, 15.0));
                assert_eq!(points[1], egui::pos2(22.0, 18.0));
            }
            other => panic!("unexpected shape: {:?}", other),
        }
    }

    #[test]
    fn test_stack_align_to_align2() {
        assert_eq!(StackAlign::TopLeft.to_align2(), egui::Align2::LEFT_TOP);
        assert_eq!(StackAlign::Center.to_align2(), egui::Align2::CENTER_CENTER);
        assert_eq!(
            StackAlign::BottomRight.to_align2(),
            egui::Align2::RIGHT_BOTTOM
        );
    }
}
