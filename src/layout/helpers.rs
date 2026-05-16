//! Small layout helper functions shared by stack/flex/grid APIs.

use egui::{Color32, Frame, Margin, Stroke, Vec2};

/// Apply Figma-style Auto Layout item spacing to a UI.
pub fn auto_layout(ui: &mut egui::Ui, gap: f32, _padding: f32) {
    ui.spacing_mut().item_spacing = Vec2::splat(gap);
}

/// Create a Frame with design-token-friendly parameters.
pub fn styled_frame(bg: Color32, rounding: f32, padding: f32, stroke: Option<Stroke>) -> Frame {
    let padding_i8 = padding.round() as i8;
    let mut frame = Frame::NONE
        .inner_margin(Margin::same(padding_i8))
        .fill(bg)
        .corner_radius(rounding);

    if let Some(s) = stroke {
        frame = frame.stroke(s);
    }

    frame
}

/// Horizontal rule (divider line).
pub fn hrule(ui: &mut egui::Ui, color: Color32, thickness: f32) {
    let available = ui.available_size();
    let mut clip = ui.clip_rect();
    clip.set_height(available.y);
    let y_center = clip.center().y;
    ui.painter()
        .hline(clip.x_range(), y_center, Stroke::new(thickness, color));
}

/// Vertical rule.
pub fn vrule(ui: &mut egui::Ui, color: Color32, thickness: f32) {
    let available = ui.available_size();
    let mut clip = ui.clip_rect();
    clip.set_width(available.x);
    let x_center = clip.center().x;
    ui.painter()
        .vline(x_center, clip.y_range(), Stroke::new(thickness, color));
}

/// Allocate space maintaining aspect ratio within available bounds.
pub fn aspect_ratio_fit(ui: &mut egui::Ui, ratio: f32) -> egui::Rect {
    let available = ui.available_size();
    let (w, h) = if available.x / available.y > ratio {
        (available.y * ratio, available.y)
    } else {
        (available.x, available.x / ratio)
    };
    let offset = egui::vec2((available.x - w) * 0.5, (available.y - h) * 0.5);
    let min = ui.cursor().min + offset;
    egui::Rect::from_min_size(min, egui::vec2(w, h))
}
