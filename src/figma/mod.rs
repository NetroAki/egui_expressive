//! Figma Design Token Exporter
//!
//! Parses Figma Tokens plugin JSON exports (or Figma REST API style exports)
//! and emits Rust source code for [`crate::style::DesignTokens`].

use serde::Deserialize;

/// Errors that can occur during Figma token export.
#[derive(Debug)]
pub enum FigmaExportError {
    /// JSON parsing failed.
    ParseError(String),
    /// A required field is missing from the token JSON.
    MissingField(String),
    /// The color value could not be parsed.
    InvalidColor(String),
}

impl std::fmt::Display for FigmaExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FigmaExportError::ParseError(s) => write!(f, "Parse error: {}", s),
            FigmaExportError::MissingField(s) => write!(f, "Missing field: {}", s),
            FigmaExportError::InvalidColor(s) => write!(f, "Invalid color: {}", s),
        }
    }
}

impl std::error::Error for FigmaExportError {}

// ─── Figma Tokens Plugin JSON Structures ────────────────────────────────────────

/// A single token entry in Figma Tokens plugin format.
#[derive(Debug, Deserialize)]
struct TokenEntry {
    value: TokenValue,
    #[serde(rename = "type")]
    token_type: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TokenValue {
    String(String),
    Object(TokenObject),
}

#[derive(Debug, Deserialize)]
struct TokenObject {
    value: Option<String>,
    #[serde(rename = "type")]
    token_type: Option<String>,
}

/// Top-level structure for Figma Tokens plugin JSON (with "global" wrapper).
#[derive(Debug, Deserialize)]
struct FigmaTokensGlobal {
    global: Option<FigmaTokenGroups>,
}

/// Direct token groups without "global" wrapper.
#[derive(Debug, Deserialize)]
struct FigmaTokenGroups {
    #[serde(flatten)]
    groups: std::collections::HashMap<String, serde_json::Value>,
}

/// A parsed color value ready for code generation.
#[derive(Clone, Debug)]
struct ParsedColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl ParsedColor {
    fn to_color32_expr(&self) -> String {
        format!(
            "Color32::from_rgba_unmultiplied({}, {}, {}, {})",
            self.r, self.g, self.b, self.a
        )
    }
}

/// Parse a hex color string like "#rrggbb" or "#rrggbbaa" into (r, g, b, a).
fn parse_hex_color(s: &str) -> Result<ParsedColor, FigmaExportError> {
    let s = s.trim();
    if !s.starts_with('#') {
        return Err(FigmaExportError::InvalidColor(format!(
            "Expected '#' prefix, got: {}",
            s
        )));
    }
    let hex = &s[1..];
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| {
                FigmaExportError::InvalidColor(format!("Invalid hex: {}", &hex[0..2]))
            })?;
            let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| {
                FigmaExportError::InvalidColor(format!("Invalid hex: {}", &hex[2..4]))
            })?;
            let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| {
                FigmaExportError::InvalidColor(format!("Invalid hex: {}", &hex[4..6]))
            })?;
            Ok(ParsedColor { r, g, b, a: 255 })
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| {
                FigmaExportError::InvalidColor(format!("Invalid hex: {}", &hex[0..2]))
            })?;
            let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| {
                FigmaExportError::InvalidColor(format!("Invalid hex: {}", &hex[2..4]))
            })?;
            let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| {
                FigmaExportError::InvalidColor(format!("Invalid hex: {}", &hex[4..6]))
            })?;
            let a = u8::from_str_radix(&hex[6..8], 16).map_err(|_| {
                FigmaExportError::InvalidColor(format!("Invalid hex: {}", &hex[6..8]))
            })?;
            Ok(ParsedColor { r, g, b, a })
        }
        _ => Err(FigmaExportError::InvalidColor(format!(
            "Expected 6 or 8 hex chars, got {} ('{}')",
            hex.len(),
            s
        ))),
    }
}

/// Parse a color from various CSS/RGB formats: #rrggbb, #rrggbbaa, rgb(r,g,b), rgba(r,g,b,a).
fn parse_color_value(s: &str) -> Result<ParsedColor, FigmaExportError> {
    let s = s.trim();

    // Try hex format first
    if s.starts_with('#') {
        return parse_hex_color(s);
    }

    // Try rgb/rgba format
    if s.starts_with("rgba") || s.starts_with("rgb") {
        let inner = s
            .trim_start_matches("rgba")
            .trim_start_matches("rgb")
            .trim_matches(|c| c == '(' || c == ')' || c == ' ');
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 3 || parts.len() == 4 {
            let r: u8 = parts[0].trim().parse().map_err(|_| {
                FigmaExportError::InvalidColor(format!("Invalid r value: {}", parts[0]))
            })?;
            let g: u8 = parts[1].trim().parse().map_err(|_| {
                FigmaExportError::InvalidColor(format!("Invalid g value: {}", parts[1]))
            })?;
            let b: u8 = parts[2].trim().parse().map_err(|_| {
                FigmaExportError::InvalidColor(format!("Invalid b value: {}", parts[2]))
            })?;
            let a: u8 = if parts.len() == 4 {
                // Parse alpha as 0-1 float or 0-255 int
                let alpha_str = parts[3].trim();
                if alpha_str.contains('.') {
                    let alpha: f32 = alpha_str.parse().map_err(|_| {
                        FigmaExportError::InvalidColor(format!("Invalid alpha: {}", alpha_str))
                    })?;
                    (alpha.clamp(0.0, 1.0) * 255.0) as u8
                } else {
                    alpha_str.parse().map_err(|_| {
                        FigmaExportError::InvalidColor(format!("Invalid alpha: {}", alpha_str))
                    })?
                }
            } else {
                255
            };
            return Ok(ParsedColor { r, g, b, a });
        }
    }

    Err(FigmaExportError::InvalidColor(format!(
        "Unrecognized color format: {}",
        s
    )))
}

// ─── Figma REST API Style Export Structures ────────────────────────────────────

/// Simplified style entry from Figma REST API `/v1/files/:key/styles`.
#[derive(Debug, Deserialize)]
struct FigmaStyleEntry {
    name: String,
    #[serde(rename = "styleType")]
    style_type: String,
    description: Option<String>,
}

/// Figma REST API styles response.
#[derive(Debug, Deserialize)]
struct FigmaStylesResponse {
    styles: Option<Vec<FigmaStyleEntry>>,
}

// ─── Token Extraction ──────────────────────────────────────────────────────────

/// Intermediate representation of extracted tokens.
#[derive(Default, Debug)]
struct ExtractedTokens {
    surface: std::collections::HashMap<String, ParsedColor>,
    accent: std::collections::HashMap<String, ParsedColor>,
    spacing: std::collections::HashMap<String, f32>,
    rounding: std::collections::HashMap<String, f32>,
}

impl ExtractedTokens {
    fn surface_color(&self, stop: &str) -> Option<ParsedColor> {
        self.surface.get(stop).cloned()
    }

    fn accent_color(&self, name: &str) -> Option<ParsedColor> {
        self.accent.get(name).cloned()
    }
}

/// Extract tokens from Figma Tokens plugin JSON format.
fn extract_tokens_from_figma_tokens(
    json: &serde_json::Value,
) -> Result<ExtractedTokens, FigmaExportError> {
    let mut tokens = ExtractedTokens::default();

    // Try "global" wrapper first
    if let Some(global) = json.get("global") {
        extract_token_groups(global, &mut tokens)?;
        return Ok(tokens);
    }

    // Otherwise treat root as token groups
    extract_token_groups(json, &mut tokens)?;
    Ok(tokens)
}

fn extract_token_groups(
    root: &serde_json::Value,
    tokens: &mut ExtractedTokens,
) -> Result<(), FigmaExportError> {
    if let Some(obj) = root.as_object() {
        for (group_name, group_value) in obj {
            if let Some(group) = group_value.as_object() {
                match group_name.as_str() {
                    "surface" => {
                        for (stop, value) in group {
                            if let Some(color) = extract_color_value(value) {
                                tokens.surface.insert(stop.clone(), color);
                            }
                        }
                    }
                    "accent" => {
                        for (name, value) in group {
                            if let Some(color) = extract_color_value(value) {
                                tokens.accent.insert(name.clone(), color);
                            }
                        }
                    }
                    "spacing" => {
                        for (name, value) in group {
                            if let Some(spacing) = extract_spacing_value(value) {
                                tokens.spacing.insert(name.clone(), spacing);
                            }
                        }
                    }
                    "rounding" => {
                        for (name, value) in group {
                            if let Some(r) = extract_spacing_value(value) {
                                tokens.rounding.insert(name.clone(), r);
                            }
                        }
                    }
                    _ => {
                        // Skip unknown groups
                    }
                }
            }
        }
    }
    Ok(())
}

fn extract_color_value(value: &serde_json::Value) -> Option<ParsedColor> {
    // Figma Tokens plugin format: { "value": "#rrggbb", "type": "color" }
    if let Some(obj) = value.as_object() {
        if let Some(val) = obj.get("value").and_then(|v| v.as_str()) {
            if let Ok(color) = parse_color_value(val) {
                return Some(color);
            }
        }
        // Alternative: direct string value
        if let Some(val) = value.as_str() {
            if let Ok(color) = parse_color_value(val) {
                return Some(color);
            }
        }
    }
    // Direct string
    if let Some(val) = value.as_str() {
        parse_color_value(val).ok()
    } else {
        None
    }
}

fn extract_spacing_value(value: &serde_json::Value) -> Option<f32> {
    if let Some(obj) = value.as_object() {
        if let Some(val) = obj.get("value") {
            if let Some(s) = val.as_str() {
                return s.trim().parse::<f32>().ok();
            }
            if let Some(n) = val.as_f64() {
                return Some(n as f32);
            }
        }
    }
    if let Some(s) = value.as_str() {
        s.trim().parse::<f32>().ok()
    } else {
        None
    }
}

// ─── Code Generation ──────────────────────────────────────────────────────────

const SURFACE_STOPS: [&str; 13] = [
    "50", "100", "150", "200", "250", "300", "400", "500", "600", "700", "800", "900", "950",
];

const ACCENT_NAMES: [&str; 6] = ["glow", "active", "midi", "audio", "warn", "danger"];

const SPACING_NAMES: [&str; 6] = ["xs", "sm", "md", "lg", "xl", "xxl"];

const ROUNDING_NAMES: [&str; 4] = ["sm", "md", "lg", "full"];

/// Generate the Rust source code from extracted tokens.
fn generate_rust_code(tokens: &ExtractedTokens) -> String {
    let mut out = String::new();

    out.push_str("// Auto-generated by egui_expressive figma exporter\n");
    out.push_str("// Source: Figma Tokens export\n\n");

    out.push_str(
        "use egui_expressive::{AccentColors, DesignTokens, SpacingScale, SurfacePalette};\n",
    );
    out.push_str("use egui::Color32;\n\n");

    out.push_str("pub fn design_tokens() -> DesignTokens {\n");
    out.push_str("    DesignTokens {\n");

    // Surface palette
    out.push_str("        surface: SurfacePalette {\n");
    for stop in &SURFACE_STOPS {
        let color = tokens.surface_color(stop).unwrap_or_else(|| ParsedColor {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        });
        out.push_str(&format!(
            "            s{}: {},\n",
            stop,
            color.to_color32_expr()
        ));
    }
    out.push_str("        },\n");

    // Accent colors
    out.push_str("        accent: AccentColors {\n");
    for name in &ACCENT_NAMES {
        let color = tokens.accent_color(name).unwrap_or_else(|| ParsedColor {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        });
        out.push_str(&format!(
            "            {}: {},\n",
            name,
            color.to_color32_expr()
        ));
    }
    out.push_str("        },\n");

    // Spacing scale
    out.push_str("        spacing: SpacingScale {\n");
    for name in &SPACING_NAMES {
        let value = tokens.spacing.get(*name).copied().unwrap_or(0.0);
        out.push_str(&format!("            {}: {},\n", name, value));
    }
    out.push_str("        },\n");

    // Rounding: use "md" as base rounding, "lg" as panel_rounding
    let rounding = tokens.rounding.get("md").copied().unwrap_or(4.0);
    let panel_rounding = tokens.rounding.get("lg").copied().unwrap_or(8.0);
    out.push_str(&format!("        rounding: {},\n", rounding));
    out.push_str(&format!("        panel_rounding: {},\n", panel_rounding));

    out.push_str("    }\n");
    out.push_str("}\n");

    out
}

/// Parse Figma Tokens plugin JSON (or Figma REST API styles JSON) and emit Rust source
/// code for [`crate::style::DesignTokens`].
pub fn figma_tokens_to_rust(json: &str) -> Result<String, FigmaExportError> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| FigmaExportError::ParseError(e.to_string()))?;

    // Check if this is a Figma REST API styles response
    if let Some(styles_response) = value.get("styles") {
        if let Ok(resp) = serde_json::from_str::<FigmaStylesResponse>(json) {
            if let Some(styles) = resp.styles {
                return generate_from_figma_styles(&styles);
            }
        }
        // If styles exists but didn't parse as expected, try token groups anyway
    }

    // Try as Figma Tokens plugin JSON
    let tokens = extract_tokens_from_figma_tokens(&value)?;
    Ok(generate_rust_code(&tokens))
}

/// Generate Rust code from Figma REST API styles response.
/// This handles the simpler FILL-only export where we don't have actual color values
/// stored — we emit placeholder comments for manual fill-in.
fn generate_from_figma_styles(styles: &[FigmaStyleEntry]) -> Result<String, FigmaExportError> {
    let mut out = String::new();

    out.push_str("// Auto-generated by egui_expressive figma exporter\n");
    out.push_str("// Source: Figma REST API styles export\n");
    out.push_str("// NOTE: Figma REST API does not include actual color values.\n");
    out.push_str("//       Replace the placeholder Color32::DEFAULT values below.\n\n");

    out.push_str(
        "use egui_expressive::{AccentColors, DesignTokens, SpacingScale, SurfacePalette};\n",
    );
    out.push_str("use egui::Color32;\n\n");

    out.push_str("pub fn design_tokens() -> DesignTokens {\n");
    out.push_str("    DesignTokens {\n");

    // Group styles by prefix
    let mut surface_entries = Vec::new();
    let mut accent_entries = Vec::new();

    for style in styles {
        if style.style_type != "FILL" {
            continue;
        }
        let parts: Vec<&str> = style.name.split('/').collect();
        if parts.len() >= 2 {
            let prefix = parts[0];
            let name = parts[1..].join("/");
            match prefix {
                "surface" => surface_entries.push((name, style.description.clone())),
                "accent" => accent_entries.push((name, style.description.clone())),
                _ => {}
            }
        }
    }

    // Surface palette — emit all stops found, placeholder for rest
    out.push_str("        surface: SurfacePalette {\n");
    for stop in &SURFACE_STOPS {
        let entry = surface_entries.iter().find(|(n, _)| n == *stop);
        if let Some((_, desc)) = entry {
            let comment = desc.as_deref().unwrap_or("");
            if !comment.is_empty() {
                out.push_str(&format!(
                    "            // {}: \"{}\"\n",
                    format!("s{}", stop),
                    comment
                ));
            }
            out.push_str(&format!(
                "            s{}: Color32::from_rgba_unmultiplied(0, 0, 0, 255), // TODO: fill\n",
                stop
            ));
        } else {
            out.push_str(&format!(
                "            s{}: Color32::from_rgba_unmultiplied(0, 0, 0, 255), // TODO: fill\n",
                stop
            ));
        }
    }
    out.push_str("        },\n");

    // Accent colors
    out.push_str("        accent: AccentColors {\n");
    for name in &ACCENT_NAMES {
        let entry = accent_entries.iter().find(|(n, _)| n == name);
        if let Some((_, desc)) = entry {
            let comment = desc.as_deref().unwrap_or("");
            if !comment.is_empty() {
                out.push_str(&format!("            // {}: \"{}\"\n", name, comment));
            }
        }
        out.push_str(&format!(
            "            {}: Color32::from_rgba_unmultiplied(0, 0, 0, 255), // TODO: fill\n",
            name
        ));
    }
    out.push_str("        },\n");

    // Default spacing and rounding (no data from REST API)
    out.push_str("        spacing: SpacingScale {\n");
    for name in &SPACING_NAMES {
        out.push_str(&format!("            {}: 0.0, // TODO: fill\n", name));
    }
    out.push_str("        },\n");

    out.push_str("        rounding: 4.0, // TODO: fill\n");
    out.push_str("        panel_rounding: 8.0, // TODO: fill\n");

    out.push_str("    }\n");
    out.push_str("}\n");

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color_6() {
        let color = parse_hex_color("#ff8040").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
        assert_eq!(color.a, 255);
    }

    #[test]
    fn test_parse_hex_color_8() {
        let color = parse_hex_color("#ff804080").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
        assert_eq!(color.a, 128);
    }

    #[test]
    fn test_parse_color_value_rgb() {
        let color = parse_color_value("rgb(255, 128, 64)").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
        assert_eq!(color.a, 255);
    }

    #[test]
    fn test_parse_color_value_rgba() {
        let color = parse_color_value("rgba(255, 128, 64, 0.5)").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
        assert_eq!(color.a, 127); // 0.5 * 255 = 127.5 truncates to 127
    }

    #[test]
    fn test_figma_tokens_basic() {
        let json = r##"{
            "global": {
                "surface": {
                    "50": { "value": "#f8f8f8", "type": "color" },
                    "950": { "value": "#0a0a0a", "type": "color" }
                },
                "accent": {
                    "glow": { "value": "#7c3aed", "type": "color" }
                },
                "spacing": {
                    "md": { "value": "8", "type": "spacing" }
                },
                "rounding": {
                    "md": { "value": "4", "type": "borderRadius" }
                }
            }
        }"##;

        let result = figma_tokens_to_rust(json);
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
        let code = result.unwrap();
        assert!(code.contains("pub fn design_tokens()"));
        assert!(code.contains("SurfacePalette"));
        assert!(code.contains("AccentColors"));
    }

    #[test]
    fn test_figma_tokens_no_wrapper() {
        let json = r##"{
            "surface": {
                "50": { "value": "#ffffff", "type": "color" }
            },
            "accent": {
                "glow": { "value": "#000000", "type": "color" }
            }
        }"##;

        let result = figma_tokens_to_rust(json);
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    }
}
