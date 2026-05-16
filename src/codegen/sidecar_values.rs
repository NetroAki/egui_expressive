use super::*;

pub(crate) fn parse_sidecar_color(
    obj: &serde_json::Map<String, serde_json::Value>,
) -> Option<Color32> {
    if let Some(c_str) = obj.get("color").and_then(|v| v.as_str()) {
        crate::svg::parse_svg_color(c_str)
    } else if let (Some(r), Some(g), Some(b)) = (obj.get("r"), obj.get("g"), obj.get("b")) {
        Some(Color32::from_rgb(
            r.as_u64().unwrap_or(0) as u8,
            g.as_u64().unwrap_or(0) as u8,
            b.as_u64().unwrap_or(0) as u8,
        ))
    } else {
        None
    }
}

pub(crate) fn parse_sidecar_gradient(v: &serde_json::Value) -> Option<GradientDef> {
    let g = v.as_object()?;
    let type_name = g.get("type").and_then(|t| t.as_str());
    let gradient_type = match type_name {
        Some("radial") => GradientType::Radial,
        Some("linear") | None => GradientType::Linear,
        Some(_) => return None,
    };
    let parse_point = |value: Option<&serde_json::Value>| -> Option<[f32; 2]> {
        let value = value?;
        if let Some(arr) = value.as_array() {
            return Some([arr.first()?.as_f64()? as f32, arr.get(1)?.as_f64()? as f32]);
        }
        let obj = value.as_object()?;
        Some([
            obj.get("x")?.as_f64()? as f32,
            obj.get("y")?.as_f64()? as f32,
        ])
    };
    let parse_transform = |value: Option<&serde_json::Value>| -> Option<[f32; 6]> {
        let value = value?;
        if let Some(arr) = value.as_array() {
            return Some([
                arr.first()?.as_f64()? as f32,
                arr.get(1)?.as_f64()? as f32,
                arr.get(2)?.as_f64()? as f32,
                arr.get(3)?.as_f64()? as f32,
                arr.get(4)?.as_f64()? as f32,
                arr.get(5)?.as_f64()? as f32,
            ]);
        }
        let obj = value.as_object()?;
        let number = |names: &[&str]| -> Option<f32> {
            names
                .iter()
                .find_map(|name| obj.get(*name).and_then(|v| v.as_f64()))
                .map(|v| v as f32)
        };
        Some([
            number(&["a", "mValueA"])?,
            number(&["b", "mValueB"])?,
            number(&["c", "mValueC"])?,
            number(&["d", "mValueD"])?,
            number(&["e", "tx", "mValueTX"])?,
            number(&["f", "ty", "mValueTY"])?,
        ])
    };
    let stops = g
        .get("stops")
        .and_then(|s| s.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|stop| {
                    let position = stop.get("position")?.as_f64()? as f32;
                    let color = stop
                        .get("color")?
                        .as_str()
                        .and_then(crate::svg::parse_svg_color)
                        .unwrap_or(egui::Color32::BLACK);
                    let opacity = stop
                        .get("opacity")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.0)
                        .clamp(0.0, 1.0) as f32;
                    let [r, g, b, a] = color.to_srgba_unmultiplied();
                    Some(GradientStop {
                        position,
                        color: Color32::from_rgba_unmultiplied(
                            r,
                            g,
                            b,
                            (a as f32 * opacity).round() as u8,
                        ),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    Some(GradientDef {
        gradient_type,
        angle_deg: g
            .get("angle")
            .and_then(|a| a.as_f64())
            .map(|v| v as f32)
            .unwrap_or(0.0),
        center: parse_point(g.get("center")),
        focal_point: parse_point(g.get("focalPoint").or_else(|| g.get("focal_point"))),
        radius: g.get("radius").and_then(|r| r.as_f64()).map(|r| r as f32),
        transform: parse_transform(g.get("transform").or_else(|| g.get("matrix"))),
        stops,
    })
}

pub(crate) fn parse_sidecar_pattern(v: &serde_json::Value) -> Option<crate::scene::PatternDef> {
    if let Some(name) = v.as_str() {
        let seed = stable_pattern_seed(name);
        let (foreground, background) = seeded_pattern_colors(seed);
        return Some(crate::scene::PatternDef {
            name: name.to_string(),
            seed,
            foreground,
            background,
            cell_size: 8.0,
            mark_size: 1.0,
        });
    }
    let g = v.as_object()?;
    let type_name = g.get("type").and_then(|t| t.as_str());
    match type_name {
        Some("linear" | "radial") => return None,
        Some(_) => {}
        None => {
            let has_pattern_metadata = g.contains_key("patternName")
                || g.contains_key("pattern_name")
                || g.contains_key("name")
                || g.contains_key("seed")
                || g.contains_key("cellSize")
                || g.contains_key("cell_size");
            if !has_pattern_metadata {
                return None;
            }
        }
    }
    let name = g
        .get("patternName")
        .or_else(|| g.get("pattern_name"))
        .or_else(|| g.get("name"))
        .and_then(|v| v.as_str())
        .or(type_name)
        .unwrap_or("pattern")
        .to_string();
    let seed = g
        .get("seed")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or_else(|| stable_pattern_seed(&name));
    let (foreground, background) = seeded_pattern_colors(seed);
    let cell_size = g
        .get("cellSize")
        .or_else(|| g.get("cell_size"))
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(8.0)
        .clamp(2.0, 64.0);
    let mark_size = g
        .get("markSize")
        .or_else(|| g.get("mark_size"))
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(1.0)
        .clamp(0.5, 16.0);
    Some(crate::scene::PatternDef {
        name,
        seed,
        foreground,
        background,
        cell_size,
        mark_size,
    })
}

pub(crate) fn parse_sidecar_effect(e: &serde_json::Value) -> Option<EffectDef> {
    let effect_type_str = e
        .get("effect_type")
        .or_else(|| e.get("effectType"))
        .or_else(|| e.get("type"))?
        .as_str()?;
    let effect_type = match effect_type_str {
        "dropShadow" | "drop-shadow" => EffectType::DropShadow,
        "innerShadow" | "inner-shadow" => EffectType::InnerShadow,
        "outerGlow" | "outer-glow" => EffectType::OuterGlow,
        "innerGlow" | "inner-glow" => EffectType::InnerGlow,
        "gaussianBlur" | "gaussian-blur" => EffectType::GaussianBlur,
        "bevel" => EffectType::Bevel,
        "feather" => EffectType::Feather,
        "noise" | "grain" => EffectType::Noise,
        "liveEffect" | "live-effect" => EffectType::LiveEffect,
        _ => EffectType::Unknown(effect_type_str.to_string()),
    };
    let f32_key = |key: &str, fallback: f32| {
        e.get(key)
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(fallback)
    };
    Some(EffectDef {
        effect_type,
        x: f32_key("x", 0.0),
        y: f32_key("y", 0.0),
        blur: f32_key("blur", 0.0),
        spread: f32_key("spread", 0.0),
        color: e
            .get("color")
            .and_then(|v| v.as_str())
            .and_then(crate::svg::parse_svg_color)
            .unwrap_or(egui::Color32::BLACK),
        blend_mode: e
            .get("blendMode")
            .and_then(|v| v.as_str())
            .unwrap_or("normal")
            .parse::<BlendMode>()
            .unwrap_or(BlendMode::Normal),
        depth: f32_key("depth", 0.0),
        angle: f32_key("angle", 0.0),
        highlight: e
            .get("highlight")
            .and_then(|v| v.as_str())
            .and_then(crate::svg::parse_svg_color),
        shadow_color: e
            .get("shadowColor")
            .and_then(|v| v.as_str())
            .and_then(crate::svg::parse_svg_color),
        radius: f32_key("radius", 0.0),
        amount: f32_key("amount", 0.0),
        scale: f32_key("scale", 2.0),
        seed: e
            .get("seed")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(0),
    })
}

pub(crate) fn parse_sidecar_text_runs(elem_value: &serde_json::Value) -> Vec<TextRun> {
    elem_value
        .get("textRuns")
        .and_then(|v| v.as_array())
        .map(|runs| {
            runs.iter()
                .filter_map(|r| {
                    let ro = r.as_object()?;
                    Some(TextRun {
                        text: ro.get("text")?.as_str()?.to_string(),
                        size: ro
                            .get("style")
                            .and_then(|s| s.get("size"))
                            .and_then(|v| v.as_f64())
                            .unwrap_or(14.0) as f32,
                        weight: ro
                            .get("style")
                            .and_then(|s| s.get("weight"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(400) as u16,
                        color: ro.get("style").and_then(|s| s.get("color")).and_then(|c| {
                            let co = c.as_object()?;
                            Some(Color32::from_rgb(
                                co.get("r")?.as_u64()? as u8,
                                co.get("g")?.as_u64()? as u8,
                                co.get("b")?.as_u64()? as u8,
                            ))
                        }),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn parse_sidecar_third_party_effects(
    elem_value: &serde_json::Value,
) -> Vec<ThirdPartyEffect> {
    elem_value
        .get("thirdPartyEffects")
        .and_then(|v| v.as_array())
        .map(|effects| {
            effects
                .iter()
                .filter_map(|e| {
                    let eo = e.as_object()?;
                    Some(ThirdPartyEffect {
                        effect_type: eo.get("type")?.as_str()?.to_string(),
                        opaque: eo.get("opaque").and_then(|v| v.as_bool()).unwrap_or(false),
                        note: eo
                            .get("note")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn parse_sidecar_notes(elem_value: &serde_json::Value) -> Vec<String> {
    elem_value
        .get("notes")
        .and_then(|v| v.as_array())
        .map(|notes| {
            notes
                .iter()
                .filter_map(|n| n.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn parse_sidecar_appearance_fills(
    elem_value: &serde_json::Value,
) -> Vec<AppearanceFill> {
    elem_value
        .get("appearanceFills")
        .and_then(|v| v.as_array())
        .map(|fills| {
            fills
                .iter()
                .filter_map(|f| {
                    let fo = f.as_object()?;
                    Some(AppearanceFill {
                        color: parse_sidecar_color(fo).unwrap_or(Color32::BLACK),
                        gradient: fo.get("gradient").and_then(parse_sidecar_gradient),
                        opacity: fo.get("opacity").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                        blend_mode: fo
                            .get("blendMode")
                            .and_then(|v| v.as_str())
                            .unwrap_or("normal")
                            .parse::<BlendMode>()
                            .unwrap_or(BlendMode::Normal),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn parse_sidecar_appearance_strokes(
    elem_value: &serde_json::Value,
) -> Vec<AppearanceStroke> {
    elem_value
        .get("appearanceStrokes")
        .and_then(|v| v.as_array())
        .map(|strokes| {
            strokes
                .iter()
                .filter_map(|s| {
                    let so = s.as_object()?;
                    Some(AppearanceStroke {
                        color: parse_sidecar_color(so).unwrap_or(Color32::BLACK),
                        gradient: so.get("gradient").and_then(parse_sidecar_gradient),
                        pattern: so
                            .get("gradient")
                            .and_then(parse_sidecar_pattern)
                            .or_else(|| so.get("pattern").and_then(parse_sidecar_pattern)),
                        width: so.get("width").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                        opacity: so.get("opacity").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                        blend_mode: so
                            .get("blendMode")
                            .and_then(|v| v.as_str())
                            .unwrap_or("normal")
                            .parse::<BlendMode>()
                            .unwrap_or(BlendMode::Normal),
                        cap: so
                            .get("cap")
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse().ok()),
                        join: so
                            .get("join")
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse().ok()),
                        dash: so.get("dash").and_then(|v| v.as_array()).map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_f64())
                                .map(|f| f as f32)
                                .collect()
                        }),
                        miter_limit: so
                            .get("miterLimit")
                            .and_then(|v| v.as_f64())
                            .map(|v| v as f32),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn parse_sidecar_path_points(elem_value: &serde_json::Value) -> Vec<PathPoint> {
    elem_value
        .get("pathPoints")
        .and_then(|v| v.as_array())
        .map(|pts| {
            pts.iter()
                .filter_map(|p| {
                    let po = p.as_object()?;
                    let anchor = po.get("anchor").and_then(|v| v.as_array())?;
                    let left_ctrl = po
                        .get("left_ctrl")
                        .or_else(|| po.get("leftDir"))
                        .and_then(|v| v.as_array())
                        .unwrap_or(anchor);
                    let right_ctrl = po
                        .get("right_ctrl")
                        .or_else(|| po.get("rightDir"))
                        .and_then(|v| v.as_array())
                        .unwrap_or(anchor);
                    Some(PathPoint {
                        anchor: [
                            anchor.first()?.as_f64()? as f32,
                            anchor.get(1)?.as_f64()? as f32,
                        ],
                        left_ctrl: [
                            left_ctrl.first()?.as_f64()? as f32,
                            left_ctrl.get(1)?.as_f64()? as f32,
                        ],
                        right_ctrl: [
                            right_ctrl.first()?.as_f64()? as f32,
                            right_ctrl.get(1)?.as_f64()? as f32,
                        ],
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}
