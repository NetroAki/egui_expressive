use super::*;

pub(crate) fn element_bounds(elem: &Element) -> (f64, f64, f64, f64) {
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
pub(crate) fn json_color(value: &Value) -> egui::Color32 {
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

pub(crate) fn stroke_gradient_value(value: &Value) -> Option<GradientDef> {
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

pub(crate) fn stroke_pattern_value(value: &Value) -> Option<egui_expressive::scene::PatternDef> {
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

pub(crate) fn element_to_layout(elem: &Element, idx: usize) -> LayoutElement {
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

pub(crate) fn parse_blend_mode(mode: &str) -> BlendMode {
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

pub(crate) fn live_effect_to_effect_def(effect: &LiveEffect) -> EffectDef {
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

pub(crate) fn param_f32(params: &HashMap<String, Value>, keys: &[&str], fallback: f32) -> f32 {
    keys.iter()
        .find_map(|key| params.get(*key).and_then(|v| v.as_f64()).map(|v| v as f32))
        .unwrap_or(fallback)
}

pub(crate) fn param_u32(params: &HashMap<String, Value>, keys: &[&str], fallback: u32) -> u32 {
    keys.iter()
        .find_map(|key| params.get(*key).and_then(|v| v.as_u64()).map(|v| v as u32))
        .unwrap_or(fallback)
}

pub(crate) fn element_belongs_to_artboard(
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
