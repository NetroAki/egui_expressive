use super::*;

pub(crate) fn parse_dict_data(data: &str) -> HashMap<String, Value> {
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
pub(crate) fn parse_live_effect_xml(content: &str) -> Vec<LiveEffect> {
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
pub(crate) fn parse_envelope_mesh(content: &str) -> Option<EnvelopeMesh> {
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
pub(crate) fn parse_3d_effect(content: &str) -> Option<ThreeD> {
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
pub(crate) fn parse_appearance(content: &str) -> (Vec<Color>, Vec<Stroke>) {
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
pub(crate) fn parse_mesh_patches(content: &str) -> Vec<MeshPatch> {
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
pub(crate) fn extract_layer_name(content: &str) -> Option<String> {
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
pub(crate) fn extract_ai_version(content: &str) -> String {
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
pub(crate) fn parse_ctms_from_stream(content: &str) -> Vec<(f64, f64, f64, f64, f64)> {
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
