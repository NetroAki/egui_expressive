use crate::scene::PathContour;
use egui::{Color32, Context, FontFamily, FontId, RichText};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum TextDecoration {
    #[default]
    None,
    Underline,
    Strikethrough,
    Both,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum TextOverflow {
    #[default]
    Visible,
    Ellipsis,
    Clip,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum TextTransform {
    #[default]
    None,
    Uppercase,
    Lowercase,
    Capitalize,
    SmallCaps,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OpenTypeFeatures {
    pub ligatures: bool,
    pub contextual_ligatures: bool,
    pub discretionary_ligatures: bool,
    pub fractions: bool,
    pub ordinals: bool,
    pub swash: bool,
    pub titling_alternates: bool,
    pub stylistic_alternates: bool,
    pub kerning: bool,
}

impl OpenTypeFeatures {
    pub fn all_off() -> Self {
        Self {
            ligatures: false,
            contextual_ligatures: false,
            discretionary_ligatures: false,
            fractions: false,
            ordinals: false,
            swash: false,
            titling_alternates: false,
            stylistic_alternates: false,
            kerning: false,
        }
    }

    pub fn can_use_fast_path(&self) -> bool {
        *self == Self::default()
    }
}

impl Default for OpenTypeFeatures {
    fn default() -> Self {
        Self {
            ligatures: true,
            contextual_ligatures: true,
            discretionary_ligatures: false,
            fractions: false,
            ordinals: false,
            swash: false,
            titling_alternates: false,
            stylistic_alternates: false,
            kerning: true,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ShapedGlyph {
    #[serde(alias = "glyphId")]
    pub glyph_id: u32,
    pub cluster: u32,
    #[serde(alias = "advanceX")]
    pub advance_x: f32,
    #[serde(alias = "advanceY")]
    pub advance_y: f32,
    #[serde(alias = "offsetX")]
    pub offset_x: f32,
    #[serde(alias = "offsetY")]
    pub offset_y: f32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contours: Vec<PathContour>,
    #[serde(default, alias = "contoursAbsolute")]
    pub contours_are_absolute: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShapedGlyphRun {
    pub text: String,
    pub glyphs: Vec<ShapedGlyph>,
}

pub fn shaped_glyph_run_advance_width(run: &ShapedGlyphRun, spec: &TypeSpec) -> f32 {
    run.glyphs
        .iter()
        .map(|glyph| glyph.advance_x.max(0.0) * spec.horizontal_scale)
        .sum()
}

#[derive(Clone, Debug)]
pub struct TypeSpec {
    pub size: f32,
    pub weight: u16,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub color: Option<Color32>,
    pub font_family: Option<String>,
    pub decoration: TextDecoration,
    pub overflow: TextOverflow,
    pub text_transform: TextTransform,
    pub open_type_features: OpenTypeFeatures,
    pub baseline_shift: f32,
    pub horizontal_scale: f32,
    pub vertical_scale: f32,
}

impl TypeSpec {
    pub fn new(size: f32) -> Self {
        Self {
            size,
            weight: 400,
            line_height: 1.4,
            letter_spacing: 0.0,
            color: None,
            font_family: None,
            decoration: TextDecoration::None,
            overflow: TextOverflow::Visible,
            text_transform: TextTransform::None,
            open_type_features: OpenTypeFeatures::default(),
            baseline_shift: 0.0,
            horizontal_scale: 1.0,
            vertical_scale: 1.0,
        }
    }

    pub fn micro_label() -> Self {
        Self::new(9.0)
            .weight(700)
            .letter_spacing(1.2)
            .line_height(1.0)
            .text_transform(TextTransform::Uppercase)
    }

    pub fn mono_readout(size: f32) -> Self {
        Self::new(size)
            .weight(600)
            .letter_spacing(0.2)
            .font_family("mono")
    }

    pub fn weight(mut self, w: u16) -> Self {
        self.weight = w;
        self
    }

    pub fn line_height(mut self, lh: f32) -> Self {
        self.line_height = lh;
        self
    }

    pub fn letter_spacing(mut self, ls: f32) -> Self {
        self.letter_spacing = ls;
        self
    }

    pub fn color(mut self, c: Color32) -> Self {
        self.color = Some(c);
        self
    }

    pub fn font_family(mut self, f: impl Into<String>) -> Self {
        self.font_family = Some(f.into());
        self
    }

    pub fn decoration(mut self, d: TextDecoration) -> Self {
        self.decoration = d;
        self
    }

    pub fn overflow(mut self, o: TextOverflow) -> Self {
        self.overflow = o;
        self
    }

    pub fn text_transform(mut self, t: TextTransform) -> Self {
        self.text_transform = t;
        self
    }

    pub fn open_type_features(mut self, otf: OpenTypeFeatures) -> Self {
        self.open_type_features = otf;
        self
    }

    pub fn ligatures(mut self, on: bool) -> Self {
        self.open_type_features.ligatures = on;
        self
    }

    pub fn baseline_shift(mut self, px: f32) -> Self {
        self.baseline_shift = px;
        self
    }

    pub fn horizontal_scale(mut self, s: f32) -> Self {
        self.horizontal_scale = s;
        self
    }

    pub fn vertical_scale(mut self, s: f32) -> Self {
        self.vertical_scale = s;
        self
    }

    pub fn effective_size(&self) -> f32 {
        self.size * self.vertical_scale
    }

    pub fn requires_full_shaper(&self) -> bool {
        !self.open_type_features.can_use_fast_path()
            || self.text_transform == TextTransform::SmallCaps
            || self.horizontal_scale != 1.0
            || self.vertical_scale != 1.0
            || self.baseline_shift != 0.0
            || self.letter_spacing != 0.0
    }

    pub(super) fn can_use_fast_path(&self) -> bool {
        self.letter_spacing == 0.0
            && self.text_transform != TextTransform::SmallCaps
            && self.open_type_features.can_use_fast_path()
            && self.horizontal_scale == 1.0
            && self.vertical_scale == 1.0
            && self.baseline_shift == 0.0
    }

    pub fn to_font_id(&self) -> FontId {
        let family = self
            .font_family
            .as_deref()
            .map(font_family_from_alias)
            .unwrap_or(FontFamily::Proportional);
        FontId::new(self.effective_size(), family)
    }

    pub fn to_rich_text(&self, text: &str) -> RichText {
        let rich_text = RichText::new(text).size(self.effective_size());
        let rich_text = match &self.font_family {
            Some(f) => rich_text.font(FontId::new(
                self.effective_size(),
                font_family_from_alias(f),
            )),
            None => rich_text,
        };

        let rich_text = match self.weight {
            100..=300 => rich_text.weak(),
            400..=500 => rich_text,
            600..=900 => rich_text.strong(),
            _ => rich_text,
        };

        match self.color {
            Some(c) => rich_text.color(c),
            None => rich_text,
        }
    }
}

fn font_family_from_alias(name: &str) -> FontFamily {
    match name.trim().to_ascii_lowercase().as_str() {
        "mono" | "monospace" => FontFamily::Monospace,
        "sans" | "proportional" => FontFamily::Proportional,
        _ => FontFamily::Name(name.to_owned().into()),
    }
}

#[cfg(test)]
mod phase8_tests {
    use super::*;

    #[test]
    fn builtin_family_aliases_map_to_egui_builtin_families() {
        assert!(matches!(
            TypeSpec::new(13.0).font_family("mono").to_font_id().family,
            FontFamily::Monospace
        ));
        assert!(matches!(
            TypeSpec::new(13.0)
                .font_family("monospace")
                .to_font_id()
                .family,
            FontFamily::Monospace
        ));
        assert!(matches!(
            TypeSpec::new(13.0).font_family("sans").to_font_id().family,
            FontFamily::Proportional
        ));
        assert!(matches!(
            TypeSpec::new(13.0)
                .font_family("proportional")
                .to_font_id()
                .family,
            FontFamily::Proportional
        ));
    }

    #[test]
    fn custom_family_names_stay_named_families() {
        let id = TypeSpec::new(13.0).font_family("Custom UI").to_font_id();
        assert!(matches!(id.family, FontFamily::Name(_)));
    }

    #[test]
    fn mono_readout_uses_builtin_monospace_alias() {
        assert!(matches!(
            TypeSpec::mono_readout(12.0).to_font_id().family,
            FontFamily::Monospace
        ));
    }

    #[test]
    fn r100_005a_type_spec_rich_text_uses_bounded_weight_emphasis() {
        let weak = format!("{:?}", TypeSpec::new(14.0).weight(300).to_rich_text("thin"));
        assert!(weak.contains("weak: true"));

        let normal = format!(
            "{:?}",
            TypeSpec::new(14.0).weight(500).to_rich_text("normal")
        );
        assert!(!normal.contains("weak: true"));
        assert!(!normal.contains("strong: true"));

        let strong = format!(
            "{:?}",
            TypeSpec::new(14.0).weight(600).to_rich_text("strong")
        );
        assert!(strong.contains("strong: true"));
    }

    #[test]
    fn r100_005a_type_spec_font_id_remains_weight_agnostic() {
        let light = TypeSpec::new(13.0).weight(300).to_font_id();
        let bold = TypeSpec::new(13.0).weight(800).to_font_id();

        assert_eq!(light.size, bold.size);
        assert_eq!(light.family, bold.family);
    }
}

impl Default for TypeSpec {
    fn default() -> Self {
        TypeSpec::new(14.0)
    }
}

/// A type scale with named presets matching common design-system conventions.
#[derive(Clone, Debug)]
pub struct TypeScale {
    pub display: TypeSpec,
    pub headline: TypeSpec,
    pub title_lg: TypeSpec,
    pub title_md: TypeSpec,
    pub title_sm: TypeSpec,
    pub body_lg: TypeSpec,
    pub body_md: TypeSpec,
    pub body_sm: TypeSpec,
    pub label_lg: TypeSpec,
    pub label_md: TypeSpec,
    pub label_sm: TypeSpec,
    pub mono: TypeSpec,
}

impl Default for TypeScale {
    fn default() -> Self {
        Self {
            display: TypeSpec::new(57.0),
            headline: TypeSpec::new(32.0),
            title_lg: TypeSpec::new(22.0),
            title_md: TypeSpec::new(16.0).weight(500),
            title_sm: TypeSpec::new(14.0).weight(500),
            body_lg: TypeSpec::new(16.0),
            body_md: TypeSpec::new(14.0),
            body_sm: TypeSpec::new(12.0),
            label_lg: TypeSpec::new(14.0).weight(500),
            label_md: TypeSpec::new(12.0).weight(500),
            label_sm: TypeSpec::new(11.0).weight(500),
            mono: TypeSpec::new(13.0).font_family("mono"),
        }
    }
}

impl TypeScale {
    const STORE_ID: &'static str = "egui_expressive_type_scale";

    /// Stores this type scale in egui's context.
    pub fn store(&self, ctx: &Context) {
        ctx.data_mut(|data| data.insert_temp(egui::Id::new(Self::STORE_ID), self.clone()));
    }

    /// Loads the type scale from egui's context, falling back to the default scale.
    pub fn load(ctx: &Context) -> Self {
        ctx.data(|data| {
            data.get_temp(egui::Id::new(Self::STORE_ID))
                .unwrap_or_else(Self::default)
        })
    }
}
