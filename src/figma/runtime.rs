use super::*;
use crate::style::{AccentColors, SpacingScale, SurfacePalette};

/// Parse a Figma Tokens plugin JSON string directly into [`DesignTokens`] at runtime.
///
/// This is the runtime equivalent of [`figma_tokens_to_rust`] — instead of generating
/// Rust source code, it returns a live `DesignTokens` value you can use immediately.
pub fn design_tokens_from_json(json: &str) -> Result<DesignTokens, FigmaExportError> {
    let root: serde_json::Value =
        serde_json::from_str(json).map_err(|e| FigmaExportError::ParseError(e.to_string()))?;

    // Unwrap "global" wrapper if present
    let tokens = if let Some(global) = root.get("global") {
        global
    } else {
        &root
    };

    // Helper: extract color from a token value
    let get_color = |key: &str| -> egui::Color32 {
        tokens
            .get(key)
            .and_then(|v| v.get("value"))
            .and_then(|v| v.as_str())
            .and_then(|s| parse_color_value(s).ok())
            .map(Into::into)
            .unwrap_or_else(|| egui::Color32::from_gray(128))
    };

    // Helper: extract f32 spacing from a token value
    let get_spacing = |key: &str| -> f32 {
        tokens
            .get(key)
            .and_then(|v| v.get("value"))
            .and_then(|v| {
                if let Some(s) = v.as_str() {
                    s.trim_end_matches("px").parse::<f32>().ok()
                } else {
                    v.as_f64().map(|f| f as f32)
                }
            })
            .unwrap_or(0.0)
    };

    let surface = SurfacePalette {
        s50: get_color("surface-50"),
        s100: get_color("surface-100"),
        s150: get_color("surface-150"),
        s200: get_color("surface-200"),
        s250: get_color("surface-250"),
        s300: get_color("surface-300"),
        s400: get_color("surface-400"),
        s500: get_color("surface-500"),
        s600: get_color("surface-600"),
        s700: get_color("surface-700"),
        s800: get_color("surface-800"),
        s900: get_color("surface-900"),
        s950: get_color("surface-950"),
    };

    let accent = AccentColors {
        glow: get_color("accent-glow"),
        active: get_color("accent-active"),
        midi: get_color("accent-midi"),
        audio: get_color("accent-audio"),
        warn: get_color("accent-warn"),
        danger: get_color("accent-danger"),
    };

    let spacing = SpacingScale {
        xs: get_spacing("spacing-xs").max(2.0),
        sm: get_spacing("spacing-sm").max(4.0),
        md: get_spacing("spacing-md").max(8.0),
        lg: get_spacing("spacing-lg").max(12.0),
        xl: get_spacing("spacing-xl").max(16.0),
        xxl: get_spacing("spacing-xxl").max(24.0),
    };

    Ok(DesignTokens {
        surface,
        accent,
        spacing,
        rounding: get_spacing("rounding").max(0.0),
        panel_rounding: get_spacing("panel-rounding").max(0.0),
    })
}
