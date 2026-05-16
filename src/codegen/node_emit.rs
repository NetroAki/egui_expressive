use super::*;
pub fn generate_node(
    node: &LayoutNode,
    indent: usize,
    token_map: Option<&HashMap<String, Color32>>,
) -> String {
    let indent_str = " ".repeat(indent);
    let mut output = String::new();
    match node {
        LayoutNode::Row {
            gap,
            children,
            bg,
            id,
        } => {
            output.push_str(&generate_row_node(
                *gap, children, *bg, id, indent, token_map,
            ));
        }
        LayoutNode::Column {
            gap,
            children,
            bg,
            id,
        } => {
            output.push_str(&generate_column_node(
                *gap, children, *bg, id, indent, token_map,
            ));
        }
        LayoutNode::ScrollArea {
            vertical,
            horizontal,
            children,
            id,
        } => {
            output.push_str(&format!("{}// ScrollArea: {}\n", indent_str, id));
            let scroll_type = match (*vertical, *horizontal) {
                (true, false) => "egui::ScrollArea::vertical()",
                (false, true) => "egui::ScrollArea::horizontal()",
                (true, true) => "egui::ScrollArea::both()",
                (false, false) => "egui::ScrollArea::vertical()",
            };
            output.push_str(&format!(
                "{}{}.id_salt({:?}).show(ui, |ui| {{\n",
                indent_str, scroll_type, id
            ));
            for child in children {
                output.push_str(&generate_node(child, indent + 4, token_map));
            }
            output.push_str(&format!("{}}});\n", indent_str));
        }
        LayoutNode::Panel { side, children, id } => {
            output.push_str(&format!("{}// Panel: {:?} - {}\n", indent_str, side, id));
            let (width, height) = calculate_panel_dimensions(children, *side);
            output.push_str(&format!(
                "{}ui.allocate_ui(egui::vec2({:.1}, {:.1}), |ui| {{\n",
                indent_str, width, height
            ));
            for child in children {
                output.push_str(&generate_node(child, indent + 4, token_map));
            }
            output.push_str(&format!("{}}});\n", indent_str));
        }
        LayoutNode::Card {
            children,
            bg,
            rounding,
            id,
        } => {
            output.push_str(&format!("{}// Card: {}\n", indent_str, id));
            let (w, h) = calculate_card_dimensions(children);
            output.push_str(&format!(
                "{}let card_rect = egui::Rect::from_min_size(ui.cursor().min, egui::vec2({:.1}, {:.1}));\n",
                indent_str, w, h
            ));
            output.push_str(&format!(
                "{}ui.painter().rect_filled(card_rect, {:.1}, {});\n",
                indent_str,
                rounding,
                color_to_token_or_literal(bg, token_map)
            ));
            output.push_str(&format!("{}vstack!(ui, gap: 8.0, {{\n", indent_str));
            for child in children {
                output.push_str(&generate_node(child, indent + 4, token_map));
            }
            output.push_str(&format!("{}}});\n", indent_str));
        }
        LayoutNode::Button { label, id } => {
            output.push_str(&format!(
                "{}// Button: {}\n{}if ui.button(\"{}\").clicked() {{\n{}{}}}\n",
                indent_str, id, indent_str, label, indent_str, indent_str
            ));
        }
        LayoutNode::Label {
            text,
            size,
            color,
            font_family,
            id,
        } => {
            let color_str = if let Some(c) = color {
                color_to_token_or_literal(c, token_map)
            } else {
                "egui::Color32::from_gray(200)".to_string()
            };
            let font_chain = if let Some(family) = font_family {
                format!(".family(egui::FontFamily::Name(\"{}\".into()))", family)
            } else {
                String::new()
            };
            output.push_str(&format!(
                "{}// Label: {}\n{}ui.label(egui::RichText::new(\"{}\").size({:.1}).color({}){});\n",
                indent_str,
                id,
                indent_str,
                text.replace('"', "\\\""),
                size,
                color_str,
                font_chain
            ));
        }
        LayoutNode::TextEdit { placeholder, id } => {
            let sanitized_id = id.replace(['-', ' '], "_");
            output.push_str(&format!(
                "{}// TextEdit: {}\n{}ui.add(egui::TextEdit::singleline(&mut state.{})",
                indent_str, id, indent_str, sanitized_id
            ));
            if !placeholder.is_empty() {
                output.push_str(&format!(
                    ".hint_text(\"{}\")",
                    placeholder.replace('"', "\\\"")
                ));
            }
            output.push_str(");\n");
        }
        LayoutNode::Separator { id } => {
            output.push_str(&format!(
                "{}// Separator: {}\n{}ui.separator();\n",
                indent_str, id, indent_str
            ));
        }
        LayoutNode::Spacer { size, id } => {
            output.push_str(&format!(
                "{}// Spacer: {}\n{}ui.add_space({:.1});\n",
                indent_str, id, indent_str, size
            ));
        }
        LayoutNode::Badge { text, id } => {
            output.push_str(&format!(
                "{}// Badge: {}\n{}ui.label(egui::RichText::new(\"{}\")\n",
                indent_str,
                id,
                indent_str,
                text.replace('"', "\\\"")
            ));
            output.push_str(&format!("{}.size(11.0)\n", indent_str));
            output.push_str(&format!(
                "{}.color(egui::Color32::from_rgb(100, 200, 255)));\n",
                indent_str
            ));
        }
        LayoutNode::Icon { name, id } => {
            output.push_str(&format!(
                "{}// Icon: {} - {} (Icons are not natively supported yet; implement custom rendering here)\n",
                indent_str, id, name
            ));
        }
        LayoutNode::Shape {
            x,
            y,
            w,
            h,
            fill,
            id,
            style,
        } => {
            output.push_str(&format!(
                "{}// Shape: {}\n{}{{\n",
                indent_str, id, indent_str
            ));
            let inner = " ".repeat(indent + 4);
            output.push_str(&format!(
                "{}let rect = egui::Rect::from_min_size(origin + egui::vec2({:.1}, {:.1}), egui::vec2({:.1}, {:.1}));\n",
                inner, x, y, w, h
            ));
            for effect in &style.effects {
                output.push_str(&super::effect_emit::emit_pre_shape_effect(
                    effect,
                    style.opacity,
                    &inner,
                ));
            }
            let fill_color = color_to_token_or_literal(fill, token_map);
            if style.opacity < 1.0 {
                output.push_str(&format!(
                    "{}let fill = egui_expressive::with_alpha({}, {:.2});\n",
                    inner, fill_color, style.opacity
                ));
            } else {
                output.push_str(&format!("{}let fill = {};\n", inner, fill_color));
            }
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
                                .map(|p| {
                                    format!("Some(origin + egui::vec2({:.1}, {:.1}))", p[0], p[1])
                                })
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
                        "vec![rect.left_top(), rect.right_top(), rect.right_bottom(), rect.left_bottom()]".to_string()
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
                let rounding = style.corner_radius;
                output.push_str(&format!(
                    "{}let shape = egui_expressive::ShapeBuilder::rect(rect).fill(fill).stroke(stroke).rounding({:.1}).build();\n",
                    inner, rounding
                ));
                output.push_str(&format!("{}painter.add(shape);\n", inner));
            }
            for effect in &style.effects {
                output.push_str(&super::effect_emit::emit_post_shape_effect(effect, &inner));
            }
            output.push_str(&format!("{}}}\n", indent_str));
        }
        LayoutNode::Image {
            x,
            y,
            w,
            h,
            id,
            style,
        } => {
            output.push_str(&format!(
                "{}// Image: {}\n{}{{\n",
                indent_str, id, indent_str
            ));
            let inner = " ".repeat(indent + 4);
            output.push_str(&format!(
                "{}let rect = egui::Rect::from_min_size(origin + egui::vec2({:.1}, {:.1}), egui::vec2({:.1}, {:.1}));\n",
                inner, x, y, w, h
            ));
            let alpha = (255.0 * style.opacity).clamp(0.0, 255.0) as u8;
            if let Some(path) = &style.image_path {
                output.push_str(&format!(
                    "{}egui_expressive::paint_image_slot(ui, &ui.painter(), rect, Some(\"{}\"), \"{}\", egui::Color32::from_rgba_unmultiplied(255, 255, 255, {}), \"Missing Image\");\n",
                    inner, path, id, alpha
                ));
            } else {
                output.push_str(&format!(
                    "{}// Note: Image asset slot emitted without linked path for \"{}\".\n",
                    inner, id
                ));
                output.push_str(&format!(
                    "{}egui_expressive::paint_image_slot(ui, &ui.painter(), rect, None, \"{}\", egui::Color32::from_rgba_unmultiplied(255, 255, 255, {}), \"Image Slot\");\n",
                    inner, id, alpha
                ));
            }
            output.push_str(&format!("{}}}\n", indent_str));
        }
        LayoutNode::Unknown { id, comment } => {
            output.push_str(&format!("{}// Unknown: {} ({})\n", indent_str, id, comment));
        }
        LayoutNode::RichScene(scene_node) => {
            output.push_str(&format!("{}// RichScene: {}\n", indent_str, scene_node.id));
            output.push_str(&format!("{}{{\n", indent_str));
            let inner = " ".repeat(indent + 4);
            output.push_str(&format!(
                "{}let node = {};\n",
                inner,
                generate_scene_node_code(scene_node, indent + 4)
            ));
            output.push_str(&format!("{}egui_expressive::scene::render_node(ui, &painter, origin.to_vec2(), &node, 1.0);\n", inner));
            output.push_str(&format!("{}}}\n", indent_str));
        }
    }
    output
}
