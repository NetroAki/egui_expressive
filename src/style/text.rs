/// A text style combining font, color, and optional letter spacing.
#[derive(Clone, Debug)]
pub struct TextStyle {
    pub font_id: egui::FontId,
    pub color: egui::Color32,
    /// Extra pixels between characters (0.0 = normal).
    pub letter_spacing: f32,
}

impl TextStyle {
    pub fn new(font_id: egui::FontId, color: egui::Color32) -> Self {
        Self {
            font_id,
            color,
            letter_spacing: 0.0,
        }
    }

    pub fn with_spacing(mut self, spacing: f32) -> Self {
        self.letter_spacing = spacing;
        self
    }
}

/// Named text styles for a consistent type system.
#[derive(Clone, Debug)]
pub struct TextStyles {
    pub label: TextStyle,
    pub heading: TextStyle,
    pub mono: TextStyle,
    pub small: TextStyle,
    pub hint: TextStyle,
}

impl TextStyles {
    /// Default dark-theme text styles matching the mockup.
    pub fn dark() -> Self {
        let fg = egui::Color32::from_rgb(220, 220, 225);
        let muted = egui::Color32::from_rgb(130, 130, 142);
        Self {
            label: TextStyle::new(egui::FontId::proportional(12.0), fg),
            heading: TextStyle::new(egui::FontId::proportional(14.0), fg),
            mono: TextStyle::new(egui::FontId::monospace(11.0), fg),
            small: TextStyle::new(egui::FontId::proportional(10.0), muted),
            hint: TextStyle::new(egui::FontId::proportional(9.0), muted).with_spacing(0.5),
        }
    }
}

/// Paint text with optional letter spacing by advancing character by character.
/// Falls back to a single `painter.text()` call when `letter_spacing == 0.0`.
pub fn styled_text(
    painter: &egui::Painter,
    pos: egui::Pos2,
    text: &str,
    style: &TextStyle,
) -> egui::Rect {
    if style.letter_spacing == 0.0 || text.is_empty() {
        return painter.text(
            pos,
            egui::Align2::LEFT_TOP,
            text,
            style.font_id.clone(),
            style.color,
        );
    }

    // Character-by-character advance for letter spacing
    let mut cursor = pos;
    let mut total_rect = egui::Rect::from_min_size(pos, egui::Vec2::ZERO);
    for ch in text.chars() {
        let s = ch.to_string();
        let r = painter.text(
            cursor,
            egui::Align2::LEFT_TOP,
            &s,
            style.font_id.clone(),
            style.color,
        );
        total_rect = total_rect.union(r);
        cursor.x += r.width() + style.letter_spacing;
    }
    total_rect
}

// ─── Scrollbar Styling ────────────────────────────────────────────────────────

/// Apply custom scrollbar styling to egui's Visuals.
/// Call this before rendering to match the mockup's thin dark scrollbars.
///
pub fn apply_scrollbar_style(
    visuals: &mut egui::Visuals,
    track_color: egui::Color32,
    thumb_color: egui::Color32,
) {
    visuals.extreme_bg_color = track_color;
    // thumb color is applied via the inactive widget bg
    visuals.widgets.inactive.bg_fill = thumb_color;
    visuals.widgets.hovered.bg_fill = thumb_color.linear_multiply(1.2);
    // Note: scroll bar width is in Style::spacing, not Visuals.
    // Width must be set via ctx.style_mut(|s| s.spacing.scroll.bar_width = width)
}

/// Apply the mockup's default thin scrollbar style to the egui context.
/// Call once at app startup or per-frame before rendering.
pub fn apply_default_scrollbar_style(ctx: &egui::Context) {
    ctx.global_style_mut(|style| {
        style.spacing.scroll.bar_width = 4.0;
        style.spacing.scroll.bar_inner_margin = 1.0;
        style.spacing.scroll.bar_outer_margin = 0.0;
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(22, 22, 27);
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(55, 55, 63);
    });
}
