//! Adobe Illustrator (.ai) file parser
//!
//! Parses .ai files (PDF wrappers) and extracts visual properties from AIPrivateData streams.

use lopdf::Document;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

static ARTBOARD_RE: OnceLock<Regex> = OnceLock::new();
static ARTBOARD_NAME_RE: OnceLock<Regex> = OnceLock::new();

fn artboard_re() -> &'static Regex {
    ARTBOARD_RE.get_or_init(|| {
        Regex::new(
            r"%AI9_Artboard\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)"
        ).expect("valid artboard regex")
    })
}

fn artboard_name_re() -> &'static Regex {
    ARTBOARD_NAME_RE.get_or_init(|| {
        Regex::new(r"%AI9_ArtboardName\s+([^\n]+)").expect("valid artboard name regex")
    })
}

use egui_expressive::codegen::{generate_artboard_file, LayoutElement, ElementType};

/// RGBA Color representation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
    #[serde(default)]
    pub opacity: f64,
    #[serde(default)]
    pub blend_mode: String,
}

/// Stroke representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stroke {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
    #[serde(default)]
    pub width: f64,
    #[serde(default)]
    pub opacity: f64,
}

/// Live Effect parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveEffectParams {
    #[serde(flatten)]
    pub params: HashMap<String, Value>,
}

/// Live Effect representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveEffect {
    pub name: String,
    pub params: LiveEffectParams,
}

/// Mesh patch corner and color
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshPatch {
    pub corners: Vec<Vec<f64>>,
    pub colors: Vec<Vec<u8>>,
}

/// Envelope mesh representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvelopeMesh {
    pub rows: usize,
    pub cols: usize,
    pub points: Vec<Vec<f64>>,
}

/// 3D effect representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreeD {
    #[serde(rename = "type")]
    pub effect_type: String,
    pub depth: f64,
    #[serde(rename = "rotation_x")]
    pub rotation_x: f64,
    #[serde(rename = "rotation_y")]
    pub rotation_y: f64,
    #[serde(rename = "rotation_z")]
    pub rotation_z: f64,
}

/// Artboard representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artboard {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// A Bezier path point with anchor and control handles
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PathPoint {
    pub anchor: [f64; 2],
    pub left_ctrl: [f64; 2],
    pub right_ctrl: [f64; 2],
}

/// Element representation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Element {
    pub id: String,
    #[serde(default)]
    pub live_effects: Vec<LiveEffect>,
    #[serde(default)]
    pub appearance_fills: Vec<Color>,
    #[serde(default)]
    pub appearance_strokes: Vec<Stroke>,
    #[serde(default)]
    pub mesh_patches: Vec<MeshPatch>,
    #[serde(default)]
    pub envelope_mesh: Option<EnvelopeMesh>,
    #[serde(default)]
    pub three_d: Option<ThreeD>,
    #[serde(default)]
    pub rotation_deg: f64,
    #[serde(default)]
    pub scale_x: f64,
    #[serde(default)]
    pub scale_y: f64,
    #[serde(default)]
    pub translate_x: f64,
    #[serde(default)]
    pub translate_y: f64,
    #[serde(default)]
    pub corner_radius: f64,
    #[serde(default)]
    pub path_points: Vec<PathPoint>,
    #[serde(default)]
    pub path_closed: bool,
    #[serde(default)]
    pub artboard_name: Option<String>,
}

/// Parsed AI file result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiParseResult {
    pub version: String,
    #[serde(rename = "source_file")]
    pub source_file: String,
    #[serde(rename = "ai_version")]
    pub ai_version: String,
    #[serde(default)]
    pub artboards: Vec<Artboard>,
    #[serde(default)]
    pub elements: Vec<Element>,
    #[serde(default)]
    pub errors: Vec<String>,
}

/// Parse LiveEffect Dict data string format: R key value R key value I key value B key value
fn parse_dict_data(data: &str) -> HashMap<String, Value> {
    let mut result = HashMap::new();
    let mut chars = data.chars().peekable();
    let mut current_type = None;

    while let Some(c) = chars.next() {
        match c {
            'R' | 'I' | 'B' | 'S' => {
                current_type = Some(c);
                while let Some(&' ') = chars.peek() {
                    chars.next();
                }
            }
            ' ' | '\n' | '\t' | '\r' => {}
            _ => {
                if current_type.is_some() {
                    if c == '"' {
                        let mut key = String::new();
                        for c in chars.by_ref() {
                            if c == '"' {
                                break;
                            }
                            key.push(c);
                        }
                        while let Some(&' ') = chars.peek() {
                            chars.next();
                        }
                        if let Some(t) = current_type {
                            let mut value_str = String::new();
                            while let Some(&next) = chars.peek() {
                                if next == 'R'
                                    || next == 'I'
                                    || next == 'B'
                                    || next == 'S'
                                    || next == '"'
                                {
                                    break;
                                }
                                value_str.push(chars.next().unwrap());
                            }
                            let value_str = value_str.trim();
                            if !key.is_empty() && !value_str.is_empty() {
                                let value = match t {
                                    'R' => value_str
                                        .parse::<f64>()
                                        .map(Value::from)
                                        .unwrap_or(Value::String(value_str.to_string())),
                                    'I' => value_str
                                        .parse::<i64>()
                                        .map(Value::from)
                                        .unwrap_or(Value::String(value_str.to_string())),
                                    'B' => Value::Bool(
                                        value_str == "1" || value_str.to_lowercase() == "true",
                                    ),
                                    'S' => Value::String(value_str.to_string()),
                                    _ => Value::String(value_str.to_string()),
                                };
                                result.insert(key, value);
                            }
                        }
                        current_type = None;
                    } else {
                        let mut key = String::new();
                        key.push(c);
                        while let Some(&next) = chars.peek() {
                            if next == 'R'
                                || next == 'I'
                                || next == 'B'
                                || next == 'S'
                                || next.is_whitespace()
                            {
                                break;
                            }
                            key.push(chars.next().unwrap());
                        }
                        let key = key.trim().to_string();
                        if !key.is_empty() && key.chars().all(|ch| ch.is_alphabetic() || ch == '_')
                        {
                            while let Some(&' ') = chars.peek() {
                                chars.next();
                            }
                            if let Some(t) = current_type {
                                let mut value_str = String::new();
                                while let Some(&next) = chars.peek() {
                                    if next == 'R'
                                        || next == 'I'
                                        || next == 'B'
                                        || next == 'S'
                                        || next == '"'
                                    {
                                        break;
                                    }
                                    value_str.push(chars.next().unwrap());
                                }
                                let value_str = value_str.trim().to_string();
                                if !value_str.is_empty() {
                                    let value = match t {
                                        'R' => value_str
                                            .parse::<f64>()
                                            .map(Value::from)
                                            .unwrap_or(Value::String(value_str.to_string())),
                                        'I' => value_str
                                            .parse::<i64>()
                                            .map(Value::from)
                                            .unwrap_or(Value::String(value_str.to_string())),
                                        'B' => Value::Bool(
                                            value_str == "1" || value_str.to_lowercase() == "true",
                                        ),
                                        'S' => Value::String(value_str.to_string()),
                                        _ => Value::String(value_str.to_string()),
                                    };
                                    result.insert(key, value);
                                }
                            }
                            current_type = None;
                        }
                    }
                }
            }
        }
    }
    result
}

/// Parse LiveEffect XML-like content
fn parse_live_effect_xml(content: &str) -> Vec<LiveEffect> {
    let mut effects = Vec::new();

    let live_effect_re =
        Regex::new(r#"<LiveEffect\s+name="([^"]+)"[^>]*>.*?<Dict\s+data="([^"]+)""#).ok();
    if let Some(re) = live_effect_re {
        for caps in re.captures_iter(content) {
            if let (Some(name), Some(data)) = (caps.get(1), caps.get(2)) {
                let params = parse_dict_data(data.as_str());
                effects.push(LiveEffect {
                    name: name.as_str().to_string(),
                    params: LiveEffectParams { params },
                });
            }
        }
    }

    let alt_re = Regex::new(r#"<LiveEffect\s+name="([^"]+)"[^/]*/?>"#).ok();
    if let Some(re) = alt_re {
        for caps in re.captures_iter(content) {
            if let Some(name) = caps.get(1) {
                if !effects.iter().any(|e: &LiveEffect| e.name == name.as_str()) {
                    effects.push(LiveEffect {
                        name: name.as_str().to_string(),
                        params: LiveEffectParams {
                            params: HashMap::new(),
                        },
                    });
                }
            }
        }
    }

    effects
}

/// Parse AI9_EnvelopeMesh content
fn parse_envelope_mesh(content: &str) -> Option<EnvelopeMesh> {
    let envelope_re = Regex::new(r"%AI9_EnvelopeMesh\s*\n?\s*(\d+)\s+(\d+)").ok()?;

    let caps = envelope_re.captures(content)?;
    let rows: usize = caps.get(1)?.as_str().parse().ok()?;
    let cols: usize = caps.get(2)?.as_str().parse().ok()?;

    let value_re = Regex::new(r"-?\d+\.?\d*").ok()?;
    let values: Vec<f64> = value_re
        .find_iter(content)
        .filter_map(|m| m.as_str().parse().ok())
        .collect();

    let mut points = Vec::new();
    let mut idx = 2;
    while points.len() < rows * cols && idx + 1 < values.len() {
        points.push(vec![values[idx], values[idx + 1]]);
        idx += 2;
    }

    Some(EnvelopeMesh { rows, cols, points })
}

/// Parse AI9_3D_Extrude or AI9_3D_Revolve content
fn parse_3d_effect(content: &str) -> Option<ThreeD> {
    let is_extrude = content.contains("%AI9_3D_Extrude");
    let is_revolve = content.contains("%AI9_3D_Revolve");
    let effect_type = if is_extrude {
        "extrude"
    } else if is_revolve {
        "revolve"
    } else {
        return None;
    };

    // Find the marker and get numbers only after it
    let marker = if is_extrude {
        "%AI9_3D_Extrude"
    } else {
        "%AI9_3D_Revolve"
    };
    if let Some(pos) = content.find(marker) {
        let after_marker = &content[pos + marker.len()..];
        let value_re = Regex::new(r"-?\d+\.?\d*").ok()?;
        let values: Vec<f64> = value_re
            .find_iter(after_marker)
            .filter_map(|m| m.as_str().parse().ok())
            .collect();

        let depth = values.first().copied().unwrap_or(100.0);
        let rotation_x = values.get(1).copied().unwrap_or(0.0);
        let rotation_y = values.get(2).copied().unwrap_or(0.0);
        let rotation_z = values.get(3).copied().unwrap_or(0.0);

        return Some(ThreeD {
            effect_type: effect_type.to_string(),
            depth,
            rotation_x,
            rotation_y,
            rotation_z,
        });
    }

    None
}

/// Parse fill/stroke appearance from PostScript content
fn parse_appearance(content: &str) -> (Vec<Color>, Vec<Stroke>) {
    let mut fills = Vec::new();
    let mut strokes = Vec::new();

    let fill_stroke_re = Regex::new(
        r"\[\s*(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s*\]\s*([Xx][aA])",
    )
    .ok();

    if let Some(re) = fill_stroke_re {
        for caps in re.captures_iter(content) {
            if let (Some(r), Some(g), Some(b), Some(a), Some(op)) = (
                caps.get(1),
                caps.get(2),
                caps.get(3),
                caps.get(4),
                caps.get(5),
            ) {
                let r_val =
                    (r.as_str().parse::<f64>().unwrap_or(0.0) * 255.0).clamp(0.0, 255.0) as u8;
                let g_val =
                    (g.as_str().parse::<f64>().unwrap_or(0.0) * 255.0).clamp(0.0, 255.0) as u8;
                let b_val =
                    (b.as_str().parse::<f64>().unwrap_or(0.0) * 255.0).clamp(0.0, 255.0) as u8;
                let a_val =
                    (a.as_str().parse::<f64>().unwrap_or(1.0) * 255.0).clamp(0.0, 255.0) as u8;
                let op_str = op.as_str().to_lowercase();

                if op_str.ends_with('a') && !op_str.starts_with('x') {
                    fills.push(Color {
                        r: r_val,
                        g: g_val,
                        b: b_val,
                        a: a_val,
                        opacity: 1.0,
                        blend_mode: "normal".to_string(),
                    });
                } else if op_str.ends_with('a') && op_str.starts_with('x') {
                    strokes.push(Stroke {
                        r: r_val,
                        g: g_val,
                        b: b_val,
                        a: a_val,
                        width: 1.0,
                        opacity: 1.0,
                    });
                }
            }
        }
    }

    let xa_re =
        Regex::new(r"(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+Xa\b").ok();
    if let Some(re) = xa_re {
        for caps in re.captures_iter(content) {
            if let (Some(r), Some(g), Some(b), Some(a)) =
                (caps.get(1), caps.get(2), caps.get(3), caps.get(4))
            {
                let r_val =
                    (r.as_str().parse::<f64>().unwrap_or(0.0) * 255.0).clamp(0.0, 255.0) as u8;
                let g_val =
                    (g.as_str().parse::<f64>().unwrap_or(0.0) * 255.0).clamp(0.0, 255.0) as u8;
                let b_val =
                    (b.as_str().parse::<f64>().unwrap_or(0.0) * 255.0).clamp(0.0, 255.0) as u8;
                let a_val =
                    (a.as_str().parse::<f64>().unwrap_or(1.0) * 255.0).clamp(0.0, 255.0) as u8;

                if !fills
                    .iter()
                    .any(|f| f.r == r_val && f.g == g_val && f.b == b_val && f.a == a_val)
                {
                    fills.push(Color {
                        r: r_val,
                        g: g_val,
                        b: b_val,
                        a: a_val,
                        opacity: 1.0,
                        blend_mode: "normal".to_string(),
                    });
                }
            }
        }
    }

    (fills, strokes)
}

/// Parse mesh patches from content
fn parse_mesh_patches(content: &str) -> Vec<MeshPatch> {
    let mut patches = Vec::new();

    let mesh_re = match Regex::new(r"%AI9_Mesh_Mixed%") {
        Ok(re) => re,
        Err(_) => return patches,
    };
    if !mesh_re.is_match(content) {
        return patches;
    }

    let value_re = match Regex::new(r"-?\d+\.?\d*") {
        Ok(re) => re,
        Err(_) => return patches,
    };
    let values: Vec<f64> = value_re
        .find_iter(content)
        .filter_map(|m| m.as_str().parse().ok())
        .collect();

    if values.len() >= 8 {
        for chunk in values.chunks(8) {
            if chunk.len() == 8 {
                patches.push(MeshPatch {
                    corners: vec![
                        vec![chunk[0], chunk[1]],
                        vec![chunk[2], chunk[3]],
                        vec![chunk[4], chunk[5]],
                        vec![chunk[6], chunk[7]],
                    ],
                    colors: vec![
                        vec![255, 255, 255, 255],
                        vec![255, 255, 255, 255],
                        vec![255, 255, 255, 255],
                        vec![255, 255, 255, 255],
                    ],
                });
            }
        }
    }

    patches
}

/// Extract layer name from content markers
fn extract_layer_name(content: &str) -> Option<String> {
    let layer_re = Regex::new(r"%%Layer:\s*([^\n]+)").ok()?;
    if let Some(caps) = layer_re.captures(content) {
        return Some(caps.get(1)?.as_str().trim().to_string());
    }

    let begin_layer_re = Regex::new(r"%AI8_BeginLayer\s*\n?\s*%%Title:\s*([^\n]+)").ok()?;
    if let Some(caps) = begin_layer_re.captures(content) {
        return Some(caps.get(1)?.as_str().trim().to_string());
    }

    None
}

/// Extract AI version from content
fn extract_ai_version(content: &str) -> String {
    let creator_re = match Regex::new(r"%AI\d+_CreatorVersion\s*\n?\s*([\d.]+)") {
        Ok(re) => re,
        Err(_) => return String::new(),
    };
    if let Some(caps) = creator_re.captures(content) {
        return caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
    }

    let illustrator_re = match Regex::new(r"%%Creator:\s*Adobe\s+Illustrator\s+([\d.]+)") {
        Ok(re) => re,
        Err(_) => return String::new(),
    };
    if let Some(caps) = illustrator_re.captures(content) {
        return caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
    }

    String::new()
}

/// Parse the current transformation matrix (cm operator) from a PDF content stream.
/// Returns (rotation_deg, scale_x, scale_y, translate_x, translate_y) or None.
fn parse_ctm_from_stream(content: &str) -> Option<(f64, f64, f64, f64, f64)> {
    let re = Regex::new(
        r"(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s+(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s+(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s+(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s+(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s+(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s+cm\b"
    ).ok()?;
    // Use the last cm operator (innermost transform)
    let caps = re.captures_iter(content).last()?;
    let a: f64 = caps.get(1)?.as_str().parse().ok()?;
    let b: f64 = caps.get(2)?.as_str().parse().ok()?;
    let c: f64 = caps.get(3)?.as_str().parse().ok()?;
    let d: f64 = caps.get(4)?.as_str().parse().ok()?;
    let e: f64 = caps.get(5)?.as_str().parse().ok()?;
    let f: f64 = caps.get(6)?.as_str().parse().ok()?;
    let rotation_deg = b.atan2(a).to_degrees();
    let scale_x = (a * a + b * b).sqrt();
    let scale_y = (c * c + d * d).sqrt();
    Some((rotation_deg, scale_x, scale_y, e, f))
}

/// Parse PostScript path geometry from a content stream.
/// Returns (path_points, is_closed).
fn parse_path_geometry(content: &str) -> (Vec<PathPoint>, bool) {
    let mut points: Vec<PathPoint> = Vec::new();
    let mut closed = false;

    // Match PostScript path operators: m (moveto), l (lineto), c (curveto), h/z (closepath)
    // Word boundary \b ensures single-letter operators are not matched inside identifiers.
    let token_re = match Regex::new(r"(-?\d+\.?\d*(?:[eE][+-]?\d+)?)|\b([mlcCLMhHzZfFbBsS])\b") {
        Ok(re) => re,
        Err(_) => return (points, closed),
    };

    let mut tokens: Vec<String> = token_re
        .find_iter(content)
        .map(|m| m.as_str().to_string())
        .collect();
    tokens.reverse();
    let mut stack: Vec<f64> = Vec::new();

    while let Some(tok) = tokens.pop() {
        if let Ok(n) = tok.parse::<f64>() {
            stack.push(n);
        } else {
            match tok.as_str() {
                "m" | "M" => {
                    if stack.len() >= 2 {
                        let y = stack.pop().unwrap();
                        let x = stack.pop().unwrap();
                        points.push(PathPoint {
                            anchor: [x, y],
                            left_ctrl: [x, y],
                            right_ctrl: [x, y],
                        });
                    }
                    stack.clear();
                }
                "l" | "L" => {
                    if stack.len() >= 2 {
                        let y = stack.pop().unwrap();
                        let x = stack.pop().unwrap();
                        points.push(PathPoint {
                            anchor: [x, y],
                            left_ctrl: [x, y],
                            right_ctrl: [x, y],
                        });
                    }
                    stack.clear();
                }
                "c" | "C" => {
                    // curveto: x1 y1 x2 y2 x3 y3 c
                    if stack.len() >= 6 {
                        let y3 = stack.pop().unwrap();
                        let x3 = stack.pop().unwrap();
                        let y2 = stack.pop().unwrap();
                        let x2 = stack.pop().unwrap();
                        let y1 = stack.pop().unwrap();
                        let x1 = stack.pop().unwrap();
                        // Update the previous point's right control handle
                        if let Some(prev) = points.last_mut() {
                            prev.right_ctrl = [x1, y1];
                        }
                        points.push(PathPoint {
                            anchor: [x3, y3],
                            left_ctrl: [x2, y2],
                            right_ctrl: [x3, y3],
                        });
                    }
                    stack.clear();
                }
                "h" | "H" | "z" | "Z" => {
                    closed = true;
                    stack.clear();
                }
                _ => {
                    stack.clear();
                }
            }
        }
    }
    (points, closed)
}

/// Detect corner radius from an 8-point rounded rectangle Bezier path.
/// Returns the radius in document units, or 0.0 if not a rounded rect.
fn detect_corner_radius(points: &[PathPoint]) -> f64 {
    // A rounded rect has exactly 8 anchor points
    if points.len() != 8 {
        return 0.0;
    }
    // The cubic Bezier approximation constant for a quarter circle
    const KAPPA: f64 = 0.5522847498;
    let mut radii = Vec::new();
    for pt in points {
        let dx_left = pt.anchor[0] - pt.left_ctrl[0];
        let dy_left = pt.anchor[1] - pt.left_ctrl[1];
        let dx_right = pt.right_ctrl[0] - pt.anchor[0];
        let dy_right = pt.right_ctrl[1] - pt.anchor[1];
        let handle_left = (dx_left * dx_left + dy_left * dy_left).sqrt();
        let handle_right = (dx_right * dx_right + dy_right * dy_right).sqrt();
        let handle = handle_left.max(handle_right);
        if handle > 0.001 {
            radii.push(handle / KAPPA);
        }
    }
    if radii.is_empty() {
        return 0.0;
    }
    let mean = radii.iter().sum::<f64>() / radii.len() as f64;
    // Check consistency: all radii within 5% of mean
    let consistent = radii.iter().all(|&r| (r - mean).abs() / mean.max(0.001) < 0.05);
    if consistent { mean } else { 0.0 }
}

/// Parse AIPrivateData stream content
fn parse_aip_private_stream(
    content: &[u8],
    stream_idx: usize,
    errors: &mut Vec<String>,
) -> Option<Element> {
    let content_str = std::str::from_utf8(content).ok()?;

    let mut element = Element {
        id: format!("element_{}", stream_idx),
        ..Default::default()
    };

    if let Some(name) = extract_layer_name(content_str) {
        element.artboard_name = Some(name.clone());
        element.id = name;
    }

    let effects = parse_live_effect_xml(content_str);
    if !effects.is_empty() {
        element.live_effects = effects;
    }

    let (fills, strokes) = parse_appearance(content_str);
    if !fills.is_empty() {
        element.appearance_fills = fills;
    }
    if !strokes.is_empty() {
        element.appearance_strokes = strokes;
    }

    if let Some(mesh) = parse_envelope_mesh(content_str) {
        element.envelope_mesh = Some(mesh);
    }

    if let Some(threed) = parse_3d_effect(content_str) {
        element.three_d = Some(threed);
    }

    let patches = parse_mesh_patches(content_str);
    if !patches.is_empty() {
        element.mesh_patches = patches;
    }

    // Parse CTM rotation
    if let Some((rot, sx, sy, tx, ty)) = parse_ctm_from_stream(content_str) {
        if rot.abs() > 0.01 {
            element.rotation_deg = rot;
        }
        element.scale_x = sx;
        element.scale_y = sy;
        element.translate_x = tx;
        element.translate_y = ty;
    }

    // Parse path geometry and detect corner radius
    let (path_pts, path_closed) = parse_path_geometry(content_str);
    if !path_pts.is_empty() {
        element.corner_radius = detect_corner_radius(&path_pts);
        element.path_points = path_pts;
        element.path_closed = path_closed;
    }

    if element.live_effects.is_empty()
        && element.appearance_fills.is_empty()
        && element.appearance_strokes.is_empty()
        && element.envelope_mesh.is_none()
        && element.three_d.is_none()
        && element.mesh_patches.is_empty()
        && element.rotation_deg.abs() < 0.01
        && element.path_points.is_empty()
    {
        errors.push(format!("warning: could not parse stream {}", stream_idx));
        return None;
    }

    Some(element)
}

/// Main parsing function
pub fn parse_ai_file(path: &Path) -> Result<AiParseResult, String> {
    let source_file = path.to_string_lossy().to_string();

    let doc = Document::load(path).map_err(|e| format!("not a valid .ai file: {}", e))?;

    let mut result = AiParseResult {
        version: "1.0".to_string(),
        source_file,
        ai_version: String::new(),
        artboards: Vec::new(),
        elements: Vec::new(),
        errors: Vec::new(),
    };

    for (object_id, object) in doc.objects.iter() {
        let content = if let Ok(stream) = object.as_stream() {
            match stream.decompressed_content() {
                Ok(decompressed) => decompressed,
                Err(_) => stream.content.clone(),
            }
        } else {
            continue;
        };

        let content_str = std::str::from_utf8(&content).unwrap_or("");

        // Always scan every decompressed stream for CTM (cm) operators and path geometry.
        // Illustrator stores per-object transforms in main PDF content streams, not just AIPrivateData.
        if content_str.contains("AIPrivateData")
            || content_str.contains("LiveEffect")
            || content_str.contains("%AI9_")
        {
            if let Some(element) =
                parse_aip_private_stream(&content, object_id.0 as usize, &mut result.errors)
            {
                let is_dup = result.elements.iter().any(|e| e.id == element.id);
                if !is_dup {
                    result.elements.push(element);
                }
            }
        } else if content_str.contains(" cm") || content_str.contains("\ncm") {
            // Main PDF content stream: scan for CTM and path geometry only
            let mut element = Element {
                id: format!("ctm_element_{}", object_id.0),
                ..Default::default()
            };
            if let Some((rot, sx, sy, tx, ty)) = parse_ctm_from_stream(content_str) {
                element.rotation_deg = rot;
                element.scale_x = sx;
                element.scale_y = sy;
                element.translate_x = tx;
                element.translate_y = ty;
            }
            let (path_pts, path_closed) = parse_path_geometry(content_str);
            if !path_pts.is_empty() {
                element.corner_radius = detect_corner_radius(&path_pts);
                element.path_points = path_pts;
                element.path_closed = path_closed;
            }
            // Only add if we found meaningful data
            if element.rotation_deg.abs() > 0.01 || !element.path_points.is_empty() {
                let is_dup = result.elements.iter().any(|e| e.id == element.id);
                if !is_dup {
                    result.elements.push(element);
                }
            }
        }

        // Parse artboard definitions from %AI9_Artboard markers
        if content_str.contains("%AI9_Artboard") || content_str.contains("%%BeginSetup") {
            {
                let re = artboard_re();
                let mut names = artboard_name_re()
                    .captures_iter(content_str)
                    .map(|c| c.get(1).map_or("", |m| m.as_str().trim()).to_string());
                for caps in re.captures_iter(content_str) {
                    if let (Some(x1), Some(y1), Some(x2), Some(y2)) =
                        (caps.get(1), caps.get(2), caps.get(3), caps.get(4))
                    {
                        let x1: f64 = x1.as_str().parse().unwrap_or(0.0);
                        let y1: f64 = y1.as_str().parse().unwrap_or(0.0);
                        let x2: f64 = x2.as_str().parse().unwrap_or(0.0);
                        let y2: f64 = y2.as_str().parse().unwrap_or(0.0);
                        let name = names.next()
                            .filter(|n| !n.is_empty())
                            .unwrap_or_else(|| format!("Artboard_{}", result.artboards.len() + 1));
                        if !result.artboards.iter().any(|a: &Artboard| a.name == name) {
                            result.artboards.push(Artboard {
                                name,
                                x: x1,
                                y: y1,
                                width: (x2 - x1).abs(),
                                height: (y2 - y1).abs(),
                            });
                        }
                    }
                }
            }
        }

        // Fallback: %%AI_ArtboardRect x1 y1 x2 y2
        if result.artboards.is_empty() {
            static AI_ARTBOARD_RECT_RE: OnceLock<Regex> = OnceLock::new();
            let re = AI_ARTBOARD_RECT_RE.get_or_init(|| {
                Regex::new(r"%%AI_ArtboardRect\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)").unwrap()
            });
            for caps in re.captures_iter(content_str) {
                if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (caps.get(1), caps.get(2), caps.get(3), caps.get(4)) {
                    let x1: f64 = x1.as_str().parse().unwrap_or(0.0);
                    let y1: f64 = y1.as_str().parse().unwrap_or(0.0);
                    let x2: f64 = x2.as_str().parse().unwrap_or(0.0);
                    let y2: f64 = y2.as_str().parse().unwrap_or(0.0);
                    let name = format!("Artboard_{}", result.artboards.len() + 1);
                    if !result.artboards.iter().any(|a: &Artboard| a.name == name) {
                        result.artboards.push(Artboard { name, x: x1, y: y1, width: (x2 - x1).abs(), height: (y2 - y1).abs() });
                    }
                }
            }
        }

        if result.ai_version.is_empty() {
            let version = extract_ai_version(content_str);
            if !version.is_empty() {
                result.ai_version = version;
            }
        }
    }

    // Fallback: %%HiResBoundingBox or %%BoundingBox
    if result.artboards.is_empty() {
        if let Ok(bytes) = std::fs::read(path) {
            let full_content = String::from_utf8_lossy(&bytes);
            static BBOX_RE: OnceLock<Regex> = OnceLock::new();
            let re = BBOX_RE.get_or_init(|| {
                Regex::new(r"%%(?:HiRes)?BoundingBox:\s*(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)").unwrap()
            });
            if let Some(caps) = re.captures(&full_content) {
                if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (caps.get(1), caps.get(2), caps.get(3), caps.get(4)) {
                    let x1: f64 = x1.as_str().parse().unwrap_or(0.0);
                    let y1: f64 = y1.as_str().parse().unwrap_or(0.0);
                    let x2: f64 = x2.as_str().parse().unwrap_or(0.0);
                    let y2: f64 = y2.as_str().parse().unwrap_or(0.0);
                    result.artboards.push(Artboard { name: "Artboard_1".to_string(), x: x1, y: y1, width: (x2 - x1).abs(), height: (y2 - y1).abs() });
                }
            }
        }
    }

    if result.ai_version.is_empty() {
        // Try to get version from document info dictionary
        if let Ok(info_ref) = doc.trailer.get(b"Info") {
            if let Ok(info_id) = info_ref.as_reference() {
                if let Ok(info) = doc.get_object(info_id) {
                    if let Ok(dict) = info.as_dict() {
                        if let Ok(creator_obj) = dict.get(b"Creator") {
                            if let Ok(s) = creator_obj.as_string() {
                                let creator_str = String::from_utf8_lossy(s.as_bytes()).to_string();
                                if let Ok(re) = Regex::new(r"Adobe Illustrator[^\d]*([\d.]+)") {
                                    if let Some(caps) = re.captures(&creator_str) {
                                        result.ai_version = caps
                                            .get(1)
                                            .map(|m| m.as_str().to_string())
                                            .unwrap_or_default();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(result)
}

/// Convert an ai_parser `Element` to a codegen `LayoutElement` for code generation.
fn element_to_layout(elem: &Element, idx: usize) -> LayoutElement {
    let id = if elem.id.is_empty() {
        format!("elem_{}", idx)
    } else {
        elem.id.clone()
    };
    // Use fill color from appearance_fills if available
    let fill_color = elem.appearance_fills.first().map(|c| {
        egui::Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
    }).unwrap_or(egui::Color32::TRANSPARENT);
    let stroke_color = elem.appearance_strokes.first().map(|s| {
        egui::Color32::from_rgba_unmultiplied(s.r, s.g, s.b, s.a)
    }).unwrap_or(egui::Color32::TRANSPARENT);
    let stroke_width = elem.appearance_strokes.first().map(|s| s.width as f32).unwrap_or(0.0);

    // Derive position and size from path_points bounding box when available,
    // otherwise fall back to CTM translate_x/translate_y with a default size.
    let (x, y, w, h) = if !elem.path_points.is_empty() {
        let min_x = elem.path_points.iter().map(|p| p.anchor[0]).fold(f64::INFINITY, f64::min);
        let min_y = elem.path_points.iter().map(|p| p.anchor[1]).fold(f64::INFINITY, f64::min);
        let max_x = elem.path_points.iter().map(|p| p.anchor[0]).fold(f64::NEG_INFINITY, f64::max);
        let max_y = elem.path_points.iter().map(|p| p.anchor[1]).fold(f64::NEG_INFINITY, f64::max);
        let w = (max_x - min_x).max(1.0);
        let h = (max_y - min_y).max(1.0);
        (min_x as f32, min_y as f32, w as f32, h as f32)
    } else {
        // Use CTM translation as position; scale gives approximate size
        let w = (elem.scale_x * 100.0).max(1.0) as f32;
        let h = (elem.scale_y * 100.0).max(1.0) as f32;
        (elem.translate_x as f32, elem.translate_y as f32, w, h)
    };

    let mut layout_elem = LayoutElement::new(id, ElementType::Shape, x, y, w, h);
    layout_elem.fill = Some(fill_color);
    layout_elem.stroke = Some((stroke_width, stroke_color));
    layout_elem.rotation_deg = elem.rotation_deg as f32;
    layout_elem.corner_radius = elem.corner_radius as f32;
    layout_elem.opacity = 1.0;
    layout_elem
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: ai-parser <file.ai> [--pretty]");
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);
    let pretty = args.iter().any(|a| a == "--pretty");
    let per_artboard = args.iter().any(|a| a == "--per-artboard");

    let result = match parse_ai_file(path) {
        Ok(r) => r,
        Err(e) => {
            let error_result = AiParseResult {
                version: "1.0".to_string(),
                source_file: args[1].clone(),
                ai_version: String::new(),
                artboards: Vec::new(),
                elements: Vec::new(),
                errors: vec![e],
            };
            if let Ok(json) = serde_json::to_string(&error_result) {
                println!("{}", json);
            } else {
                println!("{{\"error\": \"not a valid .ai file\"}}");
            }
            return;
        }
    };

    if per_artboard {
        // Output one entry per artboard (or one entry if no artboards defined)
        let artboards = if result.artboards.is_empty() {
            vec![("default".to_string(), 0.0f64, 0.0f64, f64::MAX, f64::MAX)]
        } else {
            result.artboards.iter().map(|a| {
                (a.name.clone(), a.x, a.y, a.x + a.width, a.y + a.height)
            }).collect::<Vec<_>>()
        };

        let mut entries: Vec<serde_json::Value> = Vec::new();
        for (artboard_idx, (name, _x1, _y1, _x2, _y2)) in artboards.iter().enumerate() {
            let sanitized = name
                .chars()
                .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
                .collect::<String>();
            let sanitized = if sanitized.starts_with(|c: char| c.is_ascii_digit()) {
                format!("ab_{}", sanitized)
            } else if sanitized.is_empty() {
                "artboard".to_string()
            } else {
                sanitized
            };
            let filename = format!("{}.rs", sanitized);
            let element_count = result.elements.iter()
                .filter(|e| e.artboard_name.as_deref() == Some(name.as_str())
                    || (e.artboard_name.is_none() && artboard_idx == 0))
                .count();
            let artboard_info = artboards.iter().find(|(n, _, _, _, _)| n == name);
            let (ab_w, ab_h) = artboard_info
                .map(|(_, x1, y1, x2, y2)| ((x2 - x1).abs(), (y2 - y1).abs()))
                .unwrap_or((375.0, 812.0));
            let layout_elements: Vec<LayoutElement> = result.elements.iter()
                .filter(|e| e.artboard_name.as_deref() == Some(name.as_str())
                    || (e.artboard_name.is_none() && artboard_idx == 0))
                .enumerate()
                .map(|(i, e)| element_to_layout(e, i))
                .collect();
            let code = generate_artboard_file(name, ab_w as f32, ab_h as f32, &layout_elements, &std::collections::HashMap::new());
            entries.push(serde_json::json!({
                "artboard": name,
                "filename": filename,
                "width": ab_w,
                "height": ab_h,
                "element_count": element_count,
                "code": code,
                "elements": result.elements.iter()
                    .filter(|e| e.artboard_name.as_deref() == Some(name.as_str())
                        || (e.artboard_name.is_none() && artboard_idx == 0))
                    .collect::<Vec<_>>(),
            }));
        }
        let json = if pretty {
            serde_json::to_string_pretty(&entries)
        } else {
            serde_json::to_string(&entries)
        };
        match json {
            Ok(j) => println!("{}", j),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        let json = if pretty {
            serde_json::to_string_pretty(&result)
        } else {
            serde_json::to_string(&result)
        };
        match json {
            Ok(j) => println!("{}", j),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_live_effect_xml() {
        let xml = r#"<LiveEffect name="Adobe Drop Shadow"><Dict data="R horz 7.0 R vert 7.0 I blnd 1 B enbl 1"/></LiveEffect>"#;
        let effects = parse_live_effect_xml(xml);
        assert!(!effects.is_empty());
        assert_eq!(effects[0].name, "Adobe Drop Shadow");
        assert_eq!(
            effects[0].params.params.get("horz"),
            Some(&Value::from(7.0))
        );
    }

    #[test]
    fn test_parse_dict_data() {
        let data = "R horz 7.0 R vert 7.0 R blur 5.0 I opac 75 B enbl 1";
        let params = parse_dict_data(data);
        assert_eq!(params.get("horz"), Some(&Value::from(7.0)));
        assert_eq!(params.get("vert"), Some(&Value::from(7.0)));
        assert_eq!(params.get("blur"), Some(&Value::from(5.0)));
        assert_eq!(params.get("opac"), Some(&Value::from(75_i64)));
        assert_eq!(params.get("enbl"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_parse_envelope_mesh() {
        let content = "%AI9_EnvelopeMesh 3 3\n[0 0] [50 0] [100 0]\n[0 50] [50 50] [100 50]\n[0 100] [50 100] [100 100]";
        let mesh = parse_envelope_mesh(content);
        assert!(mesh.is_some());
        let mesh = mesh.unwrap();
        assert_eq!(mesh.rows, 3);
        assert_eq!(mesh.cols, 3);
        assert!(mesh.points.len() >= 9);
    }

    #[test]
    fn test_parse_3d_extrude() {
        let content = "%AI9_3D_Extrude 100 45 30 0";
        let effect = parse_3d_effect(content);
        assert!(effect.is_some());
        let effect = effect.unwrap();
        assert_eq!(effect.effect_type, "extrude");
        assert_eq!(effect.depth, 100.0);
    }

    #[test]
    fn test_extract_layer_name() {
        let content = "%%Layer: MyLayer\n%AI8_BeginLayer";
        let name = extract_layer_name(content);
        assert_eq!(name, Some("MyLayer".to_string()));
    }

    #[test]
    fn test_extract_ai_version() {
        let content = "%AI8_CreatorVersion 25.0";
        let version = extract_ai_version(content);
        assert_eq!(version, "25.0");
    }

    #[test]
    fn test_parse_ctm_identity() {
        let content = "1 0 0 1 0 0 cm";
        let result = parse_ctm_from_stream(content);
        assert!(result.is_some());
        let (rot, sx, sy, tx, ty) = result.unwrap();
        assert!((rot).abs() < 0.001, "identity rotation should be 0, got {}", rot);
        assert!((sx - 1.0).abs() < 0.001);
        assert!((sy - 1.0).abs() < 0.001);
        assert!((tx).abs() < 0.001);
        assert!((ty).abs() < 0.001);
    }

    #[test]
    fn test_parse_ctm_90deg() {
        // 90 degree rotation: a=0, b=1, c=-1, d=0
        let content = "0 1 -1 0 0 0 cm";
        let result = parse_ctm_from_stream(content);
        assert!(result.is_some());
        let (rot, _sx, _sy, _tx, _ty) = result.unwrap();
        assert!((rot - 90.0).abs() < 0.01, "expected 90 deg, got {}", rot);
    }

    #[test]
    fn test_detect_corner_radius_zero() {
        // A simple square has no control handles → radius 0
        let points = vec![
            PathPoint { anchor: [0.0, 0.0], left_ctrl: [0.0, 0.0], right_ctrl: [0.0, 0.0] },
            PathPoint { anchor: [100.0, 0.0], left_ctrl: [100.0, 0.0], right_ctrl: [100.0, 0.0] },
            PathPoint { anchor: [100.0, 100.0], left_ctrl: [100.0, 100.0], right_ctrl: [100.0, 100.0] },
            PathPoint { anchor: [0.0, 100.0], left_ctrl: [0.0, 100.0], right_ctrl: [0.0, 100.0] },
        ];
        assert_eq!(detect_corner_radius(&points), 0.0);
    }

    #[test]
    fn test_detect_corner_radius_rounded() {
        // 8-point rounded rect with radius=50: handle distance = 50 * 0.5522847498 ≈ 27.614
        const KAPPA: f64 = 0.5522847498;
        let r = 50.0f64;
        let h = r * KAPPA;
        // Top edge: TL-right, TR-left
        let points = vec![
            PathPoint { anchor: [r, 0.0],       left_ctrl: [r - h, 0.0],   right_ctrl: [r + h, 0.0] },   // top-left corner right
            PathPoint { anchor: [100.0 - r, 0.0], left_ctrl: [100.0 - r - h, 0.0], right_ctrl: [100.0 - r + h, 0.0] }, // top-right corner left
            PathPoint { anchor: [100.0, r],      left_ctrl: [100.0, r - h], right_ctrl: [100.0, r + h] }, // right-top corner
            PathPoint { anchor: [100.0, 100.0 - r], left_ctrl: [100.0, 100.0 - r - h], right_ctrl: [100.0, 100.0 - r + h] },
            PathPoint { anchor: [100.0 - r, 100.0], left_ctrl: [100.0 - r + h, 100.0], right_ctrl: [100.0 - r - h, 100.0] },
            PathPoint { anchor: [r, 100.0],      left_ctrl: [r + h, 100.0], right_ctrl: [r - h, 100.0] },
            PathPoint { anchor: [0.0, 100.0 - r], left_ctrl: [0.0, 100.0 - r + h], right_ctrl: [0.0, 100.0 - r - h] },
            PathPoint { anchor: [0.0, r],        left_ctrl: [0.0, r + h],   right_ctrl: [0.0, r - h] },
        ];
        let detected = detect_corner_radius(&points);
        assert!((detected - r).abs() < 2.0, "expected radius ~{}, got {}", r, detected);
    }
}
