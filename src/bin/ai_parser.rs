//! Adobe Illustrator (.ai) file parser
//!
//! Parses .ai files (PDF wrappers) and extracts visual properties from AIPrivateData streams.

use flate2::read::DeflateDecoder;
use lopdf::Document;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

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

/// Decompress a FlateDecode stream manually if lopdf doesn't handle it
fn decompress_flatedecode(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder = DeflateDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| format!("decompression failed: {}", e))?;
    Ok(decompressed)
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
                        while let Some(c) = chars.next() {
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

        let depth = values.get(0).copied().unwrap_or(100.0);
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

    if element.live_effects.is_empty()
        && element.appearance_fills.is_empty()
        && element.appearance_strokes.is_empty()
        && element.envelope_mesh.is_none()
        && element.three_d.is_none()
        && element.mesh_patches.is_empty()
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
        }

        if result.ai_version.is_empty() {
            let version = extract_ai_version(content_str);
            if !version.is_empty() {
                result.ai_version = version;
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
                                if let Some(re) =
                                    Regex::new(r"Adobe Illustrator[^\d]*([\d.]+)").ok()
                                {
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

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: ai-parser <file.ai> [--pretty]");
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);
    let pretty = args.iter().any(|a| a == "--pretty");

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
}
