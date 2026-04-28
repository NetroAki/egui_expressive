//! Adobe Illustrator (.ai) file parser
//!
//! Parses .ai files (PDF wrappers) and extracts visual properties from AIPrivateData streams.

use lopdf::Document;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

static ARTBOARD_RE: OnceLock<Regex> = OnceLock::new();
static ARTBOARD_NAME_RE: OnceLock<Regex> = OnceLock::new();

fn artboard_re() -> &'static Regex {
    ARTBOARD_RE.get_or_init(|| {
        Regex::new(r"%AI9_Artboard\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)")
            .expect("valid artboard regex")
    })
}

fn artboard_name_re() -> &'static Regex {
    ARTBOARD_NAME_RE.get_or_init(|| {
        Regex::new(r"%AI9_ArtboardName\s+([^\n]+)").expect("valid artboard name regex")
    })
}

use egui_expressive::codegen::{
    generate_artboard_file, AppearanceFill, AppearanceStroke, BlendMode, EffectDef, EffectType,
    ElementType, GradientDef, GradientStop, GradientType, LayoutElement,
};

/// RGBA Color representation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
    #[serde(default)]
    pub opacity: Option<f64>,
    #[serde(default)]
    pub blend_mode: String,
}

/// Stroke representation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Stroke {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
    #[serde(default)]
    pub width: f64,
    #[serde(default)]
    pub opacity: Option<f64>,
    #[serde(default)]
    pub blend_mode: String,
    #[serde(default)]
    pub cap: Option<String>,
    #[serde(default)]
    pub join: Option<String>,
    #[serde(default)]
    pub dash: Option<Vec<f32>>,
    #[serde(default)]
    pub miter_limit: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gradient: Option<Value>,
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

/// Page tile representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageTile {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
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
    #[serde(alias = "leftDir")]
    pub left_ctrl: [f64; 2],
    #[serde(alias = "rightDir")]
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
    pub transform_candidates: Vec<[f64; 5]>,
    #[serde(default)]
    pub corner_radius: f64,
    #[serde(default)]
    pub path_points: Vec<PathPoint>,
    #[serde(default)]
    pub path_closed: bool,
    #[serde(default)]
    pub artboard_name: Option<String>,
    #[serde(default)]
    pub is_pseudo_element: bool,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub element_type: Option<String>,
    #[serde(default)]
    pub bounds: Option<[f64; 4]>,
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
    pub page_tiles: Vec<PageTile>,
    #[serde(default)]
    pub elements: Vec<Element>,
    #[serde(default)]
    pub transform_candidates: Vec<Element>,
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

    let mut blend_mode = "normal".to_string();
    let bm_re = Regex::new(r"/(?:BM|BlendMode)\s+/([A-Za-z]+)").ok();
    if let Some(re) = bm_re {
        if let Some(caps) = re.captures(content) {
            if let Some(mode) = caps.get(1) {
                blend_mode = mode.as_str().to_string();
            }
        }
    }

    let mut cap = None;
    let cap_re = Regex::new(r"\b([012])\s+J\b").ok();
    if let Some(re) = cap_re {
        if let Some(caps) = re.captures_iter(content).last() {
            cap = match caps.get(1).unwrap().as_str() {
                "0" => Some("butt".to_string()),
                "1" => Some("round".to_string()),
                "2" => Some("square".to_string()),
                _ => None,
            };
        }
    }

    let mut join = None;
    let join_re = Regex::new(r"\b([012])\s+j\b").ok();
    if let Some(re) = join_re {
        if let Some(caps) = re.captures_iter(content).last() {
            join = match caps.get(1).unwrap().as_str() {
                "0" => Some("miter".to_string()),
                "1" => Some("round".to_string()),
                "2" => Some("bevel".to_string()),
                _ => None,
            };
        }
    }

    let mut miter_limit = None;
    let miter_re = Regex::new(r"\b(\d+(?:\.\d+)?)\s+M\b").ok();
    if let Some(re) = miter_re {
        if let Some(caps) = re.captures_iter(content).last() {
            miter_limit = caps.get(1).unwrap().as_str().parse::<f32>().ok();
        }
    }

    let mut dash = None;
    let dash_re = Regex::new(r"\[(.*?)\]\s+\d+(?:\.\d+)?\s+d\b").ok();
    if let Some(re) = dash_re {
        if let Some(caps) = re.captures_iter(content).last() {
            let dash_str = caps.get(1).unwrap().as_str();
            let dash_arr: Vec<f32> = dash_str
                .split_whitespace()
                .filter_map(|s| s.parse::<f32>().ok())
                .collect();
            if !dash_arr.is_empty() {
                dash = Some(dash_arr);
            }
        }
    }

    let mut width = 1.0;
    let width_re = Regex::new(r"\b(\d+(?:\.\d+)?)\s+w\b").ok();
    if let Some(re) = width_re {
        if let Some(caps) = re.captures_iter(content).last() {
            width = caps.get(1).unwrap().as_str().parse::<f64>().unwrap_or(1.0);
        }
    }

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
                let op_str = op.as_str();

                if op_str.starts_with('X') {
                    fills.push(Color {
                        r: r_val,
                        g: g_val,
                        b: b_val,
                        a: a_val,
                        opacity: Some(1.0),
                        blend_mode: blend_mode.clone(),
                    });
                } else if op_str.starts_with('x') {
                    strokes.push(Stroke {
                        r: r_val,
                        g: g_val,
                        b: b_val,
                        a: a_val,
                        width,
                        opacity: Some(1.0),
                        blend_mode: blend_mode.clone(),
                        cap: cap.clone(),
                        join: join.clone(),
                        dash: dash.clone(),
                        miter_limit,
                        gradient: None,
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
                        opacity: Some(1.0),
                        blend_mode: blend_mode.clone(),
                    });
                }
            }
        }
    }

    // Fallback for gradients/patterns
    if fills.is_empty()
        && (content.contains(" sh\n")
            || content.contains(" sh\r")
            || content.contains(" sh ")
            || content.contains("/Pattern cs")
            || content.contains("/Pattern CS"))
    {
        fills.push(Color {
            r: 128,
            g: 128,
            b: 128,
            a: 255,
            opacity: Some(1.0),
            blend_mode: blend_mode.clone(),
        });
    }

    if strokes.is_empty() && content.contains("/Pattern CS") {
        strokes.push(Stroke {
            r: 128,
            g: 128,
            b: 128,
            a: 255,
            width,
            opacity: Some(1.0),
            blend_mode: blend_mode.clone(),
            cap: cap.clone(),
            join: join.clone(),
            dash: dash.clone(),
            miter_limit,
            gradient: Some(json!({
                "type": "pattern",
                "patternName": "parser-stroke-pattern",
                "seed": 0,
                "cellSize": 8.0,
                "markSize": 2.0
            })),
        });
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

    let chunk_size = if values.len().is_multiple_of(20) {
        20
    } else {
        8
    };

    if values.len() >= chunk_size {
        for chunk in values.chunks(chunk_size) {
            if chunk.len() == chunk_size {
                let mut colors = vec![
                    vec![255, 255, 255, 255],
                    vec![255, 255, 255, 255],
                    vec![255, 255, 255, 255],
                    vec![255, 255, 255, 255],
                ];
                if chunk_size == 20 {
                    for i in 0..4 {
                        colors[i] = vec![
                            (chunk[8 + i * 3] * 255.0).clamp(0.0, 255.0) as u8,
                            (chunk[9 + i * 3] * 255.0).clamp(0.0, 255.0) as u8,
                            (chunk[10 + i * 3] * 255.0).clamp(0.0, 255.0) as u8,
                            255,
                        ];
                    }
                }
                patches.push(MeshPatch {
                    corners: vec![
                        vec![chunk[0], chunk[1]],
                        vec![chunk[2], chunk[3]],
                        vec![chunk[4], chunk[5]],
                        vec![chunk[6], chunk[7]],
                    ],
                    colors,
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

/// Parse all current transformation matrices (cm operator) from a PDF content stream.
/// Returns a list of (rotation_deg, scale_x, scale_y, translate_x, translate_y) candidates.
fn parse_ctms_from_stream(content: &str) -> Vec<(f64, f64, f64, f64, f64)> {
    let mut ctms = Vec::new();
    let re = match Regex::new(
        r"(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s+(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s+(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s+(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s+(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s+(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s+cm\b",
    ) {
        Ok(re) => re,
        Err(_) => return ctms,
    };
    for caps in re.captures_iter(content) {
        if let (Ok(a), Ok(b), Ok(c), Ok(d), Ok(e), Ok(f)) = (
            caps.get(1).unwrap().as_str().parse::<f64>(),
            caps.get(2).unwrap().as_str().parse::<f64>(),
            caps.get(3).unwrap().as_str().parse::<f64>(),
            caps.get(4).unwrap().as_str().parse::<f64>(),
            caps.get(5).unwrap().as_str().parse::<f64>(),
            caps.get(6).unwrap().as_str().parse::<f64>(),
        ) {
            let rotation_deg = b.atan2(a).to_degrees();
            let scale_x = (a * a + b * b).sqrt();
            let scale_y = (c * c + d * d).sqrt();
            ctms.push((rotation_deg, scale_x, scale_y, e, f));
        }
    }
    ctms
}

#[derive(Clone, Debug)]
struct PdfGraphicsState {
    ctm: [f64; 6],
    fill: Color,
    stroke: Stroke,
}

impl Default for PdfGraphicsState {
    fn default() -> Self {
        Self {
            ctm: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            fill: Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
                opacity: Some(1.0),
                blend_mode: "normal".to_string(),
            },
            stroke: Stroke {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
                width: 1.0,
                opacity: Some(1.0),
                blend_mode: "normal".to_string(),
                cap: None,
                join: None,
                dash: None,
                miter_limit: None,
                gradient: None,
            },
        }
    }
}

fn concat_ctm(current: [f64; 6], next: [f64; 6]) -> [f64; 6] {
    let [a, b, c, d, e, f] = current;
    let [g, h, i, j, k, l] = next;
    [
        a * g + c * h,
        b * g + d * h,
        a * i + c * j,
        b * i + d * j,
        a * k + c * l + e,
        b * k + d * l + f,
    ]
}

fn transform_pdf_point(ctm: [f64; 6], x: f64, y: f64) -> [f64; 2] {
    [
        ctm[0] * x + ctm[2] * y + ctm[4],
        ctm[1] * x + ctm[3] * y + ctm[5],
    ]
}

fn pdf_color_from_components(values: &[f64], blend_mode: &str) -> Color {
    let (r, g, b) = match values.len() {
        0 => (0.0, 0.0, 0.0),
        1 => {
            let gray = values[0].clamp(0.0, 1.0);
            (gray, gray, gray)
        }
        2 | 3 => (
            values[0].clamp(0.0, 1.0),
            values.get(1).copied().unwrap_or(0.0).clamp(0.0, 1.0),
            values.get(2).copied().unwrap_or(0.0).clamp(0.0, 1.0),
        ),
        _ => {
            let c = values[0].clamp(0.0, 1.0);
            let m = values[1].clamp(0.0, 1.0);
            let y = values[2].clamp(0.0, 1.0);
            let k = values[3].clamp(0.0, 1.0);
            (
                (1.0 - c) * (1.0 - k),
                (1.0 - m) * (1.0 - k),
                (1.0 - y) * (1.0 - k),
            )
        }
    };

    Color {
        r: (r * 255.0).round().clamp(0.0, 255.0) as u8,
        g: (g * 255.0).round().clamp(0.0, 255.0) as u8,
        b: (b * 255.0).round().clamp(0.0, 255.0) as u8,
        a: 255,
        opacity: Some(1.0),
        blend_mode: blend_mode.to_string(),
    }
}

fn path_bounds(points: &[PathPoint]) -> Option<[f64; 4]> {
    let first = points.first()?;
    let mut min_x = first.anchor[0];
    let mut min_y = first.anchor[1];
    let mut max_x = first.anchor[0];
    let mut max_y = first.anchor[1];
    for point in points {
        for [x, y] in [point.anchor, point.left_ctrl, point.right_ctrl] {
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
    }
    Some([
        min_x,
        min_y,
        (max_x - min_x).max(1.0),
        (max_y - min_y).max(1.0),
    ])
}

fn painted_path_element(
    stream_idx: usize,
    element_idx: usize,
    points: &[PathPoint],
    closed: bool,
    state: &PdfGraphicsState,
    fill: bool,
    stroke: bool,
) -> Option<Element> {
    if points.is_empty() || (!fill && !stroke) {
        return None;
    }
    let mut element = Element {
        id: format!("pdf_path_{}_{}", stream_idx, element_idx),
        element_type: Some("shape".to_string()),
        path_points: points.to_vec(),
        path_closed: closed,
        corner_radius: detect_corner_radius(points),
        is_pseudo_element: true,
        ..Default::default()
    };
    if fill {
        element.appearance_fills.push(state.fill.clone());
    }
    if stroke && state.stroke.width > 0.0 {
        element.appearance_strokes.push(state.stroke.clone());
    }
    element.bounds = path_bounds(points);
    Some(element)
}

/// Parse painted PDF path objects from a content stream.
///
/// This keeps the Illustrator/PDF reference path vector-only: it converts PDF path paint commands
/// into the same codegen/scene primitives used by hand-authored egui_expressive code rather than
/// embedding the rendered PDF/PNG as an image.
fn parse_pdf_painted_path_elements(content: &str, stream_idx: usize) -> Vec<Element> {
    let token_re = match Regex::new(
        r"/[A-Za-z0-9_.#-]+|-?\d*\.?\d+(?:[eE][+-]?\d+)?|f\*|B\*|b\*|[A-Za-z]{1,3}|\S",
    ) {
        Ok(re) => re,
        Err(_) => return Vec::new(),
    };
    let mut state = PdfGraphicsState::default();
    let mut stack: Vec<f64> = Vec::new();
    let mut saved_states: Vec<PdfGraphicsState> = Vec::new();
    let mut path: Vec<PathPoint> = Vec::new();
    let mut closed = false;
    let mut elements = Vec::new();

    for token in token_re.find_iter(content).map(|m| m.as_str()) {
        if let Ok(value) = token.parse::<f64>() {
            stack.push(value);
            continue;
        }
        if token.starts_with('/') {
            continue;
        }

        match token {
            "q" => saved_states.push(state.clone()),
            "Q" => {
                if let Some(saved) = saved_states.pop() {
                    state = saved;
                }
                path.clear();
                closed = false;
            }
            "cm" if stack.len() >= 6 => {
                let m = [
                    stack[stack.len() - 6],
                    stack[stack.len() - 5],
                    stack[stack.len() - 4],
                    stack[stack.len() - 3],
                    stack[stack.len() - 2],
                    stack[stack.len() - 1],
                ];
                state.ctm = concat_ctm(state.ctm, m);
            }
            "w" if !stack.is_empty() => {
                state.stroke.width = stack[stack.len() - 1].max(0.0);
            }
            "J" if !stack.is_empty() => {
                state.stroke.cap = match stack[stack.len() - 1].round() as i32 {
                    0 => Some("butt".to_string()),
                    1 => Some("round".to_string()),
                    2 => Some("square".to_string()),
                    _ => state.stroke.cap.clone(),
                };
            }
            "j" if !stack.is_empty() => {
                state.stroke.join = match stack[stack.len() - 1].round() as i32 {
                    0 => Some("miter".to_string()),
                    1 => Some("round".to_string()),
                    2 => Some("bevel".to_string()),
                    _ => state.stroke.join.clone(),
                };
            }
            "M" if !stack.is_empty() => {
                state.stroke.miter_limit = Some(stack[stack.len() - 1] as f32);
            }
            "rg" if stack.len() >= 3 => {
                state.fill =
                    pdf_color_from_components(&stack[stack.len() - 3..], &state.fill.blend_mode);
            }
            "RG" if stack.len() >= 3 => {
                let mut stroke = state.stroke.clone();
                let color =
                    pdf_color_from_components(&stack[stack.len() - 3..], &stroke.blend_mode);
                stroke.r = color.r;
                stroke.g = color.g;
                stroke.b = color.b;
                stroke.a = color.a;
                state.stroke = stroke;
            }
            "g" if !stack.is_empty() => {
                state.fill =
                    pdf_color_from_components(&stack[stack.len() - 1..], &state.fill.blend_mode);
            }
            "G" if !stack.is_empty() => {
                let mut stroke = state.stroke.clone();
                let color =
                    pdf_color_from_components(&stack[stack.len() - 1..], &stroke.blend_mode);
                stroke.r = color.r;
                stroke.g = color.g;
                stroke.b = color.b;
                stroke.a = color.a;
                state.stroke = stroke;
            }
            "k" | "sc" | "scn" if !stack.is_empty() => {
                state.fill = pdf_color_from_components(&stack, &state.fill.blend_mode);
            }
            "K" | "SC" | "SCN" if !stack.is_empty() => {
                let mut stroke = state.stroke.clone();
                let color = pdf_color_from_components(&stack, &stroke.blend_mode);
                stroke.r = color.r;
                stroke.g = color.g;
                stroke.b = color.b;
                stroke.a = color.a;
                state.stroke = stroke;
            }
            "m" if stack.len() >= 2 => {
                let [x, y] =
                    transform_pdf_point(state.ctm, stack[stack.len() - 2], stack[stack.len() - 1]);
                path.clear();
                closed = false;
                path.push(PathPoint {
                    anchor: [x, y],
                    left_ctrl: [x, y],
                    right_ctrl: [x, y],
                });
            }
            "l" if stack.len() >= 2 => {
                let [x, y] =
                    transform_pdf_point(state.ctm, stack[stack.len() - 2], stack[stack.len() - 1]);
                path.push(PathPoint {
                    anchor: [x, y],
                    left_ctrl: [x, y],
                    right_ctrl: [x, y],
                });
            }
            "c" if stack.len() >= 6 => {
                let [x1, y1] =
                    transform_pdf_point(state.ctm, stack[stack.len() - 6], stack[stack.len() - 5]);
                let [x2, y2] =
                    transform_pdf_point(state.ctm, stack[stack.len() - 4], stack[stack.len() - 3]);
                let [x3, y3] =
                    transform_pdf_point(state.ctm, stack[stack.len() - 2], stack[stack.len() - 1]);
                if let Some(prev) = path.last_mut() {
                    prev.right_ctrl = [x1, y1];
                }
                path.push(PathPoint {
                    anchor: [x3, y3],
                    left_ctrl: [x2, y2],
                    right_ctrl: [x3, y3],
                });
            }
            "re" if stack.len() >= 4 => {
                let x = stack[stack.len() - 4];
                let y = stack[stack.len() - 3];
                let w = stack[stack.len() - 2];
                let h = stack[stack.len() - 1];
                let p1 = transform_pdf_point(state.ctm, x, y);
                let p2 = transform_pdf_point(state.ctm, x + w, y);
                let p3 = transform_pdf_point(state.ctm, x + w, y + h);
                let p4 = transform_pdf_point(state.ctm, x, y + h);
                path = [p1, p2, p3, p4]
                    .into_iter()
                    .map(|p| PathPoint {
                        anchor: p,
                        left_ctrl: p,
                        right_ctrl: p,
                    })
                    .collect();
                closed = true;
            }
            "h" => closed = true,
            "n" => {
                path.clear();
                closed = false;
            }
            "f" | "F" | "f*" => {
                if let Some(element) = painted_path_element(
                    stream_idx,
                    elements.len(),
                    &path,
                    true,
                    &state,
                    true,
                    false,
                ) {
                    elements.push(element);
                }
                path.clear();
                closed = false;
            }
            "S" => {
                if let Some(element) = painted_path_element(
                    stream_idx,
                    elements.len(),
                    &path,
                    closed,
                    &state,
                    false,
                    true,
                ) {
                    elements.push(element);
                }
                path.clear();
                closed = false;
            }
            "s" => {
                if let Some(element) = painted_path_element(
                    stream_idx,
                    elements.len(),
                    &path,
                    true,
                    &state,
                    false,
                    true,
                ) {
                    elements.push(element);
                }
                path.clear();
                closed = false;
            }
            "B" | "B*" | "b" | "b*" => {
                if let Some(element) = painted_path_element(
                    stream_idx,
                    elements.len(),
                    &path,
                    true,
                    &state,
                    true,
                    true,
                ) {
                    elements.push(element);
                }
                path.clear();
                closed = false;
            }
            _ => {}
        }

        if !matches!(token, "q" | "Q") {
            stack.clear();
        }
    }

    elements
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
    let consistent = radii
        .iter()
        .all(|&r| (r - mean).abs() / mean.max(0.001) < 0.05);
    if consistent {
        mean
    } else {
        0.0
    }
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
        element.name = Some(name.clone());
        element.id = format!("{}_{}", name, stream_idx);
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

    // Parse CTM rotation candidates
    let ctms = parse_ctms_from_stream(content_str);
    element.transform_candidates = ctms
        .iter()
        .map(|&(r, sx, sy, tx, ty)| [r, sx, sy, tx, ty])
        .collect();
    if let Some(&(rot, sx, sy, tx, ty)) = ctms.last() {
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
        page_tiles: Vec::new(),
        elements: Vec::new(),
        transform_candidates: Vec::new(),
        errors: Vec::new(),
    };

    static AI_ARTBOARD_RECT_RE: OnceLock<Regex> = OnceLock::new();
    let ai_artboard_rect_re = AI_ARTBOARD_RECT_RE.get_or_init(|| {
        Regex::new(
            r"%%AI_ArtboardRect\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)",
        )
        .unwrap()
    });

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
        } else if content_str.contains(" cm")
            || content_str.contains("\ncm")
            || content_str.contains(" re")
            || content_str.contains(" f")
            || content_str.contains(" S")
        {
            // Main PDF content stream: scan painted vector paths with their current PDF graphics
            // state. This keeps Illustrator comparison fixtures code/vector-based instead of
            // falling back to the PDF or PNG raster render.
            let painted_paths = parse_pdf_painted_path_elements(content_str, object_id.0 as usize);
            if !painted_paths.is_empty() {
                for element in painted_paths {
                    let is_dup = result
                        .transform_candidates
                        .iter()
                        .any(|e| e.id == element.id);
                    if !is_dup {
                        result.transform_candidates.push(element);
                    }
                }
            } else {
                // Fallback for streams that have transform/path metadata but no paint operator that
                // the vector parser can safely associate with an element yet.
                let mut element = Element {
                    id: format!("ctm_element_{}", object_id.0),
                    is_pseudo_element: true,
                    ..Default::default()
                };
                let ctms = parse_ctms_from_stream(content_str);
                element.transform_candidates = ctms
                    .iter()
                    .map(|&(r, sx, sy, tx, ty)| [r, sx, sy, tx, ty])
                    .collect();
                if let Some(&(rot, sx, sy, tx, ty)) = ctms.last() {
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
                    let is_dup = result
                        .transform_candidates
                        .iter()
                        .any(|e| e.id == element.id);
                    if !is_dup {
                        result.transform_candidates.push(element);
                    }
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
                        let name = names
                            .next()
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
            for caps in ai_artboard_rect_re.captures_iter(content_str) {
                if let (Some(x1), Some(y1), Some(x2), Some(y2)) =
                    (caps.get(1), caps.get(2), caps.get(3), caps.get(4))
                {
                    let x1: f64 = x1.as_str().parse().unwrap_or(0.0);
                    let y1: f64 = y1.as_str().parse().unwrap_or(0.0);
                    let x2: f64 = x2.as_str().parse().unwrap_or(0.0);
                    let y2: f64 = y2.as_str().parse().unwrap_or(0.0);
                    let name = format!("Artboard_{}", result.artboards.len() + 1);
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
                if let (Some(x1), Some(y1), Some(x2), Some(y2)) =
                    (caps.get(1), caps.get(2), caps.get(3), caps.get(4))
                {
                    let x1: f64 = x1.as_str().parse().unwrap_or(0.0);
                    let y1: f64 = y1.as_str().parse().unwrap_or(0.0);
                    let x2: f64 = x2.as_str().parse().unwrap_or(0.0);
                    let y2: f64 = y2.as_str().parse().unwrap_or(0.0);
                    result.artboards.push(Artboard {
                        name: "Artboard_1".to_string(),
                        x: x1,
                        y: y1,
                        width: (x2 - x1).abs(),
                        height: (y2 - y1).abs(),
                    });
                }
            }
        }
    }

    // Parse page tiles from ArtBox entries in PDF stream
    if result.page_tiles.is_empty() {
        if let Ok(bytes) = std::fs::read(path) {
            let full_content = String::from_utf8_lossy(&bytes);
            static ARTBOX_RE: OnceLock<Regex> = OnceLock::new();
            let re = ARTBOX_RE.get_or_init(|| {
                Regex::new(
                    r"ArtBox\[(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\]",
                )
                .unwrap()
            });
            for (i, caps) in re.captures_iter(&full_content).enumerate() {
                if let (Some(x1), Some(y1), Some(x2), Some(y2)) =
                    (caps.get(1), caps.get(2), caps.get(3), caps.get(4))
                {
                    let x1: f64 = x1.as_str().parse().unwrap_or(0.0);
                    let y1: f64 = y1.as_str().parse().unwrap_or(0.0);
                    let x2: f64 = x2.as_str().parse().unwrap_or(0.0);
                    let y2: f64 = y2.as_str().parse().unwrap_or(0.0);
                    result.page_tiles.push(PageTile {
                        name: format!("Page_{}", i + 1),
                        x: x1,
                        y: y1,
                        width: (x2 - x1).abs(),
                        height: (y2 - y1).abs(),
                    });
                }
            }
        }
    }

    // Parse tile region from %AI3_TileBox
    if result.page_tiles.is_empty() {
        if let Ok(bytes) = std::fs::read(path) {
            let full_content = String::from_utf8_lossy(&bytes);
            static TILEBOX_RE: OnceLock<Regex> = OnceLock::new();
            let re = TILEBOX_RE.get_or_init(|| {
                Regex::new(r"%AI3_TileBox:\s*(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)").unwrap()
            });
            if let Some(caps) = re.captures(&full_content) {
                if let (Some(x1), Some(y1), Some(x2), Some(y2)) =
                    (caps.get(1), caps.get(2), caps.get(3), caps.get(4))
                {
                    let x1: f64 = x1.as_str().parse().unwrap_or(0.0);
                    let y1: f64 = y1.as_str().parse().unwrap_or(0.0);
                    let x2: f64 = x2.as_str().parse().unwrap_or(0.0);
                    let y2: f64 = y2.as_str().parse().unwrap_or(0.0);
                    result.page_tiles.push(PageTile {
                        name: "TileRegion_1".to_string(),
                        x: x1,
                        y: y1,
                        width: (x2 - x1).abs(),
                        height: (y2 - y1).abs(),
                    });
                }
            }
        }
    }

    // If page tiles exist, prefer them over a single bounding-box artboard
    // (page tiles are more specific than a whole-canvas %%HiResBoundingBox)
    if !result.page_tiles.is_empty()
        && (result.artboards.is_empty()
            || (result.artboards.len() == 1 && !result.page_tiles.is_empty()))
    {
        result.artboards.clear();
        for tile in &result.page_tiles {
            result.artboards.push(Artboard {
                name: tile.name.clone(),
                x: tile.x,
                y: tile.y,
                width: tile.width,
                height: tile.height,
            });
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

    // Promote CTM/path candidates into transform-backed elements so they reach downstream
    // generation without leaking the internal `ctm_element_*` pseudo IDs into public output.
    for (idx, candidate) in result.transform_candidates.iter_mut().enumerate() {
        if candidate.is_pseudo_element {
            candidate.is_pseudo_element = false;
            candidate.id = format!("transform_candidate_{}", idx);
            candidate.element_type = Some("shape".to_string());
        }
    }
    result.elements.append(&mut result.transform_candidates);

    // Populate stable matching metadata
    let all_artboards: Vec<_> = result
        .artboards
        .iter()
        .map(|a| (a.name.clone(), a.x, a.y, a.x + a.width, a.y + a.height))
        .collect();
    for elem in &mut result.elements {
        let (x, y, w, h) = element_bounds(elem);
        elem.bounds = Some([x, y, w, h]);

        // Assign artboard_name by bounds if not already set
        if elem.artboard_name.is_none() {
            let mut assigned = false;
            for (name, ax, ay, ax2, ay2) in &all_artboards {
                let aw = ax2 - ax;
                let ah = ay2 - ay;
                if x < ax + aw && x + w > *ax && y < ay + ah && y + h > *ay {
                    elem.artboard_name = Some(name.clone());
                    assigned = true;
                    break;
                }
            }
            if !assigned && !all_artboards.is_empty() {
                elem.artboard_name = Some(all_artboards[0].0.clone());
            }
        }

        if elem.name.is_none() {
            elem.name = elem.artboard_name.clone();
        }
        if elem.element_type.is_none() {
            if !elem.path_points.is_empty() {
                elem.element_type = Some("path".to_string());
            } else if elem.is_pseudo_element {
                elem.element_type = Some("transform".to_string());
            } else {
                elem.element_type = Some("shape".to_string());
            }
        }
    }

    Ok(result)
}

fn element_bounds(elem: &Element) -> (f64, f64, f64, f64) {
    if !elem.path_points.is_empty() {
        let min_x = elem
            .path_points
            .iter()
            .flat_map(|p| [p.anchor[0], p.left_ctrl[0], p.right_ctrl[0]])
            .fold(f64::INFINITY, f64::min);
        let min_y = elem
            .path_points
            .iter()
            .flat_map(|p| [p.anchor[1], p.left_ctrl[1], p.right_ctrl[1]])
            .fold(f64::INFINITY, f64::min);
        let max_x = elem
            .path_points
            .iter()
            .flat_map(|p| [p.anchor[0], p.left_ctrl[0], p.right_ctrl[0]])
            .fold(f64::NEG_INFINITY, f64::max);
        let max_y = elem
            .path_points
            .iter()
            .flat_map(|p| [p.anchor[1], p.left_ctrl[1], p.right_ctrl[1]])
            .fold(f64::NEG_INFINITY, f64::max);
        let w = (max_x - min_x).max(1.0);
        let h = (max_y - min_y).max(1.0);
        (min_x, min_y, w, h)
    } else {
        let w = if elem.scale_x > 0.0 {
            elem.scale_x
        } else {
            1.0
        };
        let h = if elem.scale_y > 0.0 {
            elem.scale_y
        } else {
            1.0
        };
        (elem.translate_x, elem.translate_y, w, h)
    }
}

/// Convert an ai_parser `Element` to a codegen `LayoutElement` for code generation.
fn json_color(value: &Value) -> egui::Color32 {
    if let Some(hex) = value.as_str() {
        let hex = hex.trim_start_matches('#');
        if hex.len() >= 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128);
            return egui::Color32::from_rgb(r, g, b);
        }
    }
    let r = value.get("r").and_then(Value::as_u64).unwrap_or(128) as u8;
    let g = value.get("g").and_then(Value::as_u64).unwrap_or(128) as u8;
    let b = value.get("b").and_then(Value::as_u64).unwrap_or(128) as u8;
    let a = value.get("a").and_then(Value::as_u64).unwrap_or(255) as u8;
    egui::Color32::from_rgba_unmultiplied(r, g, b, a)
}

fn stroke_gradient_value(value: &Value) -> Option<GradientDef> {
    let kind = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("linear");
    if kind != "linear" && kind != "radial" {
        return None;
    }
    let stops = value
        .get("stops")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|stop| GradientStop {
                    position: stop.get("position").and_then(Value::as_f64).unwrap_or(0.0) as f32,
                    color: stop
                        .get("color")
                        .map(json_color)
                        .unwrap_or(egui::Color32::GRAY),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| {
            vec![
                GradientStop {
                    position: 0.0,
                    color: egui::Color32::GRAY,
                },
                GradientStop {
                    position: 1.0,
                    color: egui::Color32::LIGHT_GRAY,
                },
            ]
        });
    Some(GradientDef {
        gradient_type: if kind == "radial" {
            GradientType::Radial
        } else {
            GradientType::Linear
        },
        angle_deg: value.get("angle").and_then(Value::as_f64).unwrap_or(0.0) as f32,
        center: None,
        focal_point: None,
        radius: None,
        transform: None,
        stops,
    })
}

fn stroke_pattern_value(value: &Value) -> Option<egui_expressive::scene::PatternDef> {
    let kind = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("pattern");
    if kind == "linear" || kind == "radial" {
        return None;
    }
    Some(egui_expressive::scene::PatternDef {
        name: value
            .get("patternName")
            .or_else(|| value.get("pattern_name"))
            .and_then(Value::as_str)
            .unwrap_or("parser-stroke-pattern")
            .to_string(),
        seed: value.get("seed").and_then(Value::as_u64).unwrap_or(0) as u32,
        foreground: egui::Color32::from_rgba_unmultiplied(120, 120, 120, 220),
        background: egui::Color32::from_rgba_unmultiplied(240, 240, 240, 48),
        cell_size: value.get("cellSize").and_then(Value::as_f64).unwrap_or(8.0) as f32,
        mark_size: value.get("markSize").and_then(Value::as_f64).unwrap_or(2.0) as f32,
    })
}

fn element_to_layout(elem: &Element, idx: usize) -> LayoutElement {
    let id = if elem.id.is_empty() {
        format!("elem_{}", idx)
    } else {
        elem.id.clone()
    };
    // Use fill color from appearance_fills if available
    let fill_color = elem
        .appearance_fills
        .first()
        .map(|c| egui::Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a))
        .unwrap_or(egui::Color32::TRANSPARENT);
    let stroke_color = elem
        .appearance_strokes
        .first()
        .map(|s| egui::Color32::from_rgba_unmultiplied(s.r, s.g, s.b, s.a))
        .unwrap_or(egui::Color32::TRANSPARENT);
    let stroke_width = elem
        .appearance_strokes
        .first()
        .map(|s| s.width as f32)
        .unwrap_or(0.0);

    // Derive position and size from path_points bounding box when available,
    // otherwise fall back to CTM translate_x/translate_y with a default size.
    let (x, y, w, h) = element_bounds(elem);
    let (x, y, w, h) = (x as f32, y as f32, w as f32, h as f32);

    let mut layout_elem = LayoutElement::new(id, ElementType::Shape, x, y, w, h);
    layout_elem.fill = Some(fill_color);
    layout_elem.stroke = Some((stroke_width, stroke_color));
    layout_elem.rotation_deg = elem.rotation_deg as f32;
    layout_elem.corner_radius = elem.corner_radius as f32;
    layout_elem.opacity = 1.0;
    layout_elem.appearance_fills = elem
        .appearance_fills
        .iter()
        .map(|c| AppearanceFill {
            color: egui::Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a),
            gradient: None,
            opacity: c.opacity.unwrap_or(1.0) as f32,
            blend_mode: parse_blend_mode(&c.blend_mode),
        })
        .collect();
    layout_elem.appearance_strokes = elem
        .appearance_strokes
        .iter()
        .map(|s| AppearanceStroke {
            color: egui::Color32::from_rgba_unmultiplied(s.r, s.g, s.b, s.a),
            gradient: s.gradient.as_ref().and_then(stroke_gradient_value),
            pattern: s.gradient.as_ref().and_then(stroke_pattern_value),
            width: s.width as f32,
            opacity: s.opacity.unwrap_or(1.0) as f32,
            blend_mode: parse_blend_mode(&s.blend_mode),
            cap: s.cap.as_deref().and_then(|c| c.parse().ok()),
            join: s.join.as_deref().and_then(|j| j.parse().ok()),
            dash: s.dash.clone(),
            miter_limit: s.miter_limit,
        })
        .collect();
    layout_elem.effects = elem
        .live_effects
        .iter()
        .map(live_effect_to_effect_def)
        .collect();

    let mut appearance_stack = egui_expressive::scene::AppearanceStack::default();
    for fill in &layout_elem.appearance_fills {
        appearance_stack
            .entries
            .push(egui_expressive::scene::AppearanceEntry::Fill(
                egui_expressive::scene::FillLayer {
                    paint: egui_expressive::scene::PaintSource::Solid(fill.color),
                    opacity: fill.opacity,
                    blend_mode: fill.blend_mode.clone(),
                },
            ));
    }
    for effect in &layout_elem.effects {
        appearance_stack
            .entries
            .push(egui_expressive::scene::AppearanceEntry::Effect(
                egui_expressive::scene::EffectLayer {
                    effect_type: effect.effect_type.clone(),
                    params: effect.clone(),
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                },
            ));
    }
    for stroke in &layout_elem.appearance_strokes {
        let paint = if let Some(pattern) = &stroke.pattern {
            egui_expressive::scene::PaintSource::Pattern(pattern.clone())
        } else if let Some(gradient) = &stroke.gradient {
            if gradient.gradient_type == GradientType::Radial {
                egui_expressive::scene::PaintSource::RadialGradient(gradient.clone())
            } else {
                egui_expressive::scene::PaintSource::LinearGradient(gradient.clone())
            }
        } else {
            egui_expressive::scene::PaintSource::Solid(stroke.color)
        };
        appearance_stack
            .entries
            .push(egui_expressive::scene::AppearanceEntry::Stroke(
                egui_expressive::scene::StrokeLayer {
                    paint,
                    width: stroke.width,
                    opacity: stroke.opacity,
                    blend_mode: stroke.blend_mode.clone(),
                    cap: stroke.cap.clone(),
                    join: stroke.join.clone(),
                    dash: stroke.dash.clone(),
                    miter_limit: stroke.miter_limit,
                },
            ));
    }
    layout_elem.appearance_stack = appearance_stack;
    layout_elem.path_points = elem
        .path_points
        .iter()
        .map(|p| egui_expressive::codegen::PathPoint {
            anchor: [p.anchor[0] as f32, p.anchor[1] as f32],
            left_ctrl: [p.left_ctrl[0] as f32, p.left_ctrl[1] as f32],
            right_ctrl: [p.right_ctrl[0] as f32, p.right_ctrl[1] as f32],
        })
        .collect();
    layout_elem.path_closed = elem.path_closed;

    layout_elem
}

fn parse_blend_mode(mode: &str) -> BlendMode {
    match mode.to_lowercase().as_str() {
        "multiply" => BlendMode::Multiply,
        "screen" => BlendMode::Screen,
        "overlay" => BlendMode::Overlay,
        "darken" => BlendMode::Darken,
        "lighten" => BlendMode::Lighten,
        "color_dodge" | "colordodge" => BlendMode::ColorDodge,
        "color_burn" | "colorburn" => BlendMode::ColorBurn,
        "hard_light" | "hardlight" => BlendMode::HardLight,
        "soft_light" | "softlight" => BlendMode::SoftLight,
        "difference" => BlendMode::Difference,
        "exclusion" => BlendMode::Exclusion,
        "hue" => BlendMode::Hue,
        "saturation" => BlendMode::Saturation,
        "color" => BlendMode::Color,
        "luminosity" => BlendMode::Luminosity,
        _ => BlendMode::Normal,
    }
}

fn live_effect_to_effect_def(effect: &LiveEffect) -> EffectDef {
    let name = effect.name.to_ascii_lowercase();
    let params = &effect.params.params;
    if name.contains("noise") || name.contains("grain") || name.contains("mezzotint") {
        EffectDef {
            effect_type: EffectType::Noise,
            amount: param_f32(params, &["amount", "opacity", "intensity"], 0.16),
            scale: param_f32(params, &["scale", "size", "cellSize"], 2.0),
            seed: param_u32(params, &["seed"], 0),
            ..EffectDef::default()
        }
    } else if name.contains("blur") {
        EffectDef {
            effect_type: EffectType::GaussianBlur,
            radius: param_f32(params, &["radius", "blur"], 4.0),
            ..EffectDef::default()
        }
    } else if name.contains("drop shadow") || name.contains("dropshadow") {
        EffectDef {
            effect_type: EffectType::DropShadow,
            x: param_f32(params, &["horz", "x"], 0.0),
            y: param_f32(params, &["vert", "y"], 0.0),
            blur: param_f32(params, &["blur", "radius"], 4.0),
            spread: param_f32(params, &["spread"], 0.0),
            ..EffectDef::default()
        }
    } else if name.contains("inner shadow") || name.contains("innershadow") {
        EffectDef {
            effect_type: EffectType::InnerShadow,
            x: param_f32(params, &["horz", "x"], 0.0),
            y: param_f32(params, &["vert", "y"], 0.0),
            blur: param_f32(params, &["blur", "radius"], 4.0),
            ..EffectDef::default()
        }
    } else if name.contains("outer glow") || name.contains("outerglow") {
        EffectDef {
            effect_type: EffectType::OuterGlow,
            blur: param_f32(params, &["blur", "radius"], 4.0),
            ..EffectDef::default()
        }
    } else if name.contains("inner glow") || name.contains("innerglow") {
        EffectDef {
            effect_type: EffectType::InnerGlow,
            blur: param_f32(params, &["blur", "radius"], 4.0),
            ..EffectDef::default()
        }
    } else if name.contains("bevel") {
        EffectDef {
            effect_type: EffectType::Bevel,
            depth: param_f32(params, &["depth"], 2.0),
            angle: param_f32(params, &["angle"], 0.0),
            ..EffectDef::default()
        }
    } else {
        EffectDef {
            effect_type: EffectType::LiveEffect,
            ..EffectDef::default()
        }
    }
}

fn param_f32(params: &HashMap<String, Value>, keys: &[&str], fallback: f32) -> f32 {
    keys.iter()
        .find_map(|key| params.get(*key).and_then(|v| v.as_f64()).map(|v| v as f32))
        .unwrap_or(fallback)
}

fn param_u32(params: &HashMap<String, Value>, keys: &[&str], fallback: u32) -> u32 {
    keys.iter()
        .find_map(|key| params.get(*key).and_then(|v| v.as_u64()).map(|v| v as u32))
        .unwrap_or(fallback)
}

fn element_belongs_to_artboard(
    e: &Element,
    artboard_name: &str,
    artboard_rect: (f64, f64, f64, f64),
    all_artboards: &[(String, f64, f64, f64, f64)],
    is_first_artboard: bool,
) -> bool {
    if let Some(ref ab_name) = e.artboard_name {
        return ab_name == artboard_name;
    }

    let (x, y, w, h) = element_bounds(e);
    let (ax, ay, aw, ah) = artboard_rect;

    let intersects = x < ax + aw && x + w > ax && y < ay + ah && y + h > ay;
    if intersects {
        return true;
    }

    if is_first_artboard {
        let mut belongs_to_any = false;
        for (_, oax, oay, oax2, oay2) in all_artboards {
            let oaw = oax2 - oax;
            let oah = oay2 - oay;
            if x < oax + oaw && x + w > *oax && y < oay + oah && y + h > *oay {
                belongs_to_any = true;
                break;
            }
        }
        if !belongs_to_any {
            return true;
        }
    }

    false
}

pub fn generate_per_artboard_output(result: &AiParseResult) -> Vec<serde_json::Value> {
    let artboards = if result.artboards.is_empty() {
        vec![("default".to_string(), 0.0f64, 0.0f64, f64::MAX, f64::MAX)]
    } else {
        result
            .artboards
            .iter()
            .map(|a| (a.name.clone(), a.x, a.y, a.x + a.width, a.y + a.height))
            .collect::<Vec<_>>()
    };

    let mut entries: Vec<serde_json::Value> = Vec::new();
    for (artboard_idx, (name, _x1, _y1, _x2, _y2)) in artboards.iter().enumerate() {
        let sanitized = name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c.to_ascii_lowercase()
                } else {
                    '_'
                }
            })
            .collect::<String>();
        let sanitized = if sanitized.starts_with(|c: char| c.is_ascii_digit()) {
            format!("ab_{}", sanitized)
        } else if sanitized.is_empty() {
            "artboard".to_string()
        } else {
            sanitized
        };
        let filename = format!("{}.rs", sanitized);
        let selected_elements: Vec<&Element> = result
            .elements
            .iter()
            .filter(|e| !e.is_pseudo_element || !e.path_points.is_empty())
            .filter(|e| {
                element_belongs_to_artboard(
                    e,
                    name,
                    (*_x1, *_y1, _x2 - _x1, _y2 - _y1),
                    &artboards,
                    artboard_idx == 0,
                )
            })
            .collect();
        let element_count = selected_elements.len();
        let artboard_info = artboards.iter().find(|(n, _, _, _, _)| n == name);
        let (ab_w, ab_h) = artboard_info
            .map(|(_, x1, y1, x2, y2)| ((x2 - x1).abs(), (y2 - y1).abs()))
            .unwrap_or((375.0, 812.0));
        let layout_elements: Vec<LayoutElement> = selected_elements
            .iter()
            .enumerate()
            .map(|(i, e)| element_to_layout(e, i))
            .collect();
        let code = generate_artboard_file(
            name,
            ab_w as f32,
            ab_h as f32,
            &layout_elements,
            &std::collections::HashMap::new(),
        );
        entries.push(serde_json::json!({
            "artboard": name,
            "filename": filename,
            "width": ab_w,
            "height": ab_h,
            "element_count": element_count,
            "code": code,
            "elements": result.elements.iter()
                .filter(|e| {
                    element_belongs_to_artboard(
                        e,
                        name,
                        (*_x1, *_y1, _x2 - _x1, _y2 - _y1),
                        &artboards,
                        artboard_idx == 0,
                    )
                })
                .collect::<Vec<_>>(),
        }));
    }
    entries
}

pub fn generate_canvas_output(result: &AiParseResult) -> Vec<serde_json::Value> {
    let mut max_x = 0.0f64;
    let mut max_y = 0.0f64;

    for artboard in &result.artboards {
        max_x = max_x.max(artboard.x + artboard.width);
        max_y = max_y.max(artboard.y + artboard.height);
    }
    for element in &result.elements {
        let (x, y, w, h) = element_bounds(element);
        max_x = max_x.max(x + w);
        max_y = max_y.max(y + h);
    }

    let width = max_x.ceil().max(1.0);
    let height = max_y.ceil().max(1.0);
    let mut layout_elements: Vec<LayoutElement> = Vec::new();
    let mut background = LayoutElement::new(
        "pdf_page_background".to_string(),
        ElementType::Shape,
        0.0,
        0.0,
        width as f32,
        height as f32,
    );
    background.fill = Some(egui::Color32::WHITE);
    background.is_opaque = true;
    layout_elements.push(background);

    layout_elements.extend(
        result
            .elements
            .iter()
            .filter(|e| !e.is_pseudo_element || !e.path_points.is_empty())
            .enumerate()
            .map(|(i, e)| element_to_layout(e, i)),
    );
    let code = generate_artboard_file(
        "Full Canvas",
        width as f32,
        height as f32,
        &layout_elements,
        &std::collections::HashMap::new(),
    );

    vec![serde_json::json!({
        "artboard": "Full Canvas",
        "filename": "full_canvas.rs",
        "width": width,
        "height": height,
        "element_count": layout_elements.len(),
        "code": code,
        "elements": result.elements,
    })]
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: ai-parser <file.ai> [--pretty] [--per-artboard] [--full-canvas]");
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);
    let pretty = args.iter().any(|a| a == "--pretty");
    let per_artboard = args.iter().any(|a| a == "--per-artboard");
    let full_canvas = args.iter().any(|a| a == "--full-canvas");

    let result = match parse_ai_file(path) {
        Ok(r) => r,
        Err(e) => {
            let error_result = AiParseResult {
                version: "1.0".to_string(),
                source_file: path.to_string_lossy().to_string(),
                ai_version: String::new(),
                artboards: Vec::new(),
                page_tiles: Vec::new(),
                elements: Vec::new(),
                transform_candidates: Vec::new(),
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

    if per_artboard || full_canvas {
        let entries = if full_canvas {
            generate_canvas_output(&result)
        } else {
            generate_per_artboard_output(&result)
        };
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
    fn test_layer_name_not_artboard() {
        let content = "%%Layer: MyLayer\n%AI8_BeginLayer\n[ 1.0 0.0 0.0 1.0 ] Xa";
        let mut errors = vec![];
        let elem = parse_aip_private_stream(content.as_bytes(), 0, &mut errors).unwrap();
        assert_eq!(elem.name, Some("MyLayer".to_string()));
        assert_eq!(elem.artboard_name, None);
    }

    #[test]
    fn test_element_to_layout_copies_path() {
        let mut elem = Element::default();
        elem.path_points.push(PathPoint {
            anchor: [1.0, 2.0],
            left_ctrl: [3.0, 4.0],
            right_ctrl: [5.0, 6.0],
        });
        elem.path_closed = true;
        let layout = element_to_layout(&elem, 0);
        assert_eq!(layout.path_points.len(), 1);
        assert_eq!(layout.path_points[0].anchor, [1.0, 2.0]);
        assert!(layout.path_closed);
    }

    #[test]
    fn test_parse_appearance_stroke_properties() {
        let content = "1 J\n2 j\n4.0 M\n[2.0 4.0] 0 d\n2.5 w\n[ 0.0 1.0 0.0 1.0 ] xa";
        let (_, strokes) = parse_appearance(content);
        assert_eq!(strokes.len(), 1);
        assert_eq!(strokes[0].g, 255);
        assert_eq!(strokes[0].width, 2.5);
        assert_eq!(strokes[0].cap.as_deref(), Some("round"));
        assert_eq!(strokes[0].join.as_deref(), Some("bevel"));
        assert_eq!(strokes[0].miter_limit, Some(4.0));
        assert_eq!(strokes[0].dash.as_deref(), Some(&[2.0, 4.0][..]));
    }

    #[test]
    fn test_parse_appearance_case_sensitive() {
        let content = "[ 1.0 0.0 0.0 1.0 ] Xa\n[ 0.0 1.0 0.0 1.0 ] xa";
        let (fills, strokes) = parse_appearance(content);
        assert_eq!(fills.len(), 1);
        assert_eq!(fills[0].r, 255);
        assert_eq!(fills[0].opacity, Some(1.0));
        assert_eq!(strokes.len(), 1);
        assert_eq!(strokes[0].g, 255);
        assert_eq!(strokes[0].opacity, Some(1.0));
    }

    #[test]
    fn test_parse_appearance_blend_mode() {
        let content = "/BM /Multiply\n[ 1.0 0.0 0.0 1.0 ] Xa";
        let (fills, _) = parse_appearance(content);
        assert_eq!(fills.len(), 1);
        assert_eq!(fills[0].blend_mode, "Multiply");

        let content2 = "/BlendMode /Screen\n[ 0.0 1.0 0.0 1.0 ] xa";
        let (_, strokes) = parse_appearance(content2);
        assert_eq!(strokes.len(), 1);
        assert_eq!(strokes[0].blend_mode, "Screen");
    }

    #[test]
    fn test_parse_appearance_gradient_fallback() {
        let content = "0.0 0.0 0.0 1.0 k\n/Pattern cs\n sh\n";
        let (fills, _strokes) = parse_appearance(content);
        assert_eq!(fills.len(), 1);
        assert_eq!(fills[0].r, 128); // Fallback color
    }

    #[test]
    fn test_parse_appearance_stroke_pattern_surfaces_gradient_metadata() {
        let content = "2 w\n/Pattern CS\n";
        let (_fills, strokes) = parse_appearance(content);
        assert_eq!(strokes.len(), 1);
        assert!(strokes[0].gradient.is_some());
        let layout = element_to_layout(
            &Element {
                id: "stroke_pattern".to_string(),
                appearance_strokes: strokes,
                ..Default::default()
            },
            0,
        );
        let egui_expressive::scene::AppearanceEntry::Stroke(stroke) =
            &layout.appearance_stack.entries[0]
        else {
            panic!("expected stroke");
        };
        assert!(matches!(
            stroke.paint,
            egui_expressive::scene::PaintSource::Pattern(_)
        ));
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
        let result = parse_ctms_from_stream(content);
        assert!(!result.is_empty());
        let (rot, sx, sy, tx, ty) = result.last().unwrap();
        assert!(
            (rot).abs() < 0.001,
            "identity rotation should be 0, got {}",
            rot
        );
        assert!((sx - 1.0).abs() < 0.001);
        assert!((sy - 1.0).abs() < 0.001);
        assert!((tx).abs() < 0.001);
        assert!((ty).abs() < 0.001);
    }

    #[test]
    fn test_parse_pdf_painted_paths_extracts_code_drawn_fill() {
        let content = "q 1 0 0 1 10 20 cm 0.1 0.2 0.3 rg 0 0 30 40 re f Q";
        let elements = parse_pdf_painted_path_elements(content, 7);
        assert_eq!(elements.len(), 1);
        let element = &elements[0];
        assert_eq!(element.path_points.len(), 4);
        assert!(element.path_closed);
        assert_eq!(element.appearance_fills.len(), 1);
        assert_eq!(element.appearance_fills[0].r, 26);
        assert_eq!(element.appearance_fills[0].g, 51);
        assert_eq!(element.appearance_fills[0].b, 77);
        assert_eq!(element.bounds.unwrap(), [10.0, 20.0, 30.0, 40.0]);
    }

    #[test]
    fn test_parse_pdf_painted_paths_extracts_stroke_style() {
        let content = "2 J 1 j 4 M 3 w 0 1 0 RG 10 10 m 50 10 l S";
        let elements = parse_pdf_painted_path_elements(content, 8);
        assert_eq!(elements.len(), 1);
        let strokes = &elements[0].appearance_strokes;
        assert_eq!(strokes.len(), 1);
        assert_eq!(strokes[0].g, 255);
        assert_eq!(strokes[0].width, 3.0);
        assert_eq!(strokes[0].cap.as_deref(), Some("square"));
        assert_eq!(strokes[0].join.as_deref(), Some("round"));
        assert_eq!(strokes[0].miter_limit, Some(4.0));
    }

    #[test]
    fn test_parse_ctm_90deg() {
        // 90 degree rotation: a=0, b=1, c=-1, d=0
        let content = "0 1 -1 0 0 0 cm";
        let result = parse_ctms_from_stream(content);
        assert!(!result.is_empty());
        let (rot, _sx, _sy, _tx, _ty) = result.last().unwrap();
        assert!((rot - 90.0).abs() < 0.01, "expected 90 deg, got {}", rot);
    }

    #[test]
    fn test_detect_corner_radius_zero() {
        // A simple square has no control handles → radius 0
        let points = vec![
            PathPoint {
                anchor: [0.0, 0.0],
                left_ctrl: [0.0, 0.0],
                right_ctrl: [0.0, 0.0],
            },
            PathPoint {
                anchor: [100.0, 0.0],
                left_ctrl: [100.0, 0.0],
                right_ctrl: [100.0, 0.0],
            },
            PathPoint {
                anchor: [100.0, 100.0],
                left_ctrl: [100.0, 100.0],
                right_ctrl: [100.0, 100.0],
            },
            PathPoint {
                anchor: [0.0, 100.0],
                left_ctrl: [0.0, 100.0],
                right_ctrl: [0.0, 100.0],
            },
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
            PathPoint {
                anchor: [r, 0.0],
                left_ctrl: [r - h, 0.0],
                right_ctrl: [r + h, 0.0],
            }, // top-left corner right
            PathPoint {
                anchor: [100.0 - r, 0.0],
                left_ctrl: [100.0 - r - h, 0.0],
                right_ctrl: [100.0 - r + h, 0.0],
            }, // top-right corner left
            PathPoint {
                anchor: [100.0, r],
                left_ctrl: [100.0, r - h],
                right_ctrl: [100.0, r + h],
            }, // right-top corner
            PathPoint {
                anchor: [100.0, 100.0 - r],
                left_ctrl: [100.0, 100.0 - r - h],
                right_ctrl: [100.0, 100.0 - r + h],
            },
            PathPoint {
                anchor: [100.0 - r, 100.0],
                left_ctrl: [100.0 - r + h, 100.0],
                right_ctrl: [100.0 - r - h, 100.0],
            },
            PathPoint {
                anchor: [r, 100.0],
                left_ctrl: [r + h, 100.0],
                right_ctrl: [r - h, 100.0],
            },
            PathPoint {
                anchor: [0.0, 100.0 - r],
                left_ctrl: [0.0, 100.0 - r + h],
                right_ctrl: [0.0, 100.0 - r - h],
            },
            PathPoint {
                anchor: [0.0, r],
                left_ctrl: [0.0, r + h],
                right_ctrl: [0.0, r - h],
            },
        ];
        let detected = detect_corner_radius(&points);
        assert!(
            (detected - r).abs() < 2.0,
            "expected radius ~{}, got {}",
            r,
            detected
        );
    }

    #[test]
    fn test_parse_path_geometry() {
        let content = "10 20 m 30 40 l 50 60 70 80 90 100 c h";
        let (points, closed) = parse_path_geometry(content);
        assert!(closed);
        assert_eq!(points.len(), 3);

        // m 10 20
        assert_eq!(points[0].anchor, [10.0, 20.0]);

        // l 30 40
        assert_eq!(points[1].anchor, [30.0, 40.0]);

        // c 50 60 70 80 90 100
        assert_eq!(points[2].anchor, [90.0, 100.0]);
        assert_eq!(points[2].left_ctrl, [70.0, 80.0]);
        assert_eq!(points[1].right_ctrl, [50.0, 60.0]);
    }

    #[test]
    fn test_generate_per_artboard_output() {
        let result = AiParseResult {
            version: "1.0".to_string(),
            source_file: "test.ai".to_string(),
            ai_version: "25.0".to_string(),
            artboards: vec![Artboard {
                name: "Artboard_1".to_string(),
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            }],
            page_tiles: vec![],
            elements: vec![Element {
                id: "elem_1".to_string(),
                artboard_name: Some("Artboard_1".to_string()),
                ..Default::default()
            }],
            transform_candidates: vec![],
            errors: vec![],
        };

        let output = generate_per_artboard_output(&result);
        assert_eq!(output.len(), 1);
        let obj = output[0].as_object().unwrap();
        assert_eq!(obj.get("artboard").unwrap().as_str().unwrap(), "Artboard_1");
        assert_eq!(obj.get("element_count").unwrap().as_u64().unwrap(), 1);
        assert!(obj.get("elements").unwrap().as_array().unwrap().len() == 1);
    }

    #[test]
    fn test_parse_ai_file_real_sample() {
        let path = Path::new("UI assets from illustrator.ai");
        if path.exists() {
            let result = parse_ai_file(path).unwrap();
            assert!(!result.elements.is_empty(), "Should find elements");
            assert!(!result.artboards.is_empty(), "Should find artboards");
            assert!(
                result
                    .elements
                    .iter()
                    .any(|el| !el.appearance_fills.is_empty() || !el.appearance_strokes.is_empty()),
                "real Illustrator fixture should yield code-drawn vector appearances"
            );
            let per_artboard = generate_per_artboard_output(&result);
            assert!(
                !per_artboard.is_empty(),
                "Should generate per-artboard output"
            );

            let reference_png = Path::new("UI assets from illustrator.png");
            if reference_png.exists() {
                let reference = image::open(reference_png).unwrap().to_rgba8();
                assert_eq!([reference.width(), reference.height()], [5102, 3679]);
                let max_artboard_width = result
                    .artboards
                    .iter()
                    .map(|artboard| artboard.width)
                    .fold(0.0, f64::max);
                let max_artboard_height = result
                    .artboards
                    .iter()
                    .map(|artboard| artboard.height)
                    .fold(0.0, f64::max);
                assert!(reference.width() as f64 >= max_artboard_width);
                assert!(reference.height() as f64 >= max_artboard_height);
            }
        }
    }
}
