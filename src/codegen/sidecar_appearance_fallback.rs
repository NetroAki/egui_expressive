use super::*;

pub fn fallback_appearance_stack(
    elem_value: &serde_json::Value,
    appearance_strokes: &[AppearanceStroke],
    effects: &[EffectDef],
    stroke_alignment: crate::scene::StrokeAlignment,
    stroke: Option<(f32, Color32)>,
    fill: Option<Color32>,
    gradient: &Option<GradientDef>,
    stroke_cap: &Option<StrokeCap>,
    stroke_join: &Option<StrokeJoin>,
    stroke_dash: &Option<Vec<f32>>,
    stroke_miter_limit: Option<f32>,
    appearance_stack: crate::scene::AppearanceStack,
) -> crate::scene::AppearanceStack {
    let appearance_stack = if appearance_stack.is_empty() {
        let pattern_appearance_fills = elem_value
            .get("appearanceFills")
            .and_then(|v| v.as_array())
            .filter(|fills| {
                fills.iter().any(|fill| {
                    fill.get("gradient")
                        .or_else(|| fill.get("pattern"))
                        .and_then(parse_pattern)
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
                    .and_then(parse_pattern)
                {
                    crate::scene::PaintSource::Pattern(pattern)
                } else if let Some(gradient) = fo.get("gradient").and_then(parse_gradient) {
                    if gradient.gradient_type == GradientType::Radial {
                        crate::scene::PaintSource::RadialGradient(gradient)
                    } else {
                        crate::scene::PaintSource::LinearGradient(gradient)
                    }
                } else {
                    crate::scene::PaintSource::Solid(
                        parse_color_value(&serde_json::Value::Object(fo.clone()))
                            .unwrap_or(Color32::BLACK),
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
                        alignment: stroke.alignment,
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
            .and_then(parse_pattern)
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
                        alignment: stroke_alignment,
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
        } else if stroke.is_some() && stroke_alignment != crate::scene::StrokeAlignment::Center {
            let mut entries = Vec::new();
            if let Some(fill_color) = fill {
                let paint = if let Some(gradient) = &gradient {
                    if gradient.gradient_type == GradientType::Radial {
                        crate::scene::PaintSource::RadialGradient(gradient.clone())
                    } else {
                        crate::scene::PaintSource::LinearGradient(gradient.clone())
                    }
                } else {
                    crate::scene::PaintSource::Solid(fill_color)
                };
                entries.push(crate::scene::AppearanceEntry::Fill(
                    crate::scene::FillLayer {
                        paint,
                        opacity: 1.0,
                        blend_mode: BlendMode::Normal,
                    },
                ));
            }
            if let Some((width, color)) = stroke {
                entries.push(crate::scene::AppearanceEntry::Stroke(
                    crate::scene::StrokeLayer {
                        paint: crate::scene::PaintSource::Solid(color),
                        width,
                        alignment: stroke_alignment,
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
}
