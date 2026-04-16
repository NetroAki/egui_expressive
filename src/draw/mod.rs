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
