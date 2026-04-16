use super::color::blend_overlay;
use egui::Color32;

/// M3 elevation levels — tonal color overlay, NOT shadow-based.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum M3Elevation {
    Level0 = 0, // 0dp
    Level1 = 1, // 1dp
    Level2 = 2, // 3dp
    Level3 = 3, // 6dp
    Level4 = 4, // 8dp
    Level5 = 5, // 12dp
}

impl M3Elevation {
    /// M3 elevation uses a tonal color overlay on the surface color.
    /// Returns the surface color with the primary color tinted at the appropriate alpha.
    pub fn surface_tint(self, surface: Color32, primary: Color32) -> Color32 {
        let alpha = match self {
            Self::Level0 => 0.0,
            Self::Level1 => 0.05,
            Self::Level2 => 0.08,
            Self::Level3 => 0.11,
            Self::Level4 => 0.12,
            Self::Level5 => 0.14,
        };
        blend_overlay(surface, primary, alpha)
    }

    /// Shadow opacity for this elevation level (for optional drop shadows).
    pub fn shadow_opacity(self) -> f32 {
        match self {
            Self::Level0 => 0.0,
            Self::Level1 => 0.15,
            Self::Level2 => 0.20,
            Self::Level3 => 0.25,
            Self::Level4 => 0.30,
            Self::Level5 => 0.35,
        }
    }

    /// Approximate dp value for this elevation level.
    pub fn dp(self) -> f32 {
        match self {
            Self::Level0 => 0.0,
            Self::Level1 => 1.0,
            Self::Level2 => 3.0,
            Self::Level3 => 6.0,
            Self::Level4 => 8.0,
            Self::Level5 => 12.0,
        }
    }
}
