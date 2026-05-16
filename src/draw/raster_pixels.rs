use super::*;

pub(crate) fn path_stroke_color(stroke: &egui::epaint::PathStroke) -> Option<egui::Color32> {
    if stroke.width <= 0.0 {
        return None;
    }
    match stroke.color {
        egui::epaint::ColorMode::Solid(color) if color != egui::Color32::TRANSPARENT => Some(color),
        _ => None,
    }
}

pub(crate) fn fill_rect_shape_pixels(
    rect_shape: &egui::epaint::RectShape,
    origin: egui::Pos2,
    width: u32,
    height: u32,
    pixels: &mut [egui::Color32],
) {
    if rect_shape.fill == egui::Color32::TRANSPARENT {
        return;
    }
    if rect_shape.corner_radius == egui::CornerRadius::ZERO && rect_shape.angle.abs() <= 0.0001 {
        fill_rect_pixels(
            rect_shape.rect,
            origin,
            width,
            height,
            rect_shape.fill,
            pixels,
        );
        return;
    }
    let points =
        rounded_rect_shape_path(rect_shape.rect, rect_shape.corner_radius, rect_shape.angle);
    fill_polygon_pixels(&points, origin, width, height, rect_shape.fill, pixels);
}

pub(crate) fn stroke_rect_shape_pixels(
    rect_shape: &egui::epaint::RectShape,
    origin: egui::Pos2,
    width: u32,
    height: u32,
    pixels: &mut [egui::Color32],
) {
    if rect_shape.stroke.is_empty() {
        return;
    }
    let stroke_width = rect_shape.stroke.width.max(1.0);
    let center_outset = rect_stroke_center_outset(rect_shape.stroke_kind, stroke_width);
    let rect = rect_shape.rect.expand(center_outset);
    if !rect.is_positive() {
        return;
    }
    let radius = (rect_shape.corner_radius.average() + center_outset).max(0.0);
    let mut points = rounded_rect_path(rect, radius);
    if rect_shape.angle.abs() > 0.0001 {
        let transform =
            Transform2D::rotate_around(rect_shape.angle.to_degrees(), rect_shape.rect.center());
        for point in &mut points {
            *point = transform.apply(*point);
        }
    }
    stroke_polyline_pixels(
        &points,
        true,
        origin,
        width,
        height,
        rect_shape.stroke.width,
        rect_shape.stroke.color,
        pixels,
    );
}

pub(crate) fn rounded_rect_shape_path(
    rect: egui::Rect,
    corner_radius: egui::CornerRadius,
    angle_rad: f32,
) -> Vec<egui::Pos2> {
    let mut points = rounded_rect_path(rect, corner_radius.average());
    if angle_rad.abs() > 0.0001 {
        let transform = Transform2D::rotate_around(angle_rad.to_degrees(), rect.center());
        for point in &mut points {
            *point = transform.apply(*point);
        }
    }
    points
}

pub(crate) fn rect_stroke_center_outset(stroke_kind: egui::StrokeKind, stroke_width: f32) -> f32 {
    match stroke_kind {
        egui::StrokeKind::Inside => -stroke_width * 0.5,
        egui::StrokeKind::Middle => 0.0,
        egui::StrokeKind::Outside => stroke_width * 0.5,
    }
}

pub(crate) fn fill_rect_pixels(
    rect: egui::Rect,
    origin: egui::Pos2,
    width: u32,
    height: u32,
    color: egui::Color32,
    pixels: &mut [egui::Color32],
) {
    if color == egui::Color32::TRANSPARENT {
        return;
    }
    let x0 = (rect.min.x - origin.x).floor().max(0.0) as u32;
    let y0 = (rect.min.y - origin.y).floor().max(0.0) as u32;
    let x1 = (rect.max.x - origin.x).ceil().min(width as f32) as u32;
    let y1 = (rect.max.y - origin.y).ceil().min(height as f32) as u32;
    for y in y0..y1 {
        for x in x0..x1 {
            pixels[(y * width + x) as usize] = blend_color(
                color,
                pixels[(y * width + x) as usize],
                crate::codegen::BlendMode::Normal,
            );
        }
    }
}

pub(crate) fn fill_circle_pixels(
    center: egui::Pos2,
    radius: f32,
    origin: egui::Pos2,
    width: u32,
    height: u32,
    color: egui::Color32,
    pixels: &mut [egui::Color32],
) {
    if color == egui::Color32::TRANSPARENT || radius <= 0.0 {
        return;
    }
    let r2 = radius * radius;
    let rect = egui::Rect::from_center_size(center, egui::vec2(radius * 2.0, radius * 2.0));
    let x0 = (rect.min.x - origin.x).floor().max(0.0) as u32;
    let y0 = (rect.min.y - origin.y).floor().max(0.0) as u32;
    let x1 = (rect.max.x - origin.x).ceil().min(width as f32) as u32;
    let y1 = (rect.max.y - origin.y).ceil().min(height as f32) as u32;
    for y in y0..y1 {
        for x in x0..x1 {
            let p = egui::pos2(origin.x + x as f32 + 0.5, origin.y + y as f32 + 0.5);
            if p.distance_sq(center) <= r2 {
                let idx = (y * width + x) as usize;
                pixels[idx] = blend_color(color, pixels[idx], crate::codegen::BlendMode::Normal);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn stroke_circle_pixels(
    center: egui::Pos2,
    radius: f32,
    origin: egui::Pos2,
    width: u32,
    height: u32,
    stroke_width: f32,
    color: egui::Color32,
    pixels: &mut [egui::Color32],
) {
    if color == egui::Color32::TRANSPARENT || radius <= 0.0 || stroke_width <= 0.0 {
        return;
    }
    let half = stroke_width.max(1.0) * 0.5;
    let outer = radius + half;
    let inner = (radius - half).max(0.0);
    let outer2 = outer * outer;
    let inner2 = inner * inner;
    let rect = egui::Rect::from_center_size(center, egui::vec2(outer * 2.0, outer * 2.0));
    let x0 = (rect.min.x - origin.x).floor().max(0.0) as u32;
    let y0 = (rect.min.y - origin.y).floor().max(0.0) as u32;
    let x1 = (rect.max.x - origin.x).ceil().min(width as f32) as u32;
    let y1 = (rect.max.y - origin.y).ceil().min(height as f32) as u32;
    for y in y0..y1 {
        for x in x0..x1 {
            let p = egui::pos2(origin.x + x as f32 + 0.5, origin.y + y as f32 + 0.5);
            let d2 = p.distance_sq(center);
            if d2 <= outer2 && d2 >= inner2 {
                let idx = (y * width + x) as usize;
                pixels[idx] = blend_color(color, pixels[idx], crate::codegen::BlendMode::Normal);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn fill_ellipse_pixels(
    center: egui::Pos2,
    radius: egui::Vec2,
    angle: f32,
    origin: egui::Pos2,
    width: u32,
    height: u32,
    color: egui::Color32,
    pixels: &mut [egui::Color32],
) {
    if color == egui::Color32::TRANSPARENT || radius.x <= 0.0 || radius.y <= 0.0 {
        return;
    }
    let Some(bounds) = ellipse_bounds(center, radius, angle, 0.0) else {
        return;
    };
    let x0 = (bounds.min.x - origin.x).floor().max(0.0) as u32;
    let y0 = (bounds.min.y - origin.y).floor().max(0.0) as u32;
    let x1 = (bounds.max.x - origin.x).ceil().min(width as f32) as u32;
    let y1 = (bounds.max.y - origin.y).ceil().min(height as f32) as u32;
    for y in y0..y1 {
        for x in x0..x1 {
            let p = egui::pos2(origin.x + x as f32 + 0.5, origin.y + y as f32 + 0.5);
            if point_in_ellipse(p, center, radius, angle) {
                let idx = (y * width + x) as usize;
                pixels[idx] = blend_color(color, pixels[idx], crate::codegen::BlendMode::Normal);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn stroke_ellipse_pixels(
    center: egui::Pos2,
    radius: egui::Vec2,
    angle: f32,
    origin: egui::Pos2,
    width: u32,
    height: u32,
    stroke_width: f32,
    color: egui::Color32,
    pixels: &mut [egui::Color32],
) {
    if color == egui::Color32::TRANSPARENT
        || radius.x <= 0.0
        || radius.y <= 0.0
        || stroke_width <= 0.0
    {
        return;
    }
    let half = stroke_width.max(1.0) * 0.5;
    let outer_radius = egui::vec2(radius.x + half, radius.y + half);
    let inner_radius = egui::vec2((radius.x - half).max(0.0), (radius.y - half).max(0.0));
    let Some(bounds) = ellipse_bounds(center, outer_radius, angle, 0.0) else {
        return;
    };
    let x0 = (bounds.min.x - origin.x).floor().max(0.0) as u32;
    let y0 = (bounds.min.y - origin.y).floor().max(0.0) as u32;
    let x1 = (bounds.max.x - origin.x).ceil().min(width as f32) as u32;
    let y1 = (bounds.max.y - origin.y).ceil().min(height as f32) as u32;
    for y in y0..y1 {
        for x in x0..x1 {
            let p = egui::pos2(origin.x + x as f32 + 0.5, origin.y + y as f32 + 0.5);
            if point_in_ellipse(p, center, outer_radius, angle)
                && !point_in_ellipse(p, center, inner_radius, angle)
            {
                let idx = (y * width + x) as usize;
                pixels[idx] = blend_color(color, pixels[idx], crate::codegen::BlendMode::Normal);
            }
        }
    }
}

pub(crate) fn ellipse_bounds(
    center: egui::Pos2,
    radius: egui::Vec2,
    angle: f32,
    outset: f32,
) -> Option<egui::Rect> {
    let rx = radius.x.abs() + outset;
    let ry = radius.y.abs() + outset;
    if rx <= 0.0 || ry <= 0.0 {
        return None;
    }
    let mut points = Vec::with_capacity(64);
    let (sin, cos) = angle.sin_cos();
    for idx in 0..64 {
        let theta = std::f32::consts::TAU * idx as f32 / 64.0;
        let local = egui::vec2(theta.cos() * rx, theta.sin() * ry);
        points.push(
            center + egui::vec2(cos * local.x - sin * local.y, sin * local.x + cos * local.y),
        );
    }
    bounds_from_points(&points)
}

pub(crate) fn point_in_ellipse(
    point: egui::Pos2,
    center: egui::Pos2,
    radius: egui::Vec2,
    angle: f32,
) -> bool {
    let rx = radius.x.abs();
    let ry = radius.y.abs();
    if rx <= 0.0001 || ry <= 0.0001 {
        return false;
    }
    let v = point - center;
    let (sin, cos) = (-angle).sin_cos();
    let local = egui::vec2(cos * v.x - sin * v.y, sin * v.x + cos * v.y);
    let nx = local.x / rx;
    let ny = local.y / ry;
    nx * nx + ny * ny <= 1.0
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn stroke_polyline_pixels(
    points: &[egui::Pos2],
    closed: bool,
    origin: egui::Pos2,
    width: u32,
    height: u32,
    stroke_width: f32,
    color: egui::Color32,
    pixels: &mut [egui::Color32],
) {
    if points.len() < 2 {
        return;
    }
    for segment in points.windows(2) {
        stroke_line_pixels(
            segment[0],
            segment[1],
            origin,
            width,
            height,
            stroke_width,
            color,
            pixels,
        );
    }
    if closed {
        stroke_line_pixels(
            *points.last().unwrap(),
            points[0],
            origin,
            width,
            height,
            stroke_width,
            color,
            pixels,
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn stroke_line_pixels(
    a: egui::Pos2,
    b: egui::Pos2,
    origin: egui::Pos2,
    width: u32,
    height: u32,
    stroke_width: f32,
    color: egui::Color32,
    pixels: &mut [egui::Color32],
) {
    if color == egui::Color32::TRANSPARENT || stroke_width <= 0.0 {
        return;
    }
    let Some(bounds) = bounds_from_points(&[a, b]).map(|r| r.expand(stroke_width.max(1.0))) else {
        return;
    };
    let x0 = (bounds.min.x - origin.x).floor().max(0.0) as u32;
    let y0 = (bounds.min.y - origin.y).floor().max(0.0) as u32;
    let x1 = (bounds.max.x - origin.x).ceil().min(width as f32) as u32;
    let y1 = (bounds.max.y - origin.y).ceil().min(height as f32) as u32;
    let ab = b - a;
    let len2 = ab.length_sq();
    if len2 <= 0.0001 {
        fill_circle_pixels(a, stroke_width * 0.5, origin, width, height, color, pixels);
        return;
    }
    let radius = stroke_width.max(1.0) * 0.5;
    let radius2 = radius * radius;
    for y in y0..y1 {
        for x in x0..x1 {
            let p = egui::pos2(origin.x + x as f32 + 0.5, origin.y + y as f32 + 0.5);
            let t = ((p - a).dot(ab) / len2).clamp(0.0, 1.0);
            let closest = a + ab * t;
            if p.distance_sq(closest) <= radius2 {
                let idx = (y * width + x) as usize;
                pixels[idx] = blend_color(color, pixels[idx], crate::codegen::BlendMode::Normal);
            }
        }
    }
}

pub(crate) fn fill_polygon_pixels(
    points: &[egui::Pos2],
    origin: egui::Pos2,
    width: u32,
    height: u32,
    color: egui::Color32,
    pixels: &mut [egui::Color32],
) {
    if points.len() < 3 || color == egui::Color32::TRANSPARENT {
        return;
    }
    let Some(bounds) = bounds_from_points(points) else {
        return;
    };
    let x0 = (bounds.min.x - origin.x).floor().max(0.0) as u32;
    let y0 = (bounds.min.y - origin.y).floor().max(0.0) as u32;
    let x1 = (bounds.max.x - origin.x).ceil().min(width as f32) as u32;
    let y1 = (bounds.max.y - origin.y).ceil().min(height as f32) as u32;
    for y in y0..y1 {
        for x in x0..x1 {
            let p = egui::pos2(origin.x + x as f32 + 0.5, origin.y + y as f32 + 0.5);
            if point_in_polygon(p, points) {
                let idx = (y * width + x) as usize;
                pixels[idx] = blend_color(color, pixels[idx], crate::codegen::BlendMode::Normal);
            }
        }
    }
}

pub(crate) fn rasterize_mesh_pixels(
    mesh: &egui::epaint::Mesh,
    origin: egui::Pos2,
    width: u32,
    height: u32,
    pixels: &mut [egui::Color32],
) {
    for tri in mesh.indices.chunks_exact(3) {
        let a = &mesh.vertices[tri[0] as usize];
        let b = &mesh.vertices[tri[1] as usize];
        let c = &mesh.vertices[tri[2] as usize];
        rasterize_triangle_pixels(
            [a.pos, b.pos, c.pos],
            [a.color, b.color, c.color],
            origin,
            width,
            height,
            pixels,
        );
    }
}

pub(crate) fn rasterize_triangle_pixels(
    points: [egui::Pos2; 3],
    colors: [egui::Color32; 3],
    origin: egui::Pos2,
    width: u32,
    height: u32,
    pixels: &mut [egui::Color32],
) {
    let Some(bounds) = bounds_from_points(&points) else {
        return;
    };
    let x0 = (bounds.min.x - origin.x).floor().max(0.0) as u32;
    let y0 = (bounds.min.y - origin.y).floor().max(0.0) as u32;
    let x1 = (bounds.max.x - origin.x).ceil().min(width as f32) as u32;
    let y1 = (bounds.max.y - origin.y).ceil().min(height as f32) as u32;
    let denom = cross2(points[1] - points[0], points[2] - points[0]);
    if denom.abs() <= 0.0001 {
        return;
    }
    let channels = colors.map(|color| color.to_srgba_unmultiplied());
    for y in y0..y1 {
        for x in x0..x1 {
            let p = egui::pos2(origin.x + x as f32 + 0.5, origin.y + y as f32 + 0.5);
            let w0 = cross2(points[1] - p, points[2] - p) / denom;
            let w1 = cross2(points[2] - p, points[0] - p) / denom;
            let w2 = 1.0 - w0 - w1;
            if w0 >= -0.001 && w1 >= -0.001 && w2 >= -0.001 {
                let channel = |idx: usize| {
                    (channels[0][idx] as f32 * w0
                        + channels[1][idx] as f32 * w1
                        + channels[2][idx] as f32 * w2)
                        .round()
                        .clamp(0.0, 255.0) as u8
                };
                let color = egui::Color32::from_rgba_unmultiplied(
                    channel(0),
                    channel(1),
                    channel(2),
                    channel(3),
                );
                let idx = (y * width + x) as usize;
                pixels[idx] = blend_color(color, pixels[idx], crate::codegen::BlendMode::Normal);
            }
        }
    }
}

pub(crate) fn point_in_polygon(p: egui::Pos2, polygon: &[egui::Pos2]) -> bool {
    let mut inside = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        let pi = polygon[i];
        let pj = polygon[j];
        if ((pi.y > p.y) != (pj.y > p.y))
            && (p.x < (pj.x - pi.x) * (p.y - pi.y) / (pj.y - pi.y) + pi.x)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

pub(crate) fn apply_polygon_alpha_mask(
    pixels: &mut [egui::Color32],
    width: u32,
    height: u32,
    origin: egui::Pos2,
    polygon: &[egui::Pos2],
) {
    for y in 0..height {
        for x in 0..width {
            let p = egui::pos2(origin.x + x as f32 + 0.5, origin.y + y as f32 + 0.5);
            if !point_in_polygon(p, polygon) {
                pixels[(y * width + x) as usize] = egui::Color32::TRANSPARENT;
            }
        }
    }
}

pub(crate) fn apply_clip_mask(
    pixels: &mut [egui::Color32],
    width: u32,
    height: u32,
    origin: egui::Pos2,
    mask: &ClipMask,
) {
    for y in 0..height {
        for x in 0..width {
            let p = egui::pos2(origin.x + x as f32 + 0.5, origin.y + y as f32 + 0.5);
            if !mask.contains(p) {
                pixels[(y * width + x) as usize] = egui::Color32::TRANSPARENT;
            }
        }
    }
}

pub(crate) fn color_with_opacity(color: egui::Color32, opacity: f32) -> egui::Color32 {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    egui::Color32::from_rgba_unmultiplied(
        r,
        g,
        b,
        (a as f32 * opacity.clamp(0.0, 1.0)).round() as u8,
    )
}
