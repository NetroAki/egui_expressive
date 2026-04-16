use super::{M3ColorScheme, M3TypeScale};
use egui::{Context, Id};

/// Complete M3 theme — color scheme + type scale.
/// Stored in egui memory for global access.
#[derive(Clone, Debug)]
pub struct M3Theme {
    pub colors: M3ColorScheme,
    pub type_scale: M3TypeScale,
    pub is_dark: bool,
}

impl M3Theme {
    const MEMORY_ID: &'static str = "__m3_theme";

    pub fn dark() -> Self {
        Self {
            colors: M3ColorScheme::baseline_dark(),
            type_scale: M3TypeScale::default(),
            is_dark: true,
        }
    }

    pub fn light() -> Self {
        Self {
            colors: M3ColorScheme::baseline_light(),
            type_scale: M3TypeScale::default(),
            is_dark: false,
        }
    }

    pub fn from_seed(seed: egui::Color32, dark: bool) -> Self {
        Self {
            colors: M3ColorScheme::from_seed(seed, dark),
            type_scale: M3TypeScale::default(),
            is_dark: dark,
        }
    }

    /// Store this theme in egui context memory for global access.
    pub fn store(&self, ctx: &Context) {
        ctx.data_mut(|d| d.insert_temp(Id::new(Self::MEMORY_ID), self.clone()));
    }

    /// Load the theme from egui context memory.
    /// Falls back to dark baseline if not set.
    pub fn load(ctx: &Context) -> Self {
        ctx.data(|d| d.get_temp(Id::new(Self::MEMORY_ID)))
            .unwrap_or_else(Self::dark)
    }

    /// Apply M3 theme colors to egui's visual style.
    /// Call this once at app startup after storing the theme.
    pub fn apply_to_egui(&self, ctx: &Context) {
        let c = &self.colors;
        ctx.style_mut(|style| {
            let v = &mut style.visuals;
            v.dark_mode = self.is_dark;
            v.override_text_color = Some(c.on_surface);
            v.window_fill = c.surface_container;
            v.panel_fill = c.surface;
            v.faint_bg_color = c.surface_container_low;
            v.extreme_bg_color = c.surface_container_lowest;
            v.code_bg_color = c.surface_container_high;
            v.warn_fg_color = c.tertiary;
            v.error_fg_color = c.error;
            v.hyperlink_color = c.primary;
            v.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(
                c.primary.r(),
                c.primary.g(),
                c.primary.b(),
                40,
            );
            v.selection.stroke = egui::Stroke::new(1.0, c.primary);
            v.window_stroke = egui::Stroke::new(1.0, c.outline_variant);
            v.widgets.noninteractive.bg_fill = c.surface_container;
            v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, c.on_surface_variant);
            v.widgets.inactive.bg_fill = c.surface_container_high;
            v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, c.on_surface_variant);
            v.widgets.hovered.bg_fill = c.surface_container_highest;
            v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, c.on_surface);
            v.widgets.active.bg_fill = c.primary_container;
            v.widgets.active.fg_stroke = egui::Stroke::new(1.0, c.on_primary_container);
        });
    }
}
