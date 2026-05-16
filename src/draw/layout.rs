use super::geometry::{point_in_polygon, winding_number};
use egui::epaint::*;
use egui::*;

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

/// Fill rule for a compound clip mask.
#[derive(Clone, Copy, Debug, Default, PartialEq, Hash)]
pub enum ClipFillRule {
    /// Single polygon with standard inside/outside test (even-odd winding).
    #[default]
    SinglePolygon,
    /// Even-odd: point is inside if enclosed by an odd number of contours.
    EvenOdd,
    /// Non-zero: point is inside if the winding number is non-zero.
    NonZero,
}

/// A clip mask that may have multiple contours (for compound/hole masks).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClipMask {
    /// Contours defining the clip region. For `EvenOdd`, outer and hole contours
    /// alternate; a point is visible if enclosed by an odd number of contours.
    pub contours: Vec<Vec<egui::Pos2>>,
    /// Fill rule used to determine which pixels are inside the clip region.
    pub fill_rule: ClipFillRule,
}

impl ClipMask {
    /// Create a single-polygon clip mask.
    pub fn from_polygon(polygon: Vec<egui::Pos2>) -> Self {
        Self {
            contours: vec![polygon],
            fill_rule: ClipFillRule::SinglePolygon,
        }
    }

    /// Create a compound clip mask with even-odd fill rule (supports holes).
    pub fn compound_even_odd(contours: Vec<Vec<egui::Pos2>>) -> Self {
        Self {
            contours,
            fill_rule: ClipFillRule::EvenOdd,
        }
    }

    /// Returns true if the point is inside this clip mask.
    pub fn contains(&self, point: egui::Pos2) -> bool {
        match self.fill_rule {
            ClipFillRule::SinglePolygon => self
                .contours
                .first()
                .is_some_and(|c| point_in_polygon(point, c)),
            ClipFillRule::EvenOdd => {
                let count = self
                    .contours
                    .iter()
                    .filter(|c| point_in_polygon(point, c))
                    .count();
                count % 2 == 1
            }
            ClipFillRule::NonZero => {
                let winding: i32 = self.contours.iter().map(|c| winding_number(point, c)).sum();
                winding != 0
            }
        }
    }

    /// Bounding box of all contours.
    pub fn bounds(&self) -> Option<egui::Rect> {
        let points: Vec<&egui::Pos2> = self.contours.iter().flatten().collect();
        if points.is_empty() {
            return None;
        }
        let mut min = *points[0];
        let mut max = *points[0];
        for p in points {
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
        }
        Some(egui::Rect::from_min_max(min, max))
    }
}
