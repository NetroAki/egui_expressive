use super::*;

pub(crate) fn generate_scene_node_code(node: &crate::scene::SceneNode, indent: usize) -> String {
    let ind = " ".repeat(indent);
    let mut out = String::new();
    out.push_str("egui_expressive::scene::SceneNode {\n");
    out.push_str(&format!(
        "{}    id: \"{}\".to_string(),\n",
        ind,
        node.id.replace('"', "\\\"")
    ));

    // Geometry
    out.push_str(&format!("{}    geometry: ", ind));
    match &node.geometry {
        crate::scene::Geometry::Group { bounds } => {
            out.push_str(&format!("egui_expressive::scene::Geometry::Group {{ bounds: egui::Rect::from_min_max(egui::pos2({:.1}, {:.1}), egui::pos2({:.1}, {:.1})) }},\n", bounds.min.x, bounds.min.y, bounds.max.x, bounds.max.y));
        }
        crate::scene::Geometry::Rect {
            rect,
            corner_radius,
        } => {
            out.push_str(&format!("egui_expressive::scene::Geometry::Rect {{ rect: egui::Rect::from_min_max(egui::pos2({:.1}, {:.1}), egui::pos2({:.1}, {:.1})), corner_radius: {:.1} }},\n", rect.min.x, rect.min.y, rect.max.x, rect.max.y, corner_radius));
        }
        crate::scene::Geometry::Ellipse { rect } => {
            out.push_str(&format!("egui_expressive::scene::Geometry::Ellipse {{ rect: egui::Rect::from_min_max(egui::pos2({:.1}, {:.1}), egui::pos2({:.1}, {:.1})) }},\n", rect.min.x, rect.min.y, rect.max.x, rect.max.y));
        }
        crate::scene::Geometry::Path { points, closed } => {
            out.push_str("egui_expressive::scene::Geometry::Path {\n");
            out.push_str(&format!(
                "{}        points: egui_expressive::scene::path_points(&[\n",
                ind
            ));
            for p in points {
                out.push_str(&format!("{}            ({:.1}, {:.1}),\n", ind, p.x, p.y));
            }
            out.push_str(&format!(
                "{}        ]),\n{}        closed: {},\n{}    }},\n",
                ind, ind, closed, ind
            ));
        }
        crate::scene::Geometry::MeshPatch {
            corners,
            colors,
            subdivisions,
        } => {
            out.push_str("egui_expressive::scene::Geometry::MeshPatch { corners: [");
            for p in corners {
                out.push_str(&format!("egui::pos2({:.1}, {:.1}), ", p.x, p.y));
            }
            out.push_str("], colors: [");
            for c in colors {
                out.push_str(&format!(
                    "egui::Color32::from_rgba_unmultiplied({}, {}, {}, {}), ",
                    c.r(),
                    c.g(),
                    c.b(),
                    c.a()
                ));
            }
            out.push_str(&format!("], subdivisions: {} }},\n", subdivisions));
        }
    }

    // Appearance
    out.push_str(&format!(
        "{}    appearance: egui_expressive::scene::AppearanceStack {{\n",
        ind
    ));
    out.push_str(&format!("{}        entries: vec![\n", ind));
    for entry in &node.appearance.entries {
        match entry {
            crate::scene::AppearanceEntry::Fill(fill) => {
                out.push_str(&format!("{}            egui_expressive::scene::AppearanceEntry::Fill(egui_expressive::scene::FillLayer {{\n", ind));
                out.push_str(&format!(
                    "{}                paint: {},\n",
                    ind,
                    generate_paint_source_code(&fill.paint)
                ));
                out.push_str(&format!(
                    "{}                opacity: {:.2},\n",
                    ind, fill.opacity
                ));
                out.push_str(&format!(
                    "{}                blend_mode: egui_expressive::codegen::BlendMode::{:?},\n",
                    ind, fill.blend_mode
                ));
                out.push_str(&format!("{}            }}),\n", ind));
            }
            crate::scene::AppearanceEntry::Stroke(stroke) => {
                out.push_str(&format!("{}            egui_expressive::scene::AppearanceEntry::Stroke(egui_expressive::scene::StrokeLayer {{\n", ind));
                out.push_str(&format!(
                    "{}                paint: {},\n",
                    ind,
                    generate_paint_source_code(&stroke.paint)
                ));
                out.push_str(&format!(
                    "{}                width: {:.1},\n",
                    ind, stroke.width
                ));
                out.push_str(&format!(
                    "{}                opacity: {:.2},\n",
                    ind, stroke.opacity
                ));
                out.push_str(&format!(
                    "{}                blend_mode: egui_expressive::codegen::BlendMode::{:?},\n",
                    ind, stroke.blend_mode
                ));
                if let Some(dash) = &stroke.dash {
                    out.push_str(&format!(
                        "{}                dash: Some(vec![{}]),\n",
                        ind,
                        dash.iter()
                            .map(|d| format!("{:.1}", d))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                } else {
                    out.push_str(&format!("{}                dash: None,\n", ind));
                }
                if let Some(cap) = &stroke.cap {
                    out.push_str(&format!(
                        "{}                cap: Some(egui_expressive::codegen::StrokeCap::{:?}),\n",
                        ind, cap
                    ));
                } else {
                    out.push_str(&format!("{}                cap: None,\n", ind));
                }
                if let Some(join) = &stroke.join {
                    out.push_str(&format!(
                        "{}                join: Some(egui_expressive::codegen::StrokeJoin::{:?}),\n",
                        ind, join
                    ));
                } else {
                    out.push_str(&format!("{}                join: None,\n", ind));
                }
                if let Some(miter_limit) = stroke.miter_limit {
                    out.push_str(&format!(
                        "{}                miter_limit: Some({:.1}),\n",
                        ind, miter_limit
                    ));
                } else {
                    out.push_str(&format!("{}                miter_limit: None,\n", ind));
                }
                out.push_str(&format!("{}            }}),\n", ind));
            }
            crate::scene::AppearanceEntry::Effect(effect) => {
                out.push_str(&format!("{}            egui_expressive::scene::AppearanceEntry::Effect(egui_expressive::scene::EffectLayer {{\n", ind));
                out.push_str(&format!(
                    "{}                effect_type: egui_expressive::codegen::EffectType::{:?},\n",
                    ind, effect.effect_type
                ));
                out.push_str(&format!(
                    "{}                params: egui_expressive::codegen::EffectDef {{\n",
                    ind
                ));
                out.push_str(&format!("{}                    effect_type: egui_expressive::codegen::EffectType::{:?},\n", ind, effect.params.effect_type));
                out.push_str(&format!(
                    "{}                    x: {:.1}, y: {:.1}, blur: {:.1}, spread: {:.1},\n",
                    ind, effect.params.x, effect.params.y, effect.params.blur, effect.params.spread
                ));
                out.push_str(&format!("{}                    color: egui::Color32::from_rgba_unmultiplied({}, {}, {}, {}),\n", ind, effect.params.color.r(), effect.params.color.g(), effect.params.color.b(), effect.params.color.a()));
                out.push_str(&format!("{}                    blend_mode: egui_expressive::codegen::BlendMode::{:?},\n", ind, effect.params.blend_mode));
                out.push_str(&format!("{}                    depth: {:.1}, angle: {:.1}, radius: {:.1}, amount: {:.1}, scale: {:.1}, seed: {},\n", ind, effect.params.depth, effect.params.angle, effect.params.radius, effect.params.amount, effect.params.scale, effect.params.seed));
                out.push_str(&format!(
                    "{}                    highlight: None, shadow_color: None,\n",
                    ind
                )); // Simplified
                out.push_str(&format!("{}                }},\n", ind));
                out.push_str(&format!(
                    "{}                opacity: {:.2},\n",
                    ind, effect.opacity
                ));
                out.push_str(&format!(
                    "{}                blend_mode: egui_expressive::codegen::BlendMode::{:?},\n",
                    ind, effect.blend_mode
                ));
                out.push_str(&format!("{}            }}),\n", ind));
            }
        }
    }
    out.push_str(&format!("{}        ],\n", ind));
    out.push_str(&format!("{}    }},\n", ind));

    out.push_str(&format!("{}    opacity: {:.2},\n", ind, node.opacity));
    out.push_str(&format!(
        "{}    blend_mode: egui_expressive::codegen::BlendMode::{:?},\n",
        ind, node.blend_mode
    ));
    out.push_str(&format!(
        "{}    rotation_deg: {:.4},\n",
        ind, node.rotation_deg
    ));
    out.push_str(&format!(
        "{}    clip_children: {},\n",
        ind, node.clip_children
    ));

    out.push_str(&format!("{}    children: vec![\n", ind));
    for child in &node.children {
        out.push_str(&format!(
            "{}        {},\n",
            ind,
            generate_scene_node_code(child, indent + 8)
        ));
    }
    out.push_str(&format!("{}    ],\n", ind));

    out.push_str(&format!("{}}}", ind));
    out
}

pub(crate) fn generate_paint_source_code(paint: &crate::scene::PaintSource) -> String {
    fn opt_point_expr(point: Option<[f32; 2]>) -> String {
        point
            .map(|p| format!("Some([{:.1}, {:.1}])", p[0], p[1]))
            .unwrap_or_else(|| "None".to_string())
    }

    fn opt_f32_expr(value: Option<f32>) -> String {
        value
            .map(|v| format!("Some({:.1})", v))
            .unwrap_or_else(|| "None".to_string())
    }

    fn opt_transform_expr(value: Option<[f32; 6]>) -> String {
        value
            .map(|m| {
                format!(
                    "Some([{:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}])",
                    m[0], m[1], m[2], m[3], m[4], m[5]
                )
            })
            .unwrap_or_else(|| "None".to_string())
    }

    match paint {
        crate::scene::PaintSource::Solid(c) => format!("egui_expressive::scene::PaintSource::Solid(egui::Color32::from_rgba_unmultiplied({}, {}, {}, {}))", c.r(), c.g(), c.b(), c.a()),
        crate::scene::PaintSource::LinearGradient(g) => {
            let mut stops = String::new();
            for s in &g.stops {
                stops.push_str(&format!("egui_expressive::codegen::GradientStop {{ position: {:.2}, color: egui::Color32::from_rgba_unmultiplied({}, {}, {}, {}) }}, ", s.position, s.color.r(), s.color.g(), s.color.b(), s.color.a()));
            }
            format!("egui_expressive::scene::PaintSource::LinearGradient(egui_expressive::codegen::GradientDef {{ gradient_type: egui_expressive::codegen::GradientType::Linear, angle_deg: {:.1}, center: {}, focal_point: {}, radius: {}, transform: {}, stops: vec![{}] }})", g.angle_deg, opt_point_expr(g.center), opt_point_expr(g.focal_point), opt_f32_expr(g.radius), opt_transform_expr(g.transform), stops)
        }
        crate::scene::PaintSource::RadialGradient(g) => {
            let mut stops = String::new();
            for s in &g.stops {
                stops.push_str(&format!("egui_expressive::codegen::GradientStop {{ position: {:.2}, color: egui::Color32::from_rgba_unmultiplied({}, {}, {}, {}) }}, ", s.position, s.color.r(), s.color.g(), s.color.b(), s.color.a()));
            }
            format!("egui_expressive::scene::PaintSource::RadialGradient(egui_expressive::codegen::GradientDef {{ gradient_type: egui_expressive::codegen::GradientType::Radial, angle_deg: {:.1}, center: {}, focal_point: {}, radius: {}, transform: {}, stops: vec![{}] }})", g.angle_deg, opt_point_expr(g.center), opt_point_expr(g.focal_point), opt_f32_expr(g.radius), opt_transform_expr(g.transform), stops)
        }
        crate::scene::PaintSource::Pattern(p) => {
            format!(
                "egui_expressive::scene::PaintSource::Pattern(egui_expressive::scene::PatternDef {{ name: {:?}.to_string(), seed: {}, foreground: egui::Color32::from_rgba_unmultiplied({}, {}, {}, {}), background: egui::Color32::from_rgba_unmultiplied({}, {}, {}, {}), cell_size: {:.1}, mark_size: {:.1} }})",
                p.name,
                p.seed,
                p.foreground.r(),
                p.foreground.g(),
                p.foreground.b(),
                p.foreground.a(),
                p.background.r(),
                p.background.g(),
                p.background.b(),
                p.background.a(),
                p.cell_size,
                p.mark_size
            )
        }
        crate::scene::PaintSource::MeshGradient { corners, colors, subdivisions } => {
            let mut c_str = String::new();
            for c in corners { c_str.push_str(&format!("egui::pos2({:.1}, {:.1}), ", c.x, c.y)); }
            let mut col_str = String::new();
            for c in colors { col_str.push_str(&format!("egui::Color32::from_rgba_unmultiplied({}, {}, {}, {}), ", c.r(), c.g(), c.b(), c.a())); }
            format!("egui_expressive::scene::PaintSource::MeshGradient {{ corners: [{}], colors: [{}], subdivisions: {} }}", c_str, col_str, subdivisions)
        }
        crate::scene::PaintSource::ProceduralNoise(n) => {
            format!("egui_expressive::scene::PaintSource::ProceduralNoise(egui_expressive::scene::NoiseDef {{ seed: {}, cell_size: {:.1}, opacity: {:.2} }})", n.seed, n.cell_size, n.opacity)
        }
    }
}

/// Convert a Color32 to either a token reference or a literal
pub(crate) fn color_to_token_or_literal(
    color: &Color32,
    token_map: Option<&HashMap<String, Color32>>,
) -> String {
    if let Some(map) = token_map {
        // Look up the color in the token map — sort keys for deterministic output
        let mut entries: Vec<(&String, &Color32)> = map.iter().collect();
        entries.sort_by_key(|(name, _)| name.as_str());
        for (name, c) in entries {
            if *c == *color {
                return format!("tokens::{}", name.to_uppercase());
            }
        }
    }
    // Fall back to literal — use to_srgba_unmultiplied() to get straight-alpha bytes
    // (Color32 stores premultiplied; feeding .r()/.g()/.b() to from_rgba_unmultiplied would double-premultiply)
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    if a < 255 {
        format!(
            "egui::Color32::from_rgba_unmultiplied({}, {}, {}, {})",
            r, g, b, a
        )
    } else {
        format!("egui::Color32::from_rgb({}, {}, {})", r, g, b)
    }
}
