use super::*;
pub fn parse_appearance_stack(
    elem_value: &serde_json::Value,
    appearance_strokes: &[AppearanceStroke],
    effects: &[EffectDef],
    stroke_alignment: crate::scene::StrokeAlignment,
    stroke: Option<(f32, Color32)>,
    fill: Option<Color32>,
    gradient: &Option<GradientDef>,
    stroke_cap: &Option<StrokeCap>,
    stroke_join: &Option<StrokeJoin>,
    stroke_miter_limit: Option<f32>,
) -> crate::scene::AppearanceStack {
    let appearance_fills: Vec<AppearanceFill> =
        if let Some(fills) = elem_value.get("appearanceFills").and_then(|v| v.as_array()) {
            fills
                .iter()
                .filter_map(|f| {
                    let fo = f.as_object()?;
                    Some(AppearanceFill {
                        color: parse_color_value(&serde_json::Value::Object(fo.clone()))
                            .unwrap_or(Color32::BLACK),
                        gradient: fo.get("gradient").and_then(parse_gradient),
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
        } else {
            vec![]
        };
    let appearance_strokes: Vec<AppearanceStroke> = if let Some(strokes) = elem_value
        .get("appearanceStrokes")
        .and_then(|v| v.as_array())
    {
        strokes
            .iter()
            .filter_map(|s| {
                let so = s.as_object()?;
                Some(AppearanceStroke {
                    color: parse_color_value(&serde_json::Value::Object(so.clone()))
                        .unwrap_or(Color32::BLACK),
                    gradient: so.get("gradient").and_then(parse_gradient),
                    pattern: so
                        .get("gradient")
                        .and_then(parse_pattern)
                        .or_else(|| so.get("pattern").and_then(parse_pattern)),
                    width: so.get("width").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                    alignment: so
                        .get("alignment")
                        .or_else(|| so.get("strokeAlignment"))
                        .or_else(|| so.get("strokeAlign"))
                        .or_else(|| so.get("stroke_alignment"))
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<crate::scene::StrokeAlignment>().ok())
                        .unwrap_or_default(),
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
    } else {
        vec![]
    };
    let gradient = elem_value.get("gradient").and_then(parse_gradient);
    let parse_effect = |e: &serde_json::Value| -> Option<EffectDef> {
        let effect_type_str = e
            .get("effect_type")
            .or_else(|| e.get("effectType"))
            .or_else(|| e.get("type"))?
            .as_str()?;
        let e_obj = e.as_object()?;
        let effect_type = match effect_type_str {
            "dropShadow" | "drop-shadow" => EffectType::DropShadow,
            "innerShadow" | "inner-shadow" => EffectType::InnerShadow,
            "outerGlow" | "outer-glow" => EffectType::OuterGlow,
            "innerGlow" | "inner-glow" => EffectType::InnerGlow,
            "gaussianBlur" | "gaussian-blur" => EffectType::GaussianBlur,
            "bevel" => EffectType::Bevel,
            "feather" => EffectType::Feather,
            "noise" | "grain" => EffectType::Noise,
            "liveEffect" | "live-effect" => EffectType::Unknown(
                e.get("name")
                    .or_else(|| e.get("effectName"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(effect_type_str)
                    .to_string(),
            ),
            _ => EffectType::Unknown(effect_type_str.to_string()),
        };
        let x = e
            .get("x")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(0.0);
        let y = e
            .get("y")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(0.0);
        let blur = e
            .get("blur")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(0.0);
        let spread = e
            .get("spread")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(0.0);
        let color = parse_color_value(&serde_json::Value::Object(e_obj.clone()))
            .unwrap_or(egui::Color32::BLACK);
        let blend_mode = e
            .get("blendMode")
            .and_then(|v| v.as_str())
            .unwrap_or("normal")
            .parse::<BlendMode>()
            .unwrap_or(BlendMode::Normal);
        let depth = e
            .get("depth")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(0.0);
        let angle = e
            .get("angle")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(0.0);
        let highlight = e.get("highlight").and_then(&parse_color_value);
        let shadow_color = e
            .get("shadowColor")
            .or_else(|| e.get("shadow"))
            .or_else(|| e.get("shadow_color"))
            .and_then(&parse_color_value);
        let radius = e
            .get("radius")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(0.0);
        let amount = e
            .get("amount")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(0.0);
        let scale = e
            .get("scale")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(2.0);
        let seed = e
            .get("seed")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(0);
        Some(EffectDef {
            effect_type,
            x,
            y,
            blur,
            spread,
            color,
            blend_mode,
            depth,
            angle,
            highlight,
            shadow_color,
            radius,
            amount,
            scale,
            seed,
        })
    };
    let effects: Vec<EffectDef> = elem_value
        .get("effects")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(parse_effect).collect())
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
    let parse_path_points = |pts: &[serde_json::Value]| -> Vec<PathPoint> {
        pts.iter()
            .filter_map(|p| {
                let po = p.as_object()?;
                let anchor = po.get("anchor").and_then(|v| v.as_array())?;
                let left_ctrl = po
                    .get("left_ctrl")
                    .or_else(|| po.get("leftDir"))
                    .or_else(|| po.get("leftCtrl"))
                    .and_then(|v| v.as_array())
                    .unwrap_or(anchor);
                let right_ctrl = po
                    .get("right_ctrl")
                    .or_else(|| po.get("rightDir"))
                    .or_else(|| po.get("rightCtrl"))
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
    };
    let path_points: Vec<PathPoint> = elem_value
        .get("pathPoints")
        .and_then(|v| v.as_array())
        .map(|pts| parse_path_points(pts))
        .unwrap_or_default();
    let fill_rule = if let Some(raw) = elem_value
        .get("fillRule")
        .or_else(|| elem_value.get("fill_rule"))
        .and_then(|v| v.as_str())
    {
        match raw.to_ascii_lowercase().as_str() {
            "evenodd" | "even-odd" | "even_odd" => crate::scene::FillRule::EvenOdd,
            "nonzero" | "non-zero" | "non_zero" => crate::scene::FillRule::NonZero,
            _other => {
                // Invalid fill rule: skip element so Option-returning parser stays consistent.
                return None;
            }
        }
    } else if is_compound_path {
        // Compound paths must carry an explicit fill rule.
        return None;
    } else {
        crate::scene::FillRule::NonZero
    };
    let subpaths: Vec<crate::scene::PathContour> = elem_value
        .get("subpaths")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|subpath| {
                    let so = subpath.as_object()?;
                    let raw_points = so.get("points").and_then(|v| v.as_array())?;
                    let points = parse_path_points(raw_points);
                    if points.is_empty() {
                        return None;
                    }
                    let closed = so.get("closed").and_then(|v| v.as_bool()).unwrap_or(true);
                    Some(crate::scene::PathContour {
                        points: crate::scene::sample_layout_path(&points, closed),
                        closed,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
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
                    let paint = if let Some(pattern) = eo.get("gradient").and_then(parse_pattern) {
                        crate::scene::PaintSource::Pattern(pattern)
                    } else if let Some(pattern) = eo.get("pattern").and_then(parse_pattern) {
                        crate::scene::PaintSource::Pattern(pattern)
                    } else if let Some(gradient) = eo.get("gradient").and_then(parse_gradient) {
                        if gradient.gradient_type == GradientType::Radial {
                            crate::scene::PaintSource::RadialGradient(gradient)
                        } else {
                            crate::scene::PaintSource::LinearGradient(gradient)
                        }
                    } else {
                        crate::scene::PaintSource::Solid(
                            parse_color_value(&serde_json::Value::Object(eo.clone()))
                                .unwrap_or(Color32::BLACK),
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
                    let paint = if let Some(pattern) = eo.get("gradient").and_then(parse_pattern) {
                        crate::scene::PaintSource::Pattern(pattern)
                    } else if let Some(pattern) = eo.get("pattern").and_then(parse_pattern) {
                        crate::scene::PaintSource::Pattern(pattern)
                    } else if let Some(gradient) = eo.get("gradient").and_then(parse_gradient) {
                        if gradient.gradient_type == GradientType::Radial {
                            crate::scene::PaintSource::RadialGradient(gradient)
                        } else {
                            crate::scene::PaintSource::LinearGradient(gradient)
                        }
                    } else {
                        crate::scene::PaintSource::Solid(
                            parse_color_value(&serde_json::Value::Object(eo.clone()))
                                .unwrap_or(Color32::BLACK),
                        )
                    };
                    entries.push(crate::scene::AppearanceEntry::Stroke(
                        crate::scene::StrokeLayer {
                            paint,
                            width: eo.get("width").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                            alignment: eo
                                .get("alignment")
                                .or_else(|| eo.get("strokeAlignment"))
                                .or_else(|| eo.get("strokeAlign"))
                                .or_else(|| eo.get("stroke_alignment"))
                                .and_then(|v| v.as_str())
                                .and_then(|s| s.parse::<crate::scene::StrokeAlignment>().ok())
                                .unwrap_or_default(),
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
                    if let Some(effect_def) = parse_effect(entry) {
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
        fallback_appearance_stack(
            elem_value,
            &appearance_strokes,
            &effects,
            stroke_alignment,
            stroke,
            fill,
            &gradient,
            &stroke_cap,
            &stroke_join,
            &stroke_dash,
            stroke_miter_limit,
            appearance_stack,
        )
    } else {
        appearance_stack
    };
}
