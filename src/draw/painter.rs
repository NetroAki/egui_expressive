use egui::epaint::*;
use egui::*;

// Layered painter helpers and fluent shape builders for egui.

use egui::{
    epaint::{PathShape, PathStroke, RectShape, StrokeKind},
    Align2, Color32, CornerRadius, FontId, Id, LayerId, Order, Pos2, Rect, Shape, Stroke, Vec2,
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

/// Paint a sampled conic-style ring. This intentionally uses regular egui
/// line segments so it works without custom shaders.
pub fn conic_gradient_ring(
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    width: f32,
    start_angle: f32,
    end_angle: f32,
    colors: [Color32; 2],
) {
    let steps = ((end_angle - start_angle).abs() * radius / 4.0)
        .ceil()
        .max(8.0) as usize;
    for i in 0..steps {
        let t0 = i as f32 / steps as f32;
        let t1 = (i + 1) as f32 / steps as f32;
        let a0 = start_angle + (end_angle - start_angle) * t0;
        let a1 = start_angle + (end_angle - start_angle) * t1;
        let color = lerp_color(colors[0], colors[1], (t0 + t1) * 0.5);
        painter.line_segment(
            [
                center + Vec2::angled(a0) * radius,
                center + Vec2::angled(a1) * radius,
            ],
            Stroke::new(width, color),
        );
    }
}

/// Paint evenly spaced radial tick marks around a control ring.
pub fn tick_marks(
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    count: usize,
    length: f32,
    angle_range: std::ops::RangeInclusive<f32>,
    stroke: Stroke,
) {
    if count == 0 {
        return;
    }
    for i in 0..count {
        let t = if count == 1 {
            0.0
        } else {
            i as f32 / (count - 1) as f32
        };
        let angle = *angle_range.start() + (*angle_range.end() - *angle_range.start()) * t;
        let dir = Vec2::angled(angle);
        painter.line_segment(
            [center + dir * (radius - length), center + dir * radius],
            stroke,
        );
    }
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
