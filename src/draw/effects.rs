use egui::epaint::*;
use egui::*;

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

/// Approximate an Illustrator bevel as deterministic vector highlight/shadow strips.
///
/// The effect is intentionally code-only: it emits translucent convex polygons around the
/// rectangle edge rather than relying on a raster filter or texture fallback.
pub fn bevel_rect(
    rect: egui::Rect,
    depth: f32,
    angle_deg: f32,
    radius: f32,
    highlight: egui::Color32,
    shadow_color: egui::Color32,
) -> Vec<egui::Shape> {
    if rect.is_negative() || rect.width() <= 0.0 || rect.height() <= 0.0 {
        return Vec::new();
    }

    let max_depth = (rect.width().min(rect.height()) * 0.5).max(0.0);
    let total_depth = depth.max(radius).clamp(0.0, max_depth);
    if total_depth <= 0.0 || (highlight.a() == 0 && shadow_color.a() == 0) {
        return Vec::new();
    }

    let steps = radius.ceil().clamp(1.0, 6.0) as usize;
    let angle = angle_deg.to_radians();
    // Conventional graphic angle: 135° lights top-left in y-down screen coordinates.
    let light_x = angle.cos();
    let light_y = -angle.sin();
    let mut shapes = Vec::with_capacity(steps * 4);

    for step in 0..steps {
        let t0 = step as f32 / steps as f32;
        let t1 = (step + 1) as f32 / steps as f32;
        let outer = rect.shrink(total_depth * t0);
        let inner = rect.shrink(total_depth * t1);
        if inner.is_negative() || inner.width() <= 0.0 || inner.height() <= 0.0 {
            break;
        }
        let fade = 1.0 - t0 * 0.6;

        let edges = [
            (
                (0.0, -1.0),
                vec![
                    outer.left_top(),
                    outer.right_top(),
                    inner.right_top(),
                    inner.left_top(),
                ],
            ),
            (
                (1.0, 0.0),
                vec![
                    outer.right_top(),
                    outer.right_bottom(),
                    inner.right_bottom(),
                    inner.right_top(),
                ],
            ),
            (
                (0.0, 1.0),
                vec![
                    outer.right_bottom(),
                    outer.left_bottom(),
                    inner.left_bottom(),
                    inner.right_bottom(),
                ],
            ),
            (
                (-1.0, 0.0),
                vec![
                    outer.left_bottom(),
                    outer.left_top(),
                    inner.left_top(),
                    inner.left_bottom(),
                ],
            ),
        ];

        for ((nx, ny), points) in edges {
            let intensity = (nx * light_x + ny * light_y).clamp(-1.0, 1.0);
            if intensity.abs() < 0.05 {
                continue;
            }
            let base = if intensity > 0.0 {
                highlight
            } else {
                shadow_color
            };
            let alpha = (base.a() as f32 * intensity.abs() * fade)
                .round()
                .clamp(0.0, 255.0) as u8;
            if alpha == 0 {
                continue;
            }
            let color = egui::Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), alpha);
            shapes.push(egui::Shape::convex_polygon(
                points,
                color,
                egui::Stroke::NONE,
            ));
        }
    }

    shapes
}
