//! Elevation and shadow helpers for Tailwind-style surfaces.
//!
//! `Tw::shadow(Elevation::Level2)` routes through `egui::Frame::shadow`.
//! This matches utility-class semantics: one frame, one shadow, respecting the
//! frame's corner radius. For painter-level multi-shadow effects, use the draw
//! module's `box_shadow` helpers directly.

use egui::{Color32, Shadow};

use crate::theme::Elevation;

/// Convert a Material-style [`Elevation`] token into an egui frame shadow.
pub fn elevation_shadow(elevation: Elevation) -> Shadow {
    let (blur, spread, offset_y, alpha) = elevation.shadow_params();
    Shadow {
        offset: [0, offset_y.clamp(-128.0, 127.0).round() as i8],
        blur: blur.clamp(0.0, 255.0).round() as u8,
        spread: spread.clamp(0.0, 255.0).round() as u8,
        color: Color32::from_black_alpha(alpha),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elevation_level_zero_has_transparent_shadow() {
        assert_eq!(elevation_shadow(Elevation::Level0), Shadow::NONE);
    }
}
