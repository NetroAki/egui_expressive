use crate::m3::{M3Elevation, M3Theme};
use crate::style::with_alpha;
use egui::{Color32, CornerRadius, Pos2, Response, Sense, Stroke, Ui, Vec2, Widget};

fn button_visuals(
    theme: &M3Theme,
    variant: M3ButtonVariant,
    enabled: bool,
) -> (Color32, Color32, Stroke) {
    let c = &theme.colors;
    let (bg, fg, stroke) = match variant {
        M3ButtonVariant::Filled => (c.primary, c.on_primary, Stroke::NONE),
        M3ButtonVariant::Tonal => (
            c.secondary_container,
            c.on_secondary_container,
            Stroke::NONE,
        ),
        M3ButtonVariant::Outlined => (Color32::TRANSPARENT, c.primary, Stroke::new(1.0, c.outline)),
        M3ButtonVariant::Text => (Color32::TRANSPARENT, c.primary, Stroke::NONE),
        M3ButtonVariant::Elevated => (
            M3Elevation::Level1.surface_tint(c.surface_container_low, c.primary),
            c.primary,
            Stroke::NONE,
        ),
    };

    let disabled_alpha = if enabled { 1.0 } else { 0.38 };
    (
        with_alpha(bg, if enabled { 1.0 } else { 0.12 }),
        with_alpha(fg, disabled_alpha),
        stroke,
    )
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum M3ButtonVariant {
    #[default]
    Filled,
    Tonal,
    Outlined,
    Text,
    Elevated,
}

pub struct M3Button<'a> {
    text: &'a str,
    variant: M3ButtonVariant,
    icon: Option<char>,
    enabled: bool,
    width: Option<f32>,
}

impl<'a> M3Button<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            variant: M3ButtonVariant::Filled,
            icon: None,
            enabled: true,
            width: None,
        }
    }
    pub fn tonal(mut self) -> Self {
        self.variant = M3ButtonVariant::Tonal;
        self
    }
    pub fn outlined(mut self) -> Self {
        self.variant = M3ButtonVariant::Outlined;
        self
    }
    pub fn text_only(mut self) -> Self {
        self.variant = M3ButtonVariant::Text;
        self
    }
    pub fn elevated(mut self) -> Self {
        self.variant = M3ButtonVariant::Elevated;
        self
    }
    pub fn icon(mut self, icon: char) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn enabled(mut self, e: bool) -> Self {
        self.enabled = e;
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }
}

impl Widget for M3Button<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = M3Theme::load(ui.ctx());
        let (bg, fg, stroke) = button_visuals(&theme, self.variant, self.enabled);
        let desired_size = Vec2::new(self.width.unwrap_or(0.0), 40.0);

        let (rect, response) = ui.allocate_at_least(
            Vec2::new(desired_size.x.max(64.0), 40.0),
            if self.enabled {
                Sense::click()
            } else {
                Sense::hover()
            },
        );

        if ui.is_rect_visible(rect) {
            let rounding = CornerRadius::same(20);
            let painter = ui.painter();

            painter.rect_filled(rect, rounding, bg);
            if stroke.width > 0.0 {
                painter.rect_stroke(rect, rounding, stroke, egui::StrokeKind::Outside);
            }
            if response.hovered() && self.enabled {
                painter.rect_filled(rect, rounding, with_alpha(fg, 0.08));
            }
            if response.is_pointer_button_down_on() && self.enabled {
                painter.rect_filled(rect, rounding, with_alpha(fg, 0.12));
            }

            let content_rect = rect.shrink2(Vec2::new(24.0, 0.0));
            let center = content_rect.center();
            if let Some(icon) = self.icon {
                let icon_str = icon.to_string();
                let text_galley = ui.painter().layout_no_wrap(
                    self.text.to_string(),
                    egui::FontId::proportional(14.0),
                    fg,
                );
                let total_w = 18.0 + 8.0 + text_galley.size().x;
                let start_x = center.x - total_w / 2.0;
                painter.text(
                    Pos2::new(start_x + 9.0, center.y),
                    egui::Align2::CENTER_CENTER,
                    icon_str,
                    egui::FontId::proportional(18.0),
                    fg,
                );
                painter.text(
                    Pos2::new(start_x + 18.0 + 8.0 + text_galley.size().x / 2.0, center.y),
                    egui::Align2::CENTER_CENTER,
                    self.text,
                    egui::FontId::proportional(14.0),
                    fg,
                );
            } else {
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    self.text,
                    egui::FontId::proportional(14.0),
                    fg,
                );
            }
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase7_button_visual_subset_is_token_deterministic() {
        let theme = M3Theme::light();
        let (filled_bg, filled_fg, filled_stroke) =
            button_visuals(&theme, M3ButtonVariant::Filled, true);
        assert_eq!(filled_bg, theme.colors.primary);
        assert_eq!(filled_fg, theme.colors.on_primary);
        assert_eq!(filled_stroke, Stroke::NONE);

        let (outlined_bg, outlined_fg, outlined_stroke) =
            button_visuals(&theme, M3ButtonVariant::Outlined, true);
        assert_eq!(outlined_bg, Color32::TRANSPARENT);
        assert_eq!(outlined_fg, theme.colors.primary);
        assert_eq!(outlined_stroke.width, 1.0);
        assert_eq!(outlined_stroke.color, theme.colors.outline);

        let (disabled_bg, disabled_fg, _) = button_visuals(&theme, M3ButtonVariant::Filled, false);
        assert_eq!(disabled_bg.a(), 30);
        assert_eq!(disabled_fg.a(), 96);
    }
}
