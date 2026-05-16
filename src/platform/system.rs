//! System-theme and display-scale descriptors.

/// Theme preference observed or selected by the host app.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemThemePreference {
    System,
    Light,
    Dark,
    HighContrast,
}

impl SystemThemePreference {
    pub fn prefers_dark(self) -> Option<bool> {
        match self {
            Self::Dark => Some(true),
            Self::Light | Self::HighContrast => Some(false),
            Self::System => None,
        }
    }
}

/// High-DPI scale fact from egui `pixels_per_point` or native viewport setup.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisplayScale {
    pub pixels_per_point: f32,
}

impl DisplayScale {
    pub fn new(pixels_per_point: f32) -> Self {
        Self {
            pixels_per_point: pixels_per_point.max(0.1),
        }
    }

    pub fn logical_to_physical(self, logical_points: f32) -> f32 {
        logical_points * self.pixels_per_point
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_scale_converts_logical_points() {
        assert_eq!(DisplayScale::new(2.0).logical_to_physical(12.0), 24.0);
    }

    #[test]
    fn display_scale_clamps_low_pixels_per_point() {
        assert_eq!(DisplayScale::new(0.0).pixels_per_point, 0.1);
    }

    #[test]
    fn system_theme_prefers_dark_mapping_covers_all_variants() {
        assert_eq!(SystemThemePreference::System.prefers_dark(), None);
        assert_eq!(SystemThemePreference::Light.prefers_dark(), Some(false));
        assert_eq!(SystemThemePreference::Dark.prefers_dark(), Some(true));
        assert_eq!(
            SystemThemePreference::HighContrast.prefers_dark(),
            Some(false)
        );
    }
}
