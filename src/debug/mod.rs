//! Debugging overlays and visual helpers.

use egui::{Color32, Context, Id, LayerId, Order, Pos2, Rect, Response, Stroke, StrokeKind};

/// Debug overlay for visualizing UI rects and response zones.
///
/// Enabled via the `debug` feature flag.
#[derive(Debug, Clone)]
pub struct DebugOverlay {
    /// Show outline rectangles around widgets.
    pub show_rects: bool,
    /// Show widget ID labels.
    pub show_ids: bool,
    /// Show response hit zones.
    pub show_response_zone: bool,
    /// Show interaction state flags as text on response rects.
    pub show_interaction_state: bool,
    /// Show clip rects.
    pub show_clip_rects: bool,
    /// Color for rect outlines.
    pub rect_color: Color32,
    /// Color for text labels.
    pub text_color: Color32,
}

impl Default for DebugOverlay {
    fn default() -> Self {
        Self {
            show_rects: true,
            show_ids: false,
            show_response_zone: true,
            show_interaction_state: false,
            show_clip_rects: false,
            rect_color: Color32::from_rgba_unmultiplied(0, 200, 255, 80),
            text_color: Color32::from_rgb(0, 200, 255),
        }
    }
}

impl DebugOverlay {
    /// Draw an annotated rectangle on the debug layer.
    ///
    /// The rectangle is accumulated in memory and drawn when `show_all()` is called.
    pub fn rect(&self, ctx: &Context, rect: Rect, label: &str) {
        #[cfg(feature = "debug")]
        {
            if !self.show_rects {
                return;
            }

            let key = Id::new("__expressive_debug_rects");
            let mut rects: Vec<(Rect, String)> =
                ctx.memory(|m| m.data.get_temp(key).unwrap_or_default());
            rects.push((rect, label.to_string()));
            ctx.memory_mut(|m| m.data.insert_temp(key, rects));
        }
        #[cfg(not(feature = "debug"))]
        {
            let _ = (ctx, rect, label);
        }
    }

    /// Draw a debug outline for an egui Response (its rect + ID label).
    pub fn response(&self, ctx: &Context, response: &Response, label: &str) {
        #[cfg(feature = "debug")]
        {
            self.rect(ctx, response.rect, label);

            if self.show_interaction_state {
                debug_interaction(ctx, response, label);
            }
        }
        #[cfg(not(feature = "debug"))]
        {
            let _ = (ctx, response, label);
        }
    }

    /// Show all debug overlays for the current frame.
    ///
    /// Call this at the end of your `update()` to render any accumulated
    /// debug shapes.
    pub fn show_all(&self, ctx: &Context) {
        #[cfg(feature = "debug")]
        {
            let key = Id::new("__expressive_debug_rects");
            let rects: Vec<(Rect, String)> =
                ctx.memory(|m| m.data.get_temp(key).unwrap_or_default());

            if rects.is_empty() {
                return;
            }

            let painter = ctx.layer_painter(LayerId::new(Order::Debug, Id::new("__exp_debug")));

            for (r, label) in &rects {
                painter.rect_stroke(
                    *r,
                    0.0,
                    Stroke::new(1.0, self.rect_color),
                    StrokeKind::Outside,
                );
                if self.show_ids && !label.is_empty() {
                    painter.text(
                        r.min + egui::vec2(2.0, 2.0),
                        egui::Align2::LEFT_TOP,
                        label,
                        egui::FontId::monospace(10.0),
                        self.text_color,
                    );
                }
            }

            ctx.memory_mut(|m| m.data.remove::<Vec<(Rect, String)>>(key));
        }
        #[cfg(not(feature = "debug"))]
        {
            let _ = self;
            let _ = ctx;
        }
    }
}

/// Draw a small label on the debug layer at the given position.
#[cfg(feature = "debug")]
pub fn debug_label(ctx: &Context, pos: Pos2, label: &str) {
    let painter = ctx.layer_painter(LayerId::new(Order::Debug, Id::new("__exp_debug")));
    painter.text(
        pos,
        egui::Align2::LEFT_TOP,
        label,
        egui::FontId::monospace(10.0),
        Color32::from_rgb(0, 200, 255),
    );
}

/// Draw the response rect and state flags as text on the debug layer.
#[cfg(feature = "debug")]
pub fn debug_interaction(ctx: &Context, response: &Response, label: &str) {
    let painter = ctx.layer_painter(LayerId::new(Order::Debug, Id::new("__exp_debug")));

    // Draw the response rect outline
    painter.rect_stroke(
        response.rect,
        0.0,
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 200, 0, 100)),
        StrokeKind::Outside,
    );

    // Build interaction state text
    let mut lines = vec![label.to_string()];
    lines.push(format!(
        "hovered: {}",
        if response.hovered() { "1" } else { "0" }
    ));
    lines.push(format!(
        "pressed: {}",
        if response.is_pointer_button_down_on() {
            "1"
        } else {
            "0"
        }
    ));
    lines.push(format!(
        "dragged: {}",
        if response.dragged() { "1" } else { "0" }
    ));
    lines.push(format!(
        "focused: {}",
        if response.has_focus() { "1" } else { "0" }
    ));
    lines.push(format!(
        "clicked: {}",
        if response.clicked() { "1" } else { "0" }
    ));
    lines.push(format!(
        "dbl_click: {}",
        if response.double_clicked() { "1" } else { "0" }
    ));

    let text = lines.join("\n");
    painter.text(
        response.rect.max + egui::vec2(2.0, 0.0),
        egui::Align2::LEFT_TOP,
        text,
        egui::FontId::monospace(9.0),
        Color32::from_rgb(255, 200, 0),
    );
}
