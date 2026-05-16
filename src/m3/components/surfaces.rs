use crate::m3::{M3Elevation, M3Theme};
use egui::{CornerRadius, Frame, Margin, Response, Stroke, Ui};

fn card_visuals(theme: &M3Theme, variant: M3CardVariant) -> (egui::Color32, Stroke) {
    let c = &theme.colors;
    match variant {
        M3CardVariant::Elevated => (
            M3Elevation::Level1.surface_tint(c.surface_container_low, c.primary),
            Stroke::NONE,
        ),
        M3CardVariant::Filled => (c.surface_container_highest, Stroke::NONE),
        M3CardVariant::Outlined => (c.surface, Stroke::new(1.0, c.outline_variant)),
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub enum M3CardVariant {
    #[default]
    Elevated,
    Filled,
    Outlined,
}

pub struct M3Card {
    variant: M3CardVariant,
    padding: f32,
    width: Option<f32>,
}

impl Default for M3Card {
    fn default() -> Self {
        Self::new()
    }
}

impl M3Card {
    pub fn new() -> Self {
        Self {
            variant: M3CardVariant::Elevated,
            padding: 16.0,
            width: None,
        }
    }
    pub fn filled(mut self) -> Self {
        self.variant = M3CardVariant::Filled;
        self
    }
    pub fn outlined(mut self) -> Self {
        self.variant = M3CardVariant::Outlined;
        self
    }
    pub fn padding(mut self, p: f32) -> Self {
        self.padding = p;
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }

    pub fn show(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) -> Response {
        let theme = M3Theme::load(ui.ctx());
        let (bg, stroke) = card_visuals(&theme, self.variant);

        let frame = Frame::NONE
            .fill(bg)
            .stroke(stroke)
            .corner_radius(CornerRadius::same(12))
            .inner_margin(Margin::same(self.padding.clamp(0.0, 127.0).round() as i8));
        let resp = frame.show(ui, |ui| {
            if let Some(w) = self.width {
                ui.set_width(w);
            }
            content(ui);
        });
        resp.response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase7_card_visual_subset_is_token_deterministic() {
        let theme = M3Theme::light();
        let (filled_bg, filled_stroke) = card_visuals(&theme, M3CardVariant::Filled);
        assert_eq!(filled_bg, theme.colors.surface_container_highest);
        assert_eq!(filled_stroke, Stroke::NONE);

        let (outlined_bg, outlined_stroke) = card_visuals(&theme, M3CardVariant::Outlined);
        assert_eq!(outlined_bg, theme.colors.surface);
        assert_eq!(outlined_stroke.width, 1.0);
        assert_eq!(outlined_stroke.color, theme.colors.outline_variant);

        let card = M3Card::new().outlined().padding(12.0).width(160.0);
        assert_eq!(card.padding, 12.0);
        assert_eq!(card.width, Some(160.0));
    }
}
