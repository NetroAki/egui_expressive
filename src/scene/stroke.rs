use super::*;

pub(crate) fn paint_stroke(
    ui: &mut egui::Ui,
    origin: egui::Vec2,
    geometry: &Geometry,
    stroke: &StrokeLayer,
    node_opacity: f32,
    node_blend_mode: &BlendMode,
    force_normal_blend: bool,
) -> Vec<egui::Shape> {
    let opacity = stroke.opacity * node_opacity;
    let blend_mode = if force_normal_blend {
        &BlendMode::Normal
    } else if stroke.blend_mode != BlendMode::Normal {
        &stroke.blend_mode
    } else {
        node_blend_mode
    };

    let color = representative_stroke_color(ui, &stroke.paint, opacity, blend_mode);
    if color == egui::Color32::TRANSPARENT || stroke.width <= 0.0 {
        return vec![];
    }
    let egui_stroke = egui::Stroke::new(stroke.width, color);
    let needs_rich_stroke = stroke.dash.is_some()
        || stroke.cap.is_some()
        || stroke.join.is_some()
        || stroke.miter_limit.is_some();
    let mut shapes = Vec::new();
    match geometry {
        Geometry::Rect {
            rect,
            corner_radius,
        } => {
            let rect = offset_rect(*rect, origin);
            if needs_rich_stroke {
                shapes.extend(stroke_path_shapes(
                    crate::draw::rounded_rect_path(rect, *corner_radius),
                    true,
                    stroke,
                    color,
                    egui_stroke,
                ));
            } else {
                shapes.push(egui::Shape::Rect(egui::epaint::RectShape::stroke(
                    rect,
                    *corner_radius,
                    egui_stroke,
                    egui::StrokeKind::Outside,
                )));
            }
        }
        Geometry::Ellipse { rect } => {
            let rect = offset_rect(*rect, origin);
            if needs_rich_stroke {
                shapes.extend(stroke_path_shapes(
                    ellipse_points(rect, 48),
                    true,
                    stroke,
                    color,
                    egui_stroke,
                ));
            } else {
                shapes.push(egui::Shape::ellipse_stroke(
                    rect.center(),
                    egui::vec2(rect.width() * 0.5, rect.height() * 0.5),
                    egui_stroke,
                ));
            }
        }
        Geometry::Path { points, closed } => {
            shapes.extend(stroke_path_shapes(
                offset_points(points, origin),
                *closed,
                stroke,
                color,
                egui_stroke,
            ));
        }
        _ => {}
    }
    shapes
}

fn representative_stroke_color(
    ui: &egui::Ui,
    paint: &PaintSource,
    opacity: f32,
    blend_mode: &BlendMode,
) -> egui::Color32 {
    match paint {
        PaintSource::Solid(color) => resolve_color(ui, *color, opacity, blend_mode),
        PaintSource::LinearGradient(gradient) | PaintSource::RadialGradient(gradient) => {
            let stops = gradient_stops(gradient, opacity, ui, blend_mode);
            if stops.is_empty() {
                return egui::Color32::TRANSPARENT;
            }
            average_colors(stops.iter().map(|(_, color)| *color))
        }
        PaintSource::Pattern(pattern) => resolve_color(ui, pattern.foreground, opacity, blend_mode),
        PaintSource::MeshGradient { colors, .. } => average_colors(
            colors
                .iter()
                .map(|color| resolve_color(ui, *color, opacity, blend_mode)),
        ),
        PaintSource::ProceduralNoise(noise) => resolve_color(
            ui,
            egui::Color32::from_gray(128),
            opacity * noise.opacity,
            blend_mode,
        ),
    }
}

fn average_colors(colors: impl IntoIterator<Item = egui::Color32>) -> egui::Color32 {
    let mut count = 0u32;
    let mut r = 0u32;
    let mut g = 0u32;
    let mut b = 0u32;
    let mut a = 0u32;
    for color in colors {
        count += 1;
        r += color.r() as u32;
        g += color.g() as u32;
        b += color.b() as u32;
        a += color.a() as u32;
    }
    if count == 0 {
        return egui::Color32::TRANSPARENT;
    }
    egui::Color32::from_rgba_unmultiplied(
        (r / count) as u8,
        (g / count) as u8,
        (b / count) as u8,
        (a / count) as u8,
    )
}

pub(crate) fn draw_stroke_cap(cap: Option<&StrokeCap>) -> crate::draw::StrokeCap {
    match cap {
        Some(StrokeCap::Round) => crate::draw::StrokeCap::Round,
        Some(StrokeCap::Square) => crate::draw::StrokeCap::Square,
        _ => crate::draw::StrokeCap::Butt,
    }
}

pub(crate) fn stroke_path_shapes(
    mut points: Vec<egui::Pos2>,
    closed: bool,
    stroke: &StrokeLayer,
    color: egui::Color32,
    egui_stroke: egui::Stroke,
) -> Vec<egui::Shape> {
    let needs_rich_stroke = stroke.dash.is_some()
        || stroke.cap.is_some()
        || stroke.join.is_some()
        || stroke.miter_limit.is_some();
    if closed && points.len() > 2 && points.first() != points.last() {
        points.push(points[0]);
    }
    if needs_rich_stroke {
        let rich = crate::draw::RichStroke {
            width: stroke.width,
            color,
            dash: stroke.dash.as_ref().map(|dashes| crate::draw::DashPattern {
                dashes: dashes.clone(),
                offset: 0.0,
            }),
            cap: draw_stroke_cap(stroke.cap.as_ref()),
            join: draw_stroke_join(stroke.join.as_ref(), stroke.miter_limit),
        };
        crate::draw::dashed_path_shapes(&points, &rich)
    } else if closed {
        vec![egui::Shape::closed_line(points, egui_stroke)]
    } else {
        vec![egui::Shape::line(points, egui_stroke)]
    }
}

pub(crate) fn draw_stroke_join(
    join: Option<&StrokeJoin>,
    miter_limit: Option<f32>,
) -> crate::draw::StrokeJoin {
    match join {
        Some(StrokeJoin::Round) => crate::draw::StrokeJoin::Round,
        Some(StrokeJoin::Bevel) => crate::draw::StrokeJoin::Bevel,
        _ if miter_limit.is_some_and(|limit| limit <= 1.0) => crate::draw::StrokeJoin::Bevel,
        _ => crate::draw::StrokeJoin::Miter,
    }
}
