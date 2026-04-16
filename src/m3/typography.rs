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

impl M3TextStyle {
    pub fn to_font_id(&self) -> FontId {
        FontId::proportional(self.font_size)
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
