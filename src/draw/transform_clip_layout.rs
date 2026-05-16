use super::*;

/// 2D affine transform (SVG matrix convention: [a, b, c, d, e, f]).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
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

/// Fill rule for a CPU offscreen compound clip mask.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ClipFillRule {
    /// Only the first contour is considered.
    #[default]
    SinglePolygon,
    /// A point is inside when enclosed by an odd number of contours.
    EvenOdd,
    /// A point is inside when the summed winding number is non-zero.
    NonZero,
}

/// CPU offscreen clip mask for exact bounded group compositing.
#[derive(Clone, Debug, PartialEq)]
pub enum ClipMask {
    /// Vector contours with an explicit fill rule.
    Contours {
        contours: Vec<Vec<egui::Pos2>>,
        fill_rule: ClipFillRule,
    },
    /// Alpha-mask image sampled over `bounds`; alpha >= `threshold` is visible.
    Alpha {
        bounds: egui::Rect,
        size: [u32; 2],
        alpha: Vec<u8>,
        threshold: u8,
    },
}

impl ClipMask {
    /// Create a single-polygon clip mask.
    pub fn from_polygon(polygon: Vec<egui::Pos2>) -> Self {
        Self::Contours {
            contours: vec![polygon],
            fill_rule: ClipFillRule::SinglePolygon,
        }
    }

    /// Create an axis-aligned rectangular clip mask.
    pub fn rect(rect: egui::Rect) -> Self {
        Self::from_polygon(vec![
            rect.min,
            egui::pos2(rect.max.x, rect.min.y),
            rect.max,
            egui::pos2(rect.min.x, rect.max.y),
        ])
    }

    /// Create a rounded-rectangle contour mask using the crate's vector path approximation.
    pub fn rounded_rect(rect: egui::Rect, rounding: f32) -> Self {
        Self::from_polygon(rounded_rect_path(rect, rounding))
    }

    /// Create a compound even-odd clip mask, useful for holes.
    pub fn compound_even_odd(contours: Vec<Vec<egui::Pos2>>) -> Self {
        Self::Contours {
            contours,
            fill_rule: ClipFillRule::EvenOdd,
        }
    }

    /// Create a compound non-zero winding clip mask.
    pub fn compound_non_zero(contours: Vec<Vec<egui::Pos2>>) -> Self {
        Self::Contours {
            contours,
            fill_rule: ClipFillRule::NonZero,
        }
    }

    /// Create an alpha mask. Invalid alpha buffer lengths produce an empty mask.
    pub fn alpha(bounds: egui::Rect, size: [u32; 2], alpha: Vec<u8>, threshold: u8) -> Self {
        Self::Alpha {
            bounds,
            size,
            alpha,
            threshold,
        }
    }

    /// Returns true if the mask has enough data to clip a group.
    pub fn is_valid(&self) -> bool {
        match self {
            Self::Contours { contours, .. } => {
                !contours.is_empty() && contours.iter().all(|contour| contour_is_valid(contour))
            }
            Self::Alpha {
                bounds,
                size,
                alpha,
                ..
            } => {
                bounds.is_finite()
                    && bounds.is_positive()
                    && size[0] > 0
                    && size[1] > 0
                    && alpha.len() == (size[0] as usize * size[1] as usize)
            }
        }
    }

    /// Returns true if `point` is visible through this mask.
    pub fn contains(&self, point: egui::Pos2) -> bool {
        match self {
            Self::Contours {
                contours,
                fill_rule,
            } => match fill_rule {
                ClipFillRule::SinglePolygon => contours.first().is_some_and(|contour| {
                    contour_is_valid(contour) && point_in_polygon(point, contour)
                }),
                ClipFillRule::EvenOdd => {
                    contours
                        .iter()
                        .filter(|contour| {
                            contour_is_valid(contour) && point_in_polygon(point, contour)
                        })
                        .count()
                        % 2
                        == 1
                }
                ClipFillRule::NonZero => {
                    contours
                        .iter()
                        .filter(|contour| contour_is_valid(contour))
                        .map(|contour| winding_number(point, contour))
                        .sum::<i32>()
                        != 0
                }
            },
            Self::Alpha {
                bounds,
                size,
                alpha,
                threshold,
            } => {
                if !self.is_valid() || !bounds.contains(point) {
                    return false;
                }
                let u = ((point.x - bounds.min.x) / bounds.width()).clamp(0.0, 0.999_999);
                let v = ((point.y - bounds.min.y) / bounds.height()).clamp(0.0, 0.999_999);
                let x = (u * size[0] as f32).floor() as u32;
                let y = (v * size[1] as f32).floor() as u32;
                alpha[(y * size[0] + x) as usize] >= *threshold
            }
        }
    }

    /// Bounding box of all contours or alpha bounds.
    pub fn bounds(&self) -> Option<egui::Rect> {
        match self {
            Self::Contours { contours, .. } => contours
                .iter()
                .filter(|contour| contour_is_valid(contour))
                .flat_map(|contour| contour.iter().copied())
                .map(|point| egui::Rect::from_min_max(point, point))
                .reduce(|a, b| a.union(b)),
            Self::Alpha { bounds, .. } if bounds.is_finite() && bounds.is_positive() => {
                Some(*bounds)
            }
            Self::Alpha { .. } => None,
        }
    }

    pub(crate) fn hash_into<H: std::hash::Hasher>(&self, state: &mut H) {
        use std::hash::Hash;
        match self {
            Self::Contours {
                contours,
                fill_rule,
            } => {
                0u8.hash(state);
                fill_rule.hash(state);
                contours.len().hash(state);
                for contour in contours {
                    contour.len().hash(state);
                    for point in contour {
                        point.x.to_bits().hash(state);
                        point.y.to_bits().hash(state);
                    }
                }
            }
            Self::Alpha {
                bounds,
                size,
                alpha,
                threshold,
            } => {
                1u8.hash(state);
                bounds.min.x.to_bits().hash(state);
                bounds.min.y.to_bits().hash(state);
                bounds.max.x.to_bits().hash(state);
                bounds.max.y.to_bits().hash(state);
                size.hash(state);
                threshold.hash(state);
                alpha.hash(state);
            }
        }
    }
}

fn contour_is_valid(contour: &[egui::Pos2]) -> bool {
    contour.len() >= 3
        && contour
            .iter()
            .all(|point| point.x.is_finite() && point.y.is_finite())
        && contour_bounds(contour).is_some_and(|bounds| bounds.is_positive())
        && polygon_area(contour).abs() > 0.0001
}

fn contour_bounds(contour: &[egui::Pos2]) -> Option<egui::Rect> {
    contour
        .iter()
        .copied()
        .map(|point| egui::Rect::from_min_max(point, point))
        .reduce(|a, b| a.union(b))
}

fn polygon_area(contour: &[egui::Pos2]) -> f32 {
    let mut area = 0.0;
    for i in 0..contour.len() {
        let a = contour[i];
        let b = contour[(i + 1) % contour.len()];
        area += a.x * b.y - b.x * a.y;
    }
    area * 0.5
}

fn winding_number(p: egui::Pos2, polygon: &[egui::Pos2]) -> i32 {
    fn cross(a: egui::Vec2, b: egui::Vec2) -> f32 {
        a.x * b.y - a.y * b.x
    }
    let mut winding = 0i32;
    for i in 0..polygon.len() {
        let a = polygon[i];
        let b = polygon[(i + 1) % polygon.len()];
        if a.y <= p.y {
            if b.y > p.y && cross(b - a, p - a) > 0.0 {
                winding += 1;
            }
        } else if b.y <= p.y && cross(b - a, p - a) < 0.0 {
            winding -= 1;
        }
    }
    winding
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
