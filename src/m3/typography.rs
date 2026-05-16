use crate::typography::TypeSpec;
use egui::FontId;

/// M3 text style definition.
#[derive(Clone, Debug)]
pub struct M3TextStyle {
    pub font_size: f32,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub weight: M3FontWeight,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum M3FontWeight {
    Regular, // 400
    Medium,  // 500
    Bold,    // 700
}

impl M3FontWeight {
    pub fn css_value(self) -> u16 {
        match self {
            Self::Regular => 400,
            Self::Medium => 500,
            Self::Bold => 700,
        }
    }
}

impl M3TextStyle {
    pub fn to_font_id(&self) -> FontId {
        FontId::proportional(self.font_size)
    }

    pub fn to_type_spec(&self) -> TypeSpec {
        let line_height = if self.font_size.is_finite()
            && self.font_size > 0.0
            && self.line_height.is_finite()
            && self.line_height > 0.0
        {
            let ratio = self.line_height / self.font_size;
            if ratio.is_finite() && ratio > 0.0 {
                ratio
            } else {
                TypeSpec::new(self.font_size).line_height
            }
        } else {
            TypeSpec::new(self.font_size).line_height
        };

        TypeSpec::new(self.font_size)
            .line_height(line_height)
            .letter_spacing(self.letter_spacing)
            .weight(self.weight.css_value())
    }
}

/// Complete M3 type scale — 15 styles.
#[derive(Clone, Debug)]
pub struct M3TypeScale {
    pub display_large: M3TextStyle,
    pub display_medium: M3TextStyle,
    pub display_small: M3TextStyle,
    pub headline_large: M3TextStyle,
    pub headline_medium: M3TextStyle,
    pub headline_small: M3TextStyle,
    pub title_large: M3TextStyle,
    pub title_medium: M3TextStyle,
    pub title_small: M3TextStyle,
    pub body_large: M3TextStyle,
    pub body_medium: M3TextStyle,
    pub body_small: M3TextStyle,
    pub label_large: M3TextStyle,
    pub label_medium: M3TextStyle,
    pub label_small: M3TextStyle,
}

impl Default for M3TypeScale {
    fn default() -> Self {
        // Values from M3 spec: https://m3.material.io/styles/typography/type-scale-tokens
        Self {
            display_large: M3TextStyle {
                font_size: 57.0,
                line_height: 64.0,
                letter_spacing: -0.25,
                weight: M3FontWeight::Regular,
            },
            display_medium: M3TextStyle {
                font_size: 45.0,
                line_height: 52.0,
                letter_spacing: 0.0,
                weight: M3FontWeight::Regular,
            },
            display_small: M3TextStyle {
                font_size: 36.0,
                line_height: 44.0,
                letter_spacing: 0.0,
                weight: M3FontWeight::Regular,
            },
            headline_large: M3TextStyle {
                font_size: 32.0,
                line_height: 40.0,
                letter_spacing: 0.0,
                weight: M3FontWeight::Regular,
            },
            headline_medium: M3TextStyle {
                font_size: 28.0,
                line_height: 36.0,
                letter_spacing: 0.0,
                weight: M3FontWeight::Regular,
            },
            headline_small: M3TextStyle {
                font_size: 24.0,
                line_height: 32.0,
                letter_spacing: 0.0,
                weight: M3FontWeight::Regular,
            },
            title_large: M3TextStyle {
                font_size: 22.0,
                line_height: 28.0,
                letter_spacing: 0.0,
                weight: M3FontWeight::Regular,
            },
            title_medium: M3TextStyle {
                font_size: 16.0,
                line_height: 24.0,
                letter_spacing: 0.15,
                weight: M3FontWeight::Medium,
            },
            title_small: M3TextStyle {
                font_size: 14.0,
                line_height: 20.0,
                letter_spacing: 0.1,
                weight: M3FontWeight::Medium,
            },
            body_large: M3TextStyle {
                font_size: 16.0,
                line_height: 24.0,
                letter_spacing: 0.5,
                weight: M3FontWeight::Regular,
            },
            body_medium: M3TextStyle {
                font_size: 14.0,
                line_height: 20.0,
                letter_spacing: 0.25,
                weight: M3FontWeight::Regular,
            },
            body_small: M3TextStyle {
                font_size: 12.0,
                line_height: 16.0,
                letter_spacing: 0.4,
                weight: M3FontWeight::Regular,
            },
            label_large: M3TextStyle {
                font_size: 14.0,
                line_height: 20.0,
                letter_spacing: 0.1,
                weight: M3FontWeight::Medium,
            },
            label_medium: M3TextStyle {
                font_size: 12.0,
                line_height: 16.0,
                letter_spacing: 0.5,
                weight: M3FontWeight::Medium,
            },
            label_small: M3TextStyle {
                font_size: 11.0,
                line_height: 16.0,
                letter_spacing: 0.5,
                weight: M3FontWeight::Medium,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r100_005a_m3_font_weight_maps_to_css_values() {
        assert_eq!(M3FontWeight::Regular.css_value(), 400);
        assert_eq!(M3FontWeight::Medium.css_value(), 500);
        assert_eq!(M3FontWeight::Bold.css_value(), 700);
    }

    #[test]
    fn r100_005a_m3_text_style_converts_to_type_spec() {
        let scale = M3TypeScale::default();

        let title = scale.title_medium.to_type_spec();
        assert_eq!(title.size, 16.0);
        assert_eq!(title.line_height, 1.5);
        assert_eq!(title.letter_spacing, 0.15);
        assert_eq!(title.weight, 500);

        let body = scale.body_large.to_type_spec();
        assert_eq!(body.size, 16.0);
        assert_eq!(body.line_height, 1.5);
        assert_eq!(body.letter_spacing, 0.5);
        assert_eq!(body.weight, 400);

        let label = scale.label_large.to_type_spec();
        assert_eq!(label.size, 14.0);
        assert_eq!(label.line_height, 20.0 / 14.0);
        assert_eq!(label.letter_spacing, 0.1);
        assert_eq!(label.weight, 500);
    }

    #[test]
    fn r100_005a_m3_text_style_invalid_line_height_inputs_use_default_ratio() {
        let zero_size = M3TextStyle {
            font_size: 0.0,
            line_height: 20.0,
            letter_spacing: 0.0,
            weight: M3FontWeight::Regular,
        };
        assert_eq!(zero_size.to_type_spec().line_height, 1.4);

        let negative_size = M3TextStyle {
            font_size: -12.0,
            line_height: 20.0,
            letter_spacing: 0.0,
            weight: M3FontWeight::Medium,
        };
        assert_eq!(negative_size.to_type_spec().line_height, 1.4);

        let overflowing_ratio = M3TextStyle {
            font_size: f32::MIN_POSITIVE,
            line_height: f32::MAX,
            letter_spacing: 0.0,
            weight: M3FontWeight::Bold,
        };
        assert_eq!(overflowing_ratio.to_type_spec().line_height, 1.4);
    }

    #[test]
    fn r100_005a_m3_font_id_remains_weight_agnostic() {
        let style = M3TextStyle {
            font_size: 14.0,
            line_height: 20.0,
            letter_spacing: 0.1,
            weight: M3FontWeight::Bold,
        };
        let font_id = style.to_font_id();

        assert_eq!(font_id.size, 14.0);
        assert!(matches!(font_id.family, egui::FontFamily::Proportional));
    }
}
