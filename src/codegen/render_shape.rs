use super::*;

pub fn generate_shape_node(
    indent: usize,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    fill: Color32,
    id: &str,
    style: &VisualStyle,
    token_map: Option<&HashMap<String, Color32>>,
) -> String {
    let indent_str = " ".repeat(indent);
    let inner = " ".repeat(indent + 4);
    let mut output = String::new();
    output.push_str(&format!(
        "{}// Shape: {}\n{}{{\n",
        indent_str, id, indent_str
    ));
    let inner = " ".repeat(indent + 4);
    output.push_str(&format!(
                "{}let rect = egui::Rect::from_min_size(origin + egui::vec2({:.1}, {:.1}), egui::vec2({:.1}, {:.1}));\n",
                inner, x, y, w, h
            ));

    // Drop shadows (before shape) — scale shadow alpha by shape opacity
    // Use to_srgba_unmultiplied() to get straight-alpha bytes (Color32 stores premultiplied)
    for effect in &style.effects {
        let [sr, sg, sb, sa] = effect.color.to_srgba_unmultiplied();
        let shadow_a = (sa as f32 * style.opacity).clamp(0.0, 255.0) as u8;
        match effect.effect_type {
            EffectType::DropShadow => {
                output.push_str(&format!(
                            "{}for s in egui_expressive::box_shadow(rect, egui::Color32::from_rgba_unmultiplied({}, {}, {}, {}), {:.1}, {:.1}, egui_expressive::ShadowOffset::new({:.1}, {:.1})) {{ painter.add(s); }}\n",
                            inner,
                            sr, sg, sb, shadow_a,
                            effect.blur, effect.spread, effect.x, effect.y
                        ));
            }
            EffectType::OuterGlow => {
                output.push_str(&format!(
                            "{}for s in egui_expressive::soft_shadow(rect, egui::Color32::from_rgba_unmultiplied({}, {}, {}, {}), {:.1}, 0.0, egui_expressive::ShadowOffset::zero(), egui_expressive::BlurQuality::Medium) {{ painter.add(s); }}\n",
                            inner,
                            sr, sg, sb, shadow_a,
                            effect.blur
                        ));
            }
            _ => {}
        }
    }

    // Fill
    let fill_color = color_to_token_or_literal(fill, token_map);
    if style.opacity < 1.0 {
        output.push_str(&format!(
            "{}let fill = egui_expressive::with_alpha({}, {:.2});\n",
            inner, fill_color, style.opacity
        ));
    } else {
        output.push_str(&format!("{}let fill = {};\n", inner, fill_color));
    }

    // Stroke
    if let Some((width, color)) = style.stroke {
        let stroke_color = color_to_token_or_literal(&color, token_map);
        if style.opacity < 1.0 {
            output.push_str(&format!(
                        "{}let stroke = egui::Stroke::new({:.1}, egui_expressive::with_alpha({}, {:.2}));\n",
                        inner, width, stroke_color, style.opacity
                    ));
        } else {
            output.push_str(&format!(
                "{}let stroke = egui::Stroke::new({:.1}, {});\n",
                inner, width, stroke_color
            ));
        }
    } else {
        output.push_str(&format!("{}let stroke = egui::Stroke::NONE;\n", inner));
    }

    // Main shape: gradient or solid fill
    let has_rotation = style.rotation_deg.abs() > 0.001;
    if has_rotation && style.gradient.is_none() {
        output.push_str(&format!(
            "{}let _rot = egui_expressive::Transform2D::rotate_around({:.4}, rect.center());\n",
            inner, style.rotation_deg
        ));
        if style.corner_radius > 0.001 {
            output.push_str(&format!(
                        "{}let _rot_pts = egui_expressive::rounded_rect_path(rect, {:.1}).into_iter().map(|p| _rot.apply(p)).collect::<Vec<_>>();\n",
                        inner, style.corner_radius
                    ));
            output.push_str(&format!(
                "{}painter.add(egui::Shape::closed_line(_rot_pts.clone(), stroke));\n",
                inner
            ));
            output.push_str(&format!(
                "{}painter.add(egui::Shape::convex_polygon(_rot_pts, fill, egui::Stroke::NONE));\n",
                inner
            ));
        } else {
            output.push_str(&format!(
                        "{}let _rot_pts = vec![_rot.apply(rect.min), _rot.apply(egui::pos2(rect.max.x, rect.min.y)), _rot.apply(rect.max), _rot.apply(egui::pos2(rect.min.x, rect.max.y))];\n",
                        inner
                    ));
            output.push_str(&format!(
                "{}painter.add(egui::Shape::convex_polygon(_rot_pts, fill, stroke));\n",
                inner
            ));
        }
    } else if let Some(grad) = &style.gradient {
        if has_rotation {
            output.push_str(&format!(
                "{}let _rot = egui_expressive::Transform2D::rotate_around({:.4}, rect.center());\n",
                inner, style.rotation_deg
            ));
        }
        let stops_str: String = grad
            .stops
            .iter()
            .map(|s| {
                let [sr, sg, sb, sa] = s.color.to_srgba_unmultiplied();
                let a = (sa as f32 * style.opacity).clamp(0.0, 255.0) as u8;
                format!(
                    "({:.3}, egui::Color32::from_rgba_unmultiplied({}, {}, {}, {}))",
                    s.position, sr, sg, sb, a
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        match grad.gradient_type {
            GradientType::Linear => {
                if has_rotation || style.corner_radius > 0.001 || grad.transform.is_some() {
                    let transform_expr = grad
                                .transform
                                .map(|m| {
                                    format!(
                                        "Some(egui_expressive::Transform2D {{ a: {:.4}, b: {:.4}, c: {:.4}, d: {:.4}, e: origin.x + {:.4} - {:.4} * origin.x - {:.4} * origin.y, f: origin.y + {:.4} - {:.4} * origin.x - {:.4} * origin.y }})",
                                        m[0], m[1], m[2], m[3], m[4], m[0], m[2], m[5], m[1], m[3]
                                    )
                                })
                                .unwrap_or_else(|| "None".to_string());
                    let gradient_rect_points = if has_rotation {
                        if style.corner_radius > 0.001 {
                            format!(
                                        "egui_expressive::rounded_rect_path(rect, {:.1}).into_iter().map(|p| _rot.apply(p)).collect::<Vec<_>>()",
                                        style.corner_radius
                                    )
                        } else {
                            "vec![_rot.apply(rect.left_top()), _rot.apply(rect.right_top()), _rot.apply(rect.right_bottom()), _rot.apply(rect.left_bottom())]".to_string()
                        }
                    } else if style.corner_radius > 0.001 {
                        format!(
                            "egui_expressive::rounded_rect_path(rect, {:.1})",
                            style.corner_radius
                        )
                    } else {
                        "vec![rect.left_top(), rect.right_top(), rect.right_bottom(), rect.left_bottom()]".to_string()
                    };
                    output.push_str(&format!(
                        "{}let gradient_rect_pts = {};\n",
                        inner, gradient_rect_points
                    ));
                    output.push_str(&format!(
                                "{}let mut grad_shape = egui_expressive::gradient_path_mesh_with_transform(&gradient_rect_pts, &[{}], {:.1}, false, egui_expressive::GradientPathGeometry {{ transform: {}, ..Default::default() }}).unwrap_or(egui::Shape::Noop);\n",
                                inner, stops_str, grad.angle_deg, transform_expr
                            ));
                } else {
                    output.push_str(&format!(
                                "{}let mut grad_shape = egui_expressive::linear_gradient_rect(rect, &[{}], egui_expressive::GradientDir::Angle({:.1}));\n",
                                inner, stops_str, grad.angle_deg
                            ));
                }
            }
            GradientType::Radial => {
                let point_expr = |point: Option<[f32; 2]>| {
                    point
                        .map(|p| format!("Some(origin + egui::vec2({:.1}, {:.1}))", p[0], p[1]))
                        .unwrap_or_else(|| "None".to_string())
                };
                let radius_expr = grad
                    .radius
                    .map(|r| format!("Some({:.1})", r))
                    .unwrap_or_else(|| "None".to_string());
                let transform_expr = grad
                            .transform
                            .map(|m| {
                                format!(
                                    "Some(egui_expressive::Transform2D {{ a: {:.4}, b: {:.4}, c: {:.4}, d: {:.4}, e: origin.x + {:.4} - {:.4} * origin.x - {:.4} * origin.y, f: origin.y + {:.4} - {:.4} * origin.x - {:.4} * origin.y }})",
                                    m[0], m[1], m[2], m[3], m[4], m[0], m[2], m[5], m[1], m[3]
                                )
                            })
                            .unwrap_or_else(|| "None".to_string());
                let gradient_rect_points = if style.corner_radius > 0.001 {
                    if has_rotation {
                        format!(
                                    "egui_expressive::rounded_rect_path(rect, {:.1}).into_iter().map(|p| _rot.apply(p)).collect::<Vec<_>>()",
                                    style.corner_radius
                                )
                    } else {
                        format!(
                            "egui_expressive::rounded_rect_path(rect, {:.1})",
                            style.corner_radius
                        )
                    }
                } else if has_rotation {
                    "vec![_rot.apply(rect.left_top()), _rot.apply(rect.right_top()), _rot.apply(rect.right_bottom()), _rot.apply(rect.left_bottom())]".to_string()
                } else {
                    "vec![rect.left_top(), rect.right_top(), rect.right_bottom(), rect.left_bottom()]".to_string()
                };
                output.push_str(&format!(
                    "{}let gradient_rect_pts = {};\n",
                    inner, gradient_rect_points
                ));
                output.push_str(&format!(
                            "{}let mut grad_shape = egui_expressive::gradient_path_mesh_with_transform(&gradient_rect_pts, &[{}], {:.1}, true, egui_expressive::GradientPathGeometry {{ center: {}, focal_point: {}, radius: {}, transform: {} }}).unwrap_or(egui::Shape::Noop);\n",
                            inner,
                            stops_str,
                            grad.angle_deg,
                            point_expr(grad.center),
                            point_expr(grad.focal_point),
                            radius_expr,
                            transform_expr
                        ));
            }
        }
        output.push_str(&format!("{}painter.add(grad_shape);\n", inner));
        // Emit stroke on top of gradient fill if present
        if style.stroke.is_some() {
            let stroke_points = if style.corner_radius > 0.001 {
                if has_rotation {
                    format!(
                                "egui_expressive::rounded_rect_path(rect, {:.1}).into_iter().map(|p| _rot.apply(p)).collect::<Vec<_>>()",
                                style.corner_radius
                            )
                } else {
                    format!(
                        "egui_expressive::rounded_rect_path(rect, {:.1})",
                        style.corner_radius
                    )
                }
            } else if has_rotation {
                "vec![_rot.apply(rect.left_top()), _rot.apply(rect.right_top()), _rot.apply(rect.right_bottom()), _rot.apply(rect.left_bottom())]".to_string()
            } else {
                "vec![rect.left_top(), rect.right_top(), rect.right_bottom(), rect.left_bottom()]"
                    .to_string()
            };
            let closed_stroke_points = format!(
                "{{ let mut pts = {}; pts.push(pts[0]); pts }}",
                stroke_points
            );
            if let Some(dashes) = &style.stroke_dash {
                let dash_values = dashes
                    .iter()
                    .map(|dash| format!("{:.1}", dash))
                    .collect::<Vec<_>>()
                    .join(", ");
                let cap_variant = match style.stroke_cap {
                    Some(StrokeCap::Round) => "Round",
                    Some(StrokeCap::Square) => "Square",
                    _ => "Butt",
                };
                let join_variant = match style.stroke_join {
                    Some(StrokeJoin::Round) => "Round",
                    Some(StrokeJoin::Bevel) => "Bevel",
                    Some(StrokeJoin::Miter) | None
                        if style.stroke_miter_limit.unwrap_or(4.0) <= 1.0 =>
                    {
                        "Bevel"
                    }
                    _ => "Miter",
                };
                output.push_str(&format!(
                            "{}if stroke != egui::Stroke::NONE {{ let stroke_pts = {}; let rich_stroke = egui_expressive::RichStroke {{ width: stroke.width, color: stroke.color, dash: Some(egui_expressive::DashPattern {{ dashes: vec![{}], offset: 0.0 }}), cap: egui_expressive::StrokeCap::{}, join: egui_expressive::StrokeJoin::{} }}; egui_expressive::dashed_path(&painter, &stroke_pts, &rich_stroke); }}\n",
                            inner, closed_stroke_points, dash_values, cap_variant, join_variant
                        ));
            } else if has_rotation || style.corner_radius > 0.001 {
                output.push_str(&format!(
                            "{}if stroke != egui::Stroke::NONE {{ painter.add(egui::Shape::closed_line({}, stroke)); }}\n",
                            inner, closed_stroke_points
                        ));
            } else {
                output.push_str(&format!(
                            "{}if stroke != egui::Stroke::NONE {{ painter.rect_stroke(rect, {:.1}, stroke, egui::StrokeKind::Outside); }}\n",
                            inner, style.corner_radius
                        ));
            }
        }
    } else {
        // Solid fill — use the pre-declared `fill` and `stroke` variables (which already handle opacity)
        let rounding = style.corner_radius;
        output.push_str(&format!(
                    "{}let shape = egui_expressive::ShapeBuilder::rect(rect).fill(fill).stroke(stroke).rounding({:.1}).build();\n",
                    inner, rounding
                ));
        output.push_str(&format!("{}painter.add(shape);\n", inner));
    }

    // Post-shape effects (inner shadow, noise, bevel, blur, feather)
    for effect in &style.effects {
        match effect.effect_type {
            EffectType::InnerShadow => {
                output.push_str(&format!(
                            "{}for s in egui_expressive::inner_shadow(rect, egui::Color32::from_rgba_unmultiplied({}, {}, {}, {}), {:.1}) {{ painter.add(s); }}\n",
                            inner,
                            effect.color.r(),
                            effect.color.g(),
                            effect.color.b(),
                            effect.color.a(),
                            effect.blur
                        ));
            }
            EffectType::Noise => {
                output.push_str(&format!(
                            "{}for s in egui_expressive::noise_rect(rect, {}, {:.2}, {:.2}) {{ painter.add(s); }}\n",
                            inner, effect.seed, effect.scale, effect.amount
                        ));
            }
            EffectType::Bevel => {
                let highlight = effect
                    .highlight
                    .unwrap_or_else(|| egui::Color32::from_rgba_unmultiplied(255, 255, 255, 140));
                let shadow = effect
                    .shadow_color
                    .unwrap_or_else(|| egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150));
                output.push_str(&format!(
                            "{}for s in egui_expressive::bevel_rect(rect, {:.1}, {:.1}, {:.1}, egui::Color32::from_rgba_unmultiplied({}, {}, {}, {}), egui::Color32::from_rgba_unmultiplied({}, {}, {}, {})) {{ painter.add(s); }}\n",
                            inner,
                            effect.depth,
                            effect.angle,
                            effect.radius,
                            highlight.r(),
                            highlight.g(),
                            highlight.b(),
                            highlight.a(),
                            shadow.r(),
                            shadow.g(),
                            shadow.b(),
                            shadow.a()
                        ));
            }
            EffectType::GaussianBlur => {
                output.push_str(&format!(
                            "{}for s in egui_expressive::blur::soft_shadow(rect, egui::Color32::from_rgba_unmultiplied({}, {}, {}, {}), {:.1}, 0.0, egui_expressive::ShadowOffset::zero(), egui_expressive::blur::BlurQuality::High) {{ painter.add(s); }}\n",
                            inner,
                            effect.color.r(),
                            effect.color.g(),
                            effect.color.b(),
                            effect.color.a(),
                            effect.radius
                        ));
            }
            EffectType::Feather => {
                output.push_str(&format!(
                            "{}for s in egui_expressive::blur::soft_shadow(rect, egui::Color32::from_rgba_unmultiplied({}, {}, {}, {}), {:.1}, 0.0, egui_expressive::ShadowOffset::zero(), egui_expressive::blur::BlurQuality::High) {{ painter.add(s); }}\n",
                            inner,
                            effect.color.r(),
                            effect.color.g(),
                            effect.color.b(),
                            effect.color.a(),
                            effect.radius
                        ));
            }
            EffectType::LiveEffect => {
                output.push_str(&format!("{}// live_effect\n", inner));
            }
            EffectType::Unknown(ref name) => {
                output.push_str(&format!("{}// unknown effect: {}\n", inner, name));
            }
            _ => {}
        }
    }

    output.push_str(&format!("{}}}\n", indent_str));

    output
}
