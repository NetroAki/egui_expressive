//! Typography utility methods for `Tw`.

use crate::tailwind::builder::Tw;
use crate::tailwind::types::FontWeight;
use crate::typography::TypeSpec;

impl Tw {
    pub fn text_xs(mut self) -> Self {
        self.font_size = Some(10.0);
        self
    }

    pub fn text_sm(mut self) -> Self {
        self.font_size = Some(12.0);
        self
    }

    pub fn text_base(mut self) -> Self {
        self.font_size = Some(14.0);
        self
    }

    pub fn text_lg(mut self) -> Self {
        self.font_size = Some(16.0);
        self
    }

    pub fn text_xl(mut self) -> Self {
        self.font_size = Some(20.0);
        self
    }

    pub fn text_2xl(mut self) -> Self {
        self.font_size = Some(24.0);
        self
    }

    pub fn text_3xl(mut self) -> Self {
        self.font_size = Some(30.0);
        self
    }

    pub fn font_thin(mut self) -> Self {
        self.font_weight = FontWeight::Thin;
        self
    }

    pub fn font_extralight(mut self) -> Self {
        self.font_weight = FontWeight::ExtraLight;
        self
    }

    pub fn font_light(mut self) -> Self {
        self.font_weight = FontWeight::Light;
        self
    }

    pub fn font_normal(mut self) -> Self {
        self.font_weight = FontWeight::Normal;
        self
    }

    pub fn font_medium(mut self) -> Self {
        self.font_weight = FontWeight::Medium;
        self
    }

    pub fn font_semibold(mut self) -> Self {
        self.font_weight = FontWeight::SemiBold;
        self
    }

    pub fn font_bold(mut self) -> Self {
        self.font_weight = FontWeight::Bold;
        self
    }

    pub fn font_extrabold(mut self) -> Self {
        self.font_weight = FontWeight::ExtraBold;
        self
    }

    pub fn font_black(mut self) -> Self {
        self.font_weight = FontWeight::Black;
        self
    }

    pub fn font_weight(mut self, weight: u16) -> Self {
        self.font_weight = FontWeight::from_css(weight);
        self
    }

    pub fn font_mono(mut self) -> Self {
        self.font_family = Some("mono");
        self
    }

    pub fn font_sans(mut self) -> Self {
        self.font_family = Some("sans");
        self
    }

    pub fn tracking(mut self, v: f32) -> Self {
        self.letter_spacing = Some(v);
        self
    }

    pub fn tracking_tight(mut self) -> Self {
        self.letter_spacing = Some(-0.5);
        self
    }

    pub fn tracking_wide(mut self) -> Self {
        self.letter_spacing = Some(0.5);
        self
    }

    pub fn tracking_wider(mut self) -> Self {
        self.letter_spacing = Some(1.0);
        self
    }

    pub fn rich_text(&self, text: impl Into<String>) -> egui::RichText {
        let mut rich = egui::RichText::new(text.into());
        if let Some(size) = self.font_size {
            rich = rich.size(size);
        }
        if let Some(spacing) = self.letter_spacing {
            rich = rich.extra_letter_spacing(spacing);
        }
        if let Some(fg) = self.fg {
            rich = rich.color(fg);
        }
        if let Some(family) = self.font_family {
            rich = rich.font(
                TypeSpec::new(self.font_size.unwrap_or(14.0))
                    .font_family(family)
                    .to_font_id(),
            );
        }
        match self.font_weight.css_value() {
            100..=300 => rich.weak(),
            400..=500 => rich,
            600..=900 => rich.strong(),
            _ => rich,
        }
    }

    pub fn label(&self, ui: &mut egui::Ui, text: impl Into<String>) -> egui::Response {
        ui.label(self.rich_text(text))
    }

    pub fn to_type_spec(&self) -> TypeSpec {
        let mut spec = TypeSpec::new(self.font_size.unwrap_or(14.0));
        if let Some(spacing) = self.letter_spacing {
            spec = spec.letter_spacing(spacing);
        }
        if let Some(color) = self.fg {
            spec = spec.color(color);
        }
        if let Some(family) = self.font_family {
            spec = spec.font_family(family);
        }
        spec = spec.weight(self.font_weight.css_value());
        spec
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Color32;

    #[test]
    fn tw_to_type_spec_preserves_exact_ascii_typography_subset() {
        let spec = Tw::new()
            .text_xl()
            .font_semibold()
            .tracking_wide()
            .text_color(Color32::from_rgb(20, 30, 40))
            .to_type_spec();

        assert_eq!(spec.size, 20.0);
        assert_eq!(
            spec.weight, 600,
            "R100-005A preserves Tailwind numeric weight intent in TypeSpec"
        );
        assert_eq!(spec.letter_spacing, 0.5);
        assert_eq!(spec.color, Some(Color32::from_rgb(20, 30, 40)));
        assert_eq!(
            spec.font_family, None,
            "Phase 6 exact subset uses registered default fonts only"
        );
    }

    #[test]
    fn tw_r100_005a_to_type_spec_records_nearest_weight_step() {
        let semibold = Tw::new().font_semibold().to_type_spec();
        assert_eq!(semibold.weight, 600);

        let rounded = Tw::new().font_weight(820).to_type_spec();
        assert_eq!(rounded.weight, 800);
    }

    #[test]
    fn tw_phase8_to_type_spec_preserves_builtin_family_aliases() {
        let mono = Tw::new().text_lg().font_mono().to_type_spec();
        assert_eq!(mono.font_family.as_deref(), Some("mono"));
        assert!(matches!(
            mono.to_font_id().family,
            egui::FontFamily::Monospace
        ));

        let sans = Tw::new().font_sans().to_type_spec();
        assert_eq!(sans.font_family.as_deref(), Some("sans"));
        assert!(matches!(
            sans.to_font_id().family,
            egui::FontFamily::Proportional
        ));
    }

    #[test]
    fn tw_phase8_rich_text_can_select_builtin_mono_family() {
        let rich = Tw::new().font_mono().rich_text("0123");
        assert!(format!("{rich:?}").contains("Monospace"));
    }

    #[test]
    fn tw_rich_text_weight_remains_bounded_egui_emphasis() {
        let rich = Tw::new().font_weight(820).rich_text("bounded");
        assert!(format!("{rich:?}").contains("strong: true"));

        let contract = include_str!("../../docs/ui-framework/tw-render-contract.md");
        assert!(contract.contains("Tw::to_type_spec"));
        assert!(contract.contains("`RichText` weight rendering"));
        assert!(contract.contains("remains bounded weak/normal/strong"));
    }
}
