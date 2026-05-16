use super::*;
pub struct ArtboardInfo {
    pub name: String,
    pub width: f32,
    pub height: f32,
}
pub fn parse_json_sidecar(json: &str) -> Result<(ArtboardInfo, Vec<LayoutElement>), String> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("JSON parse error: {}", e))?;
    let artboard = value.get("artboard").ok_or("Missing 'artboard' field")?;
    let name = artboard
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled")
        .to_string();
    let width = artboard
        .get("width")
        .and_then(|v| v.as_f64())
        .unwrap_or(375.0) as f32;
    let height = artboard
        .get("height")
        .and_then(|v| v.as_f64())
        .unwrap_or(812.0) as f32;
    let artboard_info = ArtboardInfo {
        name,
        width,
        height,
    };
    let elements_array = value
        .get("elements")
        .ok_or("Missing 'elements' field")?
        .as_array()
        .ok_or("'elements' must be an array")?;
    let mut elements = Vec::new();
    for (i, elem_value) in elements_array.iter().enumerate() {
        if let Some(mut el) = parse_element(elem_value) {
            if el.id == "elem_" || el.id.starts_with("elem_") {
                el.id = format!("elem_{}", i);
            }
            elements.push(el);
        }
    }
    Ok((artboard_info, elements))
}
pub(crate) fn parse_element(elem_value: &serde_json::Value) -> Option<LayoutElement> {
    let id = elem_value
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("elem_")
        .to_string();
    let type_str = elem_value
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let el_type = match type_str.to_lowercase().as_str() {
        "group" | "g" => ElementType::Group,
        "shape" | "rect" => ElementType::Shape,
        "circle" => ElementType::Circle,
        "ellipse" => ElementType::Ellipse,
        "path" => ElementType::Path,
        "text" => ElementType::Text,
        "image" | "img" => ElementType::Image,
        _ => ElementType::Unknown,
    };
    let x = elem_value.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
    let y = elem_value.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
    let w = elem_value
        .get("w")
        .or_else(|| elem_value.get("width"))
        .and_then(|v| v.as_f64())
        .unwrap_or(100.0) as f32;
    let h = elem_value
        .get("h")
        .or_else(|| elem_value.get("height"))
        .and_then(|v| v.as_f64())
        .unwrap_or(100.0) as f32;
    let text = elem_value
        .get("text")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let text_size = elem_value
        .get("textStyle")
        .and_then(|ts| ts.get("fontSize"))
        .and_then(|v| v.as_f64())
        .map(|f| f as f32)
        .or_else(|| {
            elem_value
                .get("textStyle")
                .and_then(|ts| ts.get("font-size"))
                .and_then(|v| v.as_f64())
                .map(|f| f as f32)
        });
    let fill = elem_value
        .get("fill")
        .and_then(|v| v.as_str())
        .and_then(crate::svg::parse_svg_color);
    let stroke_width = elem_value
        .get("strokeWidth")
        .or_else(|| elem_value.get("stroke-width"))
        .and_then(|v| v.as_f64())
        .map(|f| f as f32);
    let stroke_color = elem_value
        .get("stroke")
        .and_then(|v| v.as_str())
        .and_then(crate::svg::parse_svg_color);
    let stroke = stroke_width.and_then(|w| stroke_color.map(|c| (w, c)));
    let opacity = elem_value
        .get("opacity")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(1.0);
    let rotation_deg = elem_value
        .get("rotation")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(0.0);
    let corner_radius = elem_value
        .get("cornerRadius")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(0.0);
    let stroke_dash = elem_value
        .get("strokeDash")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_f64())
                .map(|f| f as f32)
                .collect()
        });
    let clip_children = elem_value
        .get("clipChildren")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let text_align = elem_value
        .get("textAlign")
        .and_then(|v| v.as_str())
        .map(|s| match s {
            "left" => TextAlign::Left,
            "center" => TextAlign::Center,
            "right" => TextAlign::Right,
            "justified" => TextAlign::Justified,
            _ => TextAlign::Left,
        });
    let letter_spacing = elem_value
        .get("letterSpacing")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32);
    let line_height = elem_value
        .get("lineHeight")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32);
    let blend_mode = elem_value
        .get("blendMode")
        .and_then(|v| v.as_str())
        .unwrap_or("normal")
        .parse::<BlendMode>()
        .unwrap_or(BlendMode::Normal);
    let stroke_cap = elem_value
        .get("strokeCap")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<StrokeCap>().ok());
    let stroke_join = elem_value
        .get("strokeJoin")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<StrokeJoin>().ok());
    let stroke_miter_limit = elem_value
        .get("strokeMiterLimit")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32);
    let text_decoration = elem_value
        .get("textDecoration")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<TextDecoration>().ok());
    let text_transform = elem_value
        .get("textTransform")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<TextTransform>().ok());
    let symbol_name = elem_value
        .get("symbolName")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let is_compound_path = elem_value
        .get("isCompoundPath")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let is_gradient_mesh = elem_value
        .get("isGradientMesh")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let is_chart = elem_value
        .get("isChart")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let is_opaque = elem_value
        .get("isOpaque")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let text_runs = parse_sidecar_text_runs(elem_value);
    let third_party_effects = parse_sidecar_third_party_effects(elem_value);
    let notes = parse_sidecar_notes(elem_value);
    let appearance_fills = parse_sidecar_appearance_fills(elem_value);
    let appearance_strokes = parse_sidecar_appearance_strokes(elem_value);
    let gradient = elem_value.get("gradient").and_then(parse_sidecar_gradient);
    let effects: Vec<EffectDef> = elem_value
        .get("effects")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(parse_sidecar_effect).collect())
        .unwrap_or_default();
    let children = if el_type == ElementType::Group {
        elem_value
            .get("children")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(parse_element).collect())
            .unwrap_or_default()
    } else {
        vec![]
    };
    let path_closed = elem_value
        .get("pathClosed")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let path_points = parse_sidecar_path_points(elem_value);
    let appearance_stack = if let Some(stack) =
        elem_value.get("appearanceStack").and_then(|v| v.as_array())
    {
        let mut entries = Vec::new();
        for entry in stack {
            if let Some(eo) = entry.as_object() {
                let entry_type = eo
                    .get("entryType")
                    .or_else(|| eo.get("kind"))
                    .or_else(|| eo.get("type"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if entry_type == "fill" {
                    let paint = if let Some(pattern) =
                        eo.get("gradient").and_then(parse_sidecar_pattern)
                    {
                        crate::scene::PaintSource::Pattern(pattern)
                    } else if let Some(pattern) = eo.get("pattern").and_then(parse_sidecar_pattern)
                    {
                        crate::scene::PaintSource::Pattern(pattern)
                    } else if let Some(gradient) =
                        eo.get("gradient").and_then(parse_sidecar_gradient)
                    {
                        if gradient.gradient_type == GradientType::Radial {
                            crate::scene::PaintSource::RadialGradient(gradient)
                        } else {
                            crate::scene::PaintSource::LinearGradient(gradient)
                        }
                    } else {
                        crate::scene::PaintSource::Solid(
                            parse_sidecar_color(eo).unwrap_or(Color32::BLACK),
                        )
                    };
                    entries.push(crate::scene::AppearanceEntry::Fill(
                        crate::scene::FillLayer {
                            paint,
                            opacity: eo.get("opacity").and_then(|v| v.as_f64()).unwrap_or(1.0)
                                as f32,
                            blend_mode: eo
                                .get("blendMode")
                                .and_then(|v| v.as_str())
                                .unwrap_or("normal")
                                .parse()
                                .unwrap_or(BlendMode::Normal),
                        },
                    ));
                } else if entry_type == "stroke" {
                    let paint = if let Some(pattern) =
                        eo.get("gradient").and_then(parse_sidecar_pattern)
                    {
                        crate::scene::PaintSource::Pattern(pattern)
                    } else if let Some(pattern) = eo.get("pattern").and_then(parse_sidecar_pattern)
                    {
                        crate::scene::PaintSource::Pattern(pattern)
                    } else if let Some(gradient) =
                        eo.get("gradient").and_then(parse_sidecar_gradient)
                    {
                        if gradient.gradient_type == GradientType::Radial {
                            crate::scene::PaintSource::RadialGradient(gradient)
                        } else {
                            crate::scene::PaintSource::LinearGradient(gradient)
                        }
                    } else {
                        crate::scene::PaintSource::Solid(
                            parse_sidecar_color(eo).unwrap_or(Color32::BLACK),
                        )
                    };
                    entries.push(crate::scene::AppearanceEntry::Stroke(
                        crate::scene::StrokeLayer {
                            paint,
                            width: eo.get("width").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                            opacity: eo.get("opacity").and_then(|v| v.as_f64()).unwrap_or(1.0)
                                as f32,
                            blend_mode: eo
                                .get("blendMode")
                                .and_then(|v| v.as_str())
                                .unwrap_or("normal")
                                .parse()
                                .unwrap_or(BlendMode::Normal),
                            cap: eo
                                .get("cap")
                                .and_then(|v| v.as_str())
                                .and_then(|s| s.parse().ok()),
                            join: eo
                                .get("join")
                                .and_then(|v| v.as_str())
                                .and_then(|s| s.parse().ok()),
                            dash: eo
                                .get("dash")
                                .or_else(|| eo.get("strokeDash"))
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_f64())
                                        .map(|v| v as f32)
                                        .collect()
                                }),
                            miter_limit: eo
                                .get("miterLimit")
                                .or_else(|| eo.get("miter_limit"))
                                .and_then(|v| v.as_f64())
                                .map(|v| v as f32),
                        },
                    ));
                } else if entry_type == "effect"
                    || matches!(
                        entry_type,
                        "dropShadow"
                            | "drop-shadow"
                            | "innerShadow"
                            | "inner-shadow"
                            | "outerGlow"
                            | "outer-glow"
                            | "innerGlow"
                            | "inner-glow"
                            | "gaussianBlur"
                            | "gaussian-blur"
                            | "bevel"
                            | "feather"
                            | "noise"
                            | "grain"
                            | "liveEffect"
                            | "live-effect"
                    )
                {
                    if let Some(effect_def) = parse_sidecar_effect(entry) {
                        entries.push(crate::scene::AppearanceEntry::Effect(
                            crate::scene::EffectLayer {
                                effect_type: effect_def.effect_type.clone(),
                                params: effect_def.clone(),
                                opacity: eo.get("opacity").and_then(|v| v.as_f64()).unwrap_or(1.0)
                                    as f32,
                                blend_mode: effect_def.blend_mode,
                            },
                        ));
                    }
                }
            }
        }
        crate::scene::AppearanceStack { entries }
    } else {
        crate::scene::AppearanceStack::default()
    };
    let appearance_stack = if appearance_stack.is_empty() {
        let pattern_appearance_fills = elem_value
            .get("appearanceFills")
            .and_then(|v| v.as_array())
            .filter(|fills| {
                fills.iter().any(|fill| {
                    fill.get("gradient")
                        .or_else(|| fill.get("pattern"))
                        .and_then(parse_sidecar_pattern)
                        .is_some()
                })
            });
        if let Some(fills) = pattern_appearance_fills {
            let mut entries = Vec::new();
            for fill in fills {
                let Some(fo) = fill.as_object() else {
                    continue;
                };
                let paint = if let Some(pattern) = fo
                    .get("gradient")
                    .or_else(|| fo.get("pattern"))
                    .and_then(parse_sidecar_pattern)
                {
                    crate::scene::PaintSource::Pattern(pattern)
                } else if let Some(gradient) = fo.get("gradient").and_then(parse_sidecar_gradient) {
                    if gradient.gradient_type == GradientType::Radial {
                        crate::scene::PaintSource::RadialGradient(gradient)
                    } else {
                        crate::scene::PaintSource::LinearGradient(gradient)
                    }
                } else {
                    crate::scene::PaintSource::Solid(
                        parse_sidecar_color(fo).unwrap_or(Color32::BLACK),
                    )
                };
                entries.push(crate::scene::AppearanceEntry::Fill(
                    crate::scene::FillLayer {
                        paint,
                        opacity: fo.get("opacity").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                        blend_mode: fo
                            .get("blendMode")
                            .and_then(|v| v.as_str())
                            .unwrap_or("normal")
                            .parse()
                            .unwrap_or(BlendMode::Normal),
                    },
                ));
            }
            for stroke in &appearance_strokes {
                let paint = if let Some(gradient) = &stroke.gradient {
                    if gradient.gradient_type == GradientType::Radial {
                        crate::scene::PaintSource::RadialGradient(gradient.clone())
                    } else {
                        crate::scene::PaintSource::LinearGradient(gradient.clone())
                    }
                } else if let Some(pattern) = &stroke.pattern {
                    crate::scene::PaintSource::Pattern(pattern.clone())
                } else {
                    crate::scene::PaintSource::Solid(stroke.color)
                };
                entries.push(crate::scene::AppearanceEntry::Stroke(
                    crate::scene::StrokeLayer {
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
            for effect in &effects {
                entries.push(crate::scene::AppearanceEntry::Effect(
                    crate::scene::EffectLayer {
                        effect_type: effect.effect_type.clone(),
                        params: effect.clone(),
                        opacity: 1.0,
                        blend_mode: effect.blend_mode.clone(),
                    },
                ));
            }
            crate::scene::AppearanceStack { entries }
        } else if let Some(pattern) = elem_value
            .get("gradient")
            .or_else(|| elem_value.get("pattern"))
            .and_then(parse_sidecar_pattern)
        {
            let mut entries = vec![crate::scene::AppearanceEntry::Fill(
                crate::scene::FillLayer {
                    paint: crate::scene::PaintSource::Pattern(pattern),
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                },
            )];
            if let Some((width, color)) = stroke {
                entries.push(crate::scene::AppearanceEntry::Stroke(
                    crate::scene::StrokeLayer {
                        paint: crate::scene::PaintSource::Solid(color),
                        width,
                        opacity: 1.0,
                        blend_mode: BlendMode::Normal,
                        cap: stroke_cap.clone(),
                        join: stroke_join.clone(),
                        dash: stroke_dash.clone(),
                        miter_limit: stroke_miter_limit,
                    },
                ));
            }
            for effect in &effects {
                entries.push(crate::scene::AppearanceEntry::Effect(
                    crate::scene::EffectLayer {
                        effect_type: effect.effect_type.clone(),
                        params: effect.clone(),
                        opacity: 1.0,
                        blend_mode: effect.blend_mode.clone(),
                    },
                ));
            }
            crate::scene::AppearanceStack { entries }
        } else {
            appearance_stack
        }
    } else {
        appearance_stack
    };
    let image_path = elem_value
        .get("imagePath")
        .or_else(|| elem_value.get("image_path"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    Some(LayoutElement {
        id,
        el_type,
        x,
        y,
        w,
        h,
        fill,
        stroke,
        text,
        text_size,
        children,
        opacity,
        rotation_deg,
        corner_radius,
        gradient,
        blend_mode,
        effects,
        stroke_dash,
        clip_children,
        text_align,
        letter_spacing,
        line_height,
        stroke_cap,
        stroke_join,
        stroke_miter_limit,
        text_decoration,
        text_transform,
        text_runs,
        symbol_name,
        is_compound_path,
        is_gradient_mesh,
        is_chart,
        is_opaque,
        third_party_effects,
        notes,
        appearance_fills,
        appearance_strokes,
        appearance_stack,
        path_points,
        path_closed,
        artboard_name: None,
        image_path,
    })
}
