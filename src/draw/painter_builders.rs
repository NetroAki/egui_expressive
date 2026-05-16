use super::*;

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
    /// Create a new `LayeredPainter` from an [`egui::Ui`].
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
