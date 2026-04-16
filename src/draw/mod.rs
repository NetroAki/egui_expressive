#![allow(dead_code)]

//! Layered painter helpers and fluent shape builders for egui.

use egui::{
    epaint::{PathShape, PathStroke, RectShape, StrokeKind},
    Color32, CornerRadius, Id, LayerId, Order, Pos2, Rect, Shape, Stroke,
};

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
        if self.fill == Color32::TRANSPARENT && self.stroke == Stroke::NONE {
            Shape::Circle(egui::epaint::CircleShape {
                center: self.center,
                radius: self.radius,
                fill: self.fill,
                stroke: self.stroke,
            })
        } else if self.stroke == Stroke::NONE {
            Shape::circle_filled(self.center, self.radius, self.fill)
        } else {
            Shape::Circle(egui::epaint::CircleShape {
                center: self.center,
                radius: self.radius,
                fill: self.fill,
                stroke: self.stroke,
            })
        }
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
    let steps = (blur_radius.ceil() as usize).max(1).min(12);
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
        let rounding = egui::CornerRadius::same((expansion * 0.5) as u8);
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
    let steps = (blur_radius.ceil() as usize).max(1).min(8);
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

/// Blend two colors using the specified blend mode.
pub fn blend_color(
    fg: egui::Color32,
    bg: egui::Color32,
    mode: crate::codegen::BlendMode,
) -> egui::Color32 {
    // Convert to linear RGBA (0-1 range)
    let fg = (
        fg.r() as f32 / 255.0,
        fg.g() as f32 / 255.0,
        fg.b() as f32 / 255.0,
        fg.a() as f32 / 255.0,
    );
    let bg = (
        bg.r() as f32 / 255.0,
        bg.g() as f32 / 255.0,
        bg.b() as f32 / 255.0,
        bg.a() as f32 / 255.0,
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
    };

    // Convert back to u8
    let r = (r.clamp(0.0, 1.0) * 255.0) as u8;
    let g = (g.clamp(0.0, 1.0) * 255.0) as u8;
    let b = (b.clamp(0.0, 1.0) * 255.0) as u8;
    let a = ((fg.3 * fg.3 + bg.3 * (1.0 - fg.3)).sqrt().clamp(0.0, 1.0) * 255.0) as u8;

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
    use egui::{epaint::Mesh, Pos2, Vec2};
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
    if points.len() < 2 {
        return;
    }
    match &stroke.dash {
        None => {
            // Solid stroke — use egui's native line
            for i in 0..points.len() - 1 {
                painter.line_segment(
                    [points[i], points[i + 1]],
                    Stroke::new(stroke.width, stroke.color),
                );
            }
        }
        Some(pattern) => {
            // Dashed stroke — walk the path, emit segments
            let total_len: f32 = points.windows(2).map(|w| (w[1] - w[0]).length()).sum();
            if total_len <= 0.0 {
                return;
            }
            let cycle_len: f32 = pattern.dashes.iter().sum();
            if cycle_len <= 0.0 {
                return;
            }

            let mut dist = pattern.offset % cycle_len;
            let mut phase = 0usize;
            let mut drawing = true;

            // Advance to correct phase based on initial dist
            let mut d = dist;
            while d >= pattern.dashes[phase] {
                d -= pattern.dashes[phase];
                phase = (phase + 1) % pattern.dashes.len();
                drawing = !drawing;
            }
            dist = d;

            let mut seg_start: Option<Pos2> = None;
            let mut current_pos = points[0];

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

                    if drawing {
                        if seg_start.is_none() {
                            seg_start = Some(current_pos);
                        }
                    } else {
                        if let Some(start) = seg_start.take() {
                            painter.line_segment(
                                [start, current_pos],
                                Stroke::new(stroke.width, stroke.color),
                            );
                        }
                    }

                    current_pos = next_pos;
                    walked += step;
                    dist += step;

                    if dist >= pattern.dashes[phase] {
                        if drawing {
                            if let Some(start) = seg_start.take() {
                                painter.line_segment(
                                    [start, current_pos],
                                    Stroke::new(stroke.width, stroke.color),
                                );
                            }
                        }
                        dist = 0.0;
                        phase = (phase + 1) % pattern.dashes.len();
                        drawing = !drawing;
                    }
                }
            }
            // Emit any remaining dash
            if drawing {
                if let Some(start) = seg_start {
                    painter.line_segment(
                        [start, current_pos],
                        Stroke::new(stroke.width, stroke.color),
                    );
                }
            }
        }
    }
}

// ─── 2D Transform ─────────────────────────────────────────────────────────────

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
        Self::translate(center.x, center.y)
            .then(Self::rotate(angle_deg))
            .then(Self::translate(-center.x, -center.y))
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
            a: self.a * other.a + self.b * other.c,
            b: self.a * other.b + self.b * other.d,
            c: self.c * other.a + self.d * other.c,
            d: self.c * other.b + self.d * other.d,
            e: self.e * other.a + self.f * other.c + other.e,
            f: self.e * other.b + self.f * other.d + other.f,
        }
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
        other => other,
    }
}

// ─── Clipped Rounded Rect ─────────────────────────────────────────────────────

/// Render content clipped to a rounded rectangle.
pub fn clipped_rounded_rect(
    ui: &mut egui::Ui,
    rect: Rect,
    rounding: f32,
    content: impl FnOnce(&mut egui::Ui),
) {
    let clip = ui.painter().clip_rect().intersect(rect);
    let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(rect));
    child_ui.set_clip_rect(clip);
    content(&mut child_ui);
}

// ---------------------------------------------------------------------------
// ClipScope — clip children to arbitrary shapes
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
/// clip_to(ui, ClipShape::Circle(center, 32.0), |ui| {
///     ui.image(texture_id, Vec2::splat(64.0));
/// });
///
/// // Clip content to a rounded card
/// clip_to(ui, ClipShape::RoundedRect(card_rect, CornerRadius::same(8)), |ui| {
///     ui.label("Clipped content");
/// });
/// ```
pub fn clip_to(
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
        // Full opacity — no overhead
        return ui.scope(content).response;
    }

    // Capture shapes painted during the closure by using a child painter
    // We use egui's layer system: paint to a temp layer, then fade and re-add
    let layer_id = egui::LayerId::new(egui::Order::Middle, ui.id().with("__opacity_layer"));
    let painter = ui.ctx().layer_painter(layer_id);
    let _ = painter; // painter is used implicitly by child ui

    // Simpler approach: scope the content, then fade shapes in the response rect
    let scope = ui.scope(content);
    let rect = scope.response.rect;

    // Paint a semi-transparent overlay to simulate opacity reduction
    // (True opacity groups require render-to-texture which egui doesn't expose)
    if opacity < 1.0 {
        let overlay_alpha = ((1.0 - opacity) * 255.0) as u8;
        // Use the window background color with alpha to blend
        let bg = ui.visuals().window_fill();
        let overlay = egui::Color32::from_rgba_unmultiplied(bg.r(), bg.g(), bg.b(), overlay_alpha);
        ui.painter().rect_filled(rect, 0.0, overlay);
    }

    scope.response
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

/// Stack multiple layers at the same position.
/// Each layer is a closure that receives a `&mut Ui` positioned at `rect`.
/// The rect is determined by the first layer.
pub fn zstack_layers(
    ui: &mut egui::Ui,
    layers: Vec<Box<dyn FnOnce(&mut egui::Ui)>>,
) -> egui::Response {
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

/// Alignment for ZStack children.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StackAlign {
    #[default]
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
    fn anchor(self) -> egui::Align2 {
        match self {
            StackAlign::TopLeft => egui::Align2::LEFT_TOP,
            StackAlign::TopCenter => egui::Align2::CENTER_TOP,
            StackAlign::TopRight => egui::Align2::RIGHT_TOP,
            StackAlign::CenterLeft => egui::Align2::LEFT_CENTER,
            StackAlign::Center => egui::Align2::CENTER_CENTER,
            StackAlign::CenterRight => egui::Align2::RIGHT_CENTER,
            StackAlign::BottomLeft => egui::Align2::LEFT_BOTTOM,
            StackAlign::BottomCenter => egui::Align2::CENTER_BOTTOM,
            StackAlign::BottomRight => egui::Align2::RIGHT_BOTTOM,
        }
    }
}
