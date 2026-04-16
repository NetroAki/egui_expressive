#![allow(dead_code)]

use egui::{Color32, Context, CornerRadius, Id, Painter, Rect, Stroke, Vec2};

/// Runtime semantic colors following Material Design 3 principles.
#[derive(Clone, Debug)]
pub struct SemanticColors {
    pub surface: Color32,
    pub surface_dim: Color32,
    pub surface_bright: Color32,
    pub surface_container: Color32,
    pub on_surface: Color32,
    pub on_surface_variant: Color32,
    pub primary: Color32,
    pub on_primary: Color32,
    pub secondary: Color32,
    pub on_secondary: Color32,
    pub error: Color32,
    pub on_error: Color32,
    pub outline: Color32,
    pub outline_variant: Color32,
    pub scrim: Color32,
}

impl SemanticColors {
    /// Dark theme semantic colors (Material You dark scheme base).
    pub fn dark() -> Self {
        Self {
            surface: Color32::from_rgb(28, 27, 31),
            surface_dim: Color32::from_rgb(20, 20, 23),
            surface_bright: Color32::from_rgb(48, 48, 51),
            surface_container: Color32::from_rgb(35, 35, 40),
            on_surface: Color32::from_rgb(228, 226, 230),
            on_surface_variant: Color32::from_rgb(196, 196, 200),
            primary: Color32::from_rgb(187, 177, 255),
            on_primary: Color32::from_rgb(56, 48, 88),
            secondary: Color32::from_rgb(148, 166, 223),
            on_secondary: Color32::from_rgb(44, 47, 65),
            error: Color32::from_rgb(255, 180, 171),
            on_error: Color32::from_rgb(78, 36, 32),
            outline: Color32::from_rgb(96, 96, 100),
            outline_variant: Color32::from_rgb(72, 72, 76),
            scrim: Color32::from_rgb(0, 0, 0),
        }
    }

    /// Light theme semantic colors (Material You light scheme base).
    pub fn light() -> Self {
        Self {
            surface: Color32::from_rgb(255, 255, 255),
            surface_dim: Color32::from_rgb(250, 250, 250),
            surface_bright: Color32::from_rgb(255, 255, 255),
            surface_container: Color32::from_rgb(243, 239, 247),
            on_surface: Color32::from_rgb(28, 27, 31),
            on_surface_variant: Color32::from_rgb(73, 69, 79),
            primary: Color32::from_rgb(103, 80, 164),
            on_primary: Color32::from_rgb(255, 255, 255),
            secondary: Color32::from_rgb(73, 90, 135),
            on_secondary: Color32::from_rgb(255, 255, 255),
            error: Color32::from_rgb(179, 38, 30),
            on_error: Color32::from_rgb(255, 255, 255),
            outline: Color32::from_rgb(121, 116, 126),
            outline_variant: Color32::from_rgb(202, 196, 208),
            scrim: Color32::from_rgb(0, 0, 0),
        }
    }
}

/// Theme containing semantic colors and dark/light mode state.
#[derive(Clone, Debug)]
pub struct Theme {
    pub colors: SemanticColors,
    pub is_dark: bool,
}

impl Theme {
    /// Create a dark theme.
    pub fn dark() -> Self {
        Self {
            colors: SemanticColors::dark(),
            is_dark: true,
        }
    }

    /// Create a light theme.
    pub fn light() -> Self {
        Self {
            colors: SemanticColors::light(),
            is_dark: false,
        }
    }

    /// Store this theme in the context's temporary data storage.
    pub fn store(&self, ctx: &Context) {
        ctx.data_mut(|d| d.insert_temp(Id::new("__expressive_theme"), self.clone()));
    }

    /// Load the theme from context, defaulting to dark theme if not found.
    pub fn load(ctx: &Context) -> Self {
        ctx.data(|d| d.get_temp(Id::new("__expressive_theme")))
            .unwrap_or_else(Theme::dark)
    }

    /// Toggle between dark and light themes, swapping colors accordingly.
    pub fn toggle(ctx: &Context) {
        let current = Self::load(ctx);
        let new_theme = if current.is_dark {
            Theme::light()
        } else {
            Theme::dark()
        };
        new_theme.store(ctx);
    }
}

/// Named elevation levels for shadows, following Material Design 3 elevation system.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Elevation {
    Level0,
    Level1,
    Level2,
    Level3,
    Level4,
    Level5,
}

impl Elevation {
    /// Returns (blur_radius, spread, offset_y, alpha) for box_shadow.
    pub fn shadow_params(self) -> (f32, f32, f32, u8) {
        match self {
            Elevation::Level0 => (0.0, 0.0, 0.0, 0),
            Elevation::Level1 => (4.0, 0.0, 2.0, 30),
            Elevation::Level2 => (8.0, 0.0, 4.0, 40),
            Elevation::Level3 => (12.0, 0.0, 6.0, 50),
            Elevation::Level4 => (16.0, 0.0, 8.0, 60),
            Elevation::Level5 => (24.0, 0.0, 12.0, 70),
        }
    }

    /// Paint shadow and filled rect on painter.
    pub fn apply(self, painter: &Painter, rect: Rect, rounding: f32, fill: Color32) {
        let (blur_radius, _spread, offset_y, alpha) = self.shadow_params();
        let rounding_u8 = rounding.min(255.0) as u8;

        if alpha == 0 {
            // Level0: no shadow, just paint the rect
            painter.rect_filled(rect, CornerRadius::same(rounding_u8), fill);
            return;
        }

        let shadow_color = Color32::from_black_alpha(alpha);
        let shadow_offset = Vec2::new(0.0, offset_y);

        // Paint shadow (using filled rect with offset)
        painter.rect_filled(
            rect.translate(shadow_offset),
            CornerRadius::same(rounding_u8),
            shadow_color,
        );

        // Paint the filled rect on top
        painter.rect_filled(rect, CornerRadius::same(rounding_u8), fill);
    }
}

/// Border token with width and color.
#[derive(Clone, Copy, Debug)]
pub struct Border {
    pub width: f32,
    pub color: Color32,
}

impl Border {
    /// Subtle border using outline_variant color.
    pub fn subtle(ctx: &Context) -> Stroke {
        let theme = Theme::load(ctx);
        Stroke::new(1.0, theme.colors.outline_variant)
    }

    /// Default border using outline color.
    pub fn default_border(ctx: &Context) -> Stroke {
        let theme = Theme::load(ctx);
        Stroke::new(1.0, theme.colors.outline)
    }

    /// Focus border using primary color.
    pub fn focus(ctx: &Context) -> Stroke {
        let theme = Theme::load(ctx);
        Stroke::new(2.0, theme.colors.primary)
    }

    /// Danger border using error color.
    pub fn danger(ctx: &Context) -> Stroke {
        let theme = Theme::load(ctx);
        Stroke::new(1.0, theme.colors.error)
    }

    /// No border (transparent, zero width).
    pub fn none() -> Stroke {
        Stroke::new(0.0, Color32::TRANSPARENT)
    }
}

/// Convenience function to paint a border rect.
pub fn border_rect(painter: &Painter, rect: Rect, rounding: f32, stroke: Stroke) {
    if stroke.width > 0.0 {
        painter.rect_stroke(
            rect,
            CornerRadius::same(rounding.min(255.0) as u8),
            stroke,
            egui::StrokeKind::Outside,
        );
    }
}
