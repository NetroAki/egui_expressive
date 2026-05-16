use super::*;

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

/// A parsed color value ready for code generation.
#[derive(Clone, Debug)]
pub(crate) struct ParsedColor {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
    pub(crate) a: u8,
}

impl From<ParsedColor> for egui::Color32 {
    fn from(c: ParsedColor) -> Self {
        egui::Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
    }
}

impl ParsedColor {
    pub(crate) fn to_color32_expr(&self) -> String {
        format!(
            "Color32::from_rgba_unmultiplied({}, {}, {}, {})",
            self.r, self.g, self.b, self.a
        )
    }
}

/// Parse a hex color string like "#rrggbb" or "#rrggbbaa" into (r, g, b, a).
pub(crate) fn parse_hex_color(s: &str) -> Result<ParsedColor, FigmaExportError> {
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
pub(crate) fn parse_color_value(s: &str) -> Result<ParsedColor, FigmaExportError> {
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
pub(crate) struct FigmaStyleEntry {
    pub(crate) name: String,
    #[serde(rename = "styleType")]
    pub(crate) style_type: String,
    pub(crate) description: Option<String>,
}

/// Figma REST API styles response.
#[derive(Debug, Deserialize)]
pub(crate) struct FigmaStylesResponse {
    pub(crate) styles: Option<Vec<FigmaStyleEntry>>,
}

// ─── Token Extraction ──────────────────────────────────────────────────────────

/// Intermediate representation of extracted tokens.
#[derive(Default, Debug)]
pub(crate) struct ExtractedTokens {
    pub(crate) surface: std::collections::HashMap<String, ParsedColor>,
    pub(crate) accent: std::collections::HashMap<String, ParsedColor>,
    pub(crate) spacing: std::collections::HashMap<String, f32>,
    pub(crate) rounding: std::collections::HashMap<String, f32>,
}

impl ExtractedTokens {
    pub(crate) fn surface_color(&self, stop: &str) -> Option<ParsedColor> {
        self.surface.get(stop).cloned()
    }

    pub(crate) fn accent_color(&self, name: &str) -> Option<ParsedColor> {
        self.accent.get(name).cloned()
    }
}

/// Extract tokens from Figma Tokens plugin JSON format.
pub(crate) fn extract_tokens_from_figma_tokens(
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

pub(crate) fn extract_token_groups(
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

pub(crate) fn extract_color_value(value: &serde_json::Value) -> Option<ParsedColor> {
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

pub(crate) fn extract_spacing_value(value: &serde_json::Value) -> Option<f32> {
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

pub(crate) const SURFACE_STOPS: [&str; 13] = [
    "50", "100", "150", "200", "250", "300", "400", "500", "600", "700", "800", "900", "950",
];

pub(crate) const ACCENT_NAMES: [&str; 6] = ["glow", "active", "midi", "audio", "warn", "danger"];

pub(crate) const SPACING_NAMES: [&str; 6] = ["xs", "sm", "md", "lg", "xl", "xxl"];
