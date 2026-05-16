use super::*;

/// Direction for linear gradients.
pub enum GradientDir {
    TopToBottom,
    BottomToTop,
    LeftToRight,
    RightToLeft,
    /// Gradient at an arbitrary angle (degrees, CSS convention: 0° = left to right, 90° = top to bottom).
    Angle(f32),
}

/// Build a linear gradient rect as an `egui::Shape::Mesh`.
/// `stops` is a slice of `(position 0.0–1.0, Color32)` pairs.
pub fn linear_gradient_rect(
    rect: egui::Rect,
    stops: &[(f32, egui::Color32)],
    dir: GradientDir,
) -> egui::Shape {
    use egui::epaint::{Mesh, Vertex};

    if stops.is_empty() {
        return egui::Shape::Noop;
    }

    let mut mesh = Mesh::default();

    // Compute diagonal length for angle-based gradients
    let diag_len = (rect.width().powi(2) + rect.height().powi(2)).sqrt();

    // We build a quad strip along the gradient axis.
    // For each stop, we add 2 vertices (one on each side of the rect).
    let n = stops.len();
    for (i, &(t, color)) in stops.iter().enumerate() {
        let t = t.clamp(0.0, 1.0);
        let (p0, p1) = match dir {
            GradientDir::TopToBottom => (
                egui::Pos2::new(rect.min.x, rect.min.y + rect.height() * t),
                egui::Pos2::new(rect.max.x, rect.min.y + rect.height() * t),
            ),
            GradientDir::BottomToTop => (
                egui::Pos2::new(rect.min.x, rect.max.y - rect.height() * t),
                egui::Pos2::new(rect.max.x, rect.max.y - rect.height() * t),
            ),
            GradientDir::LeftToRight => (
                egui::Pos2::new(rect.min.x + rect.width() * t, rect.min.y),
                egui::Pos2::new(rect.min.x + rect.width() * t, rect.max.y),
            ),
            GradientDir::RightToLeft => (
                egui::Pos2::new(rect.max.x - rect.width() * t, rect.min.y),
                egui::Pos2::new(rect.max.x - rect.width() * t, rect.max.y),
            ),
            GradientDir::Angle(deg) => {
                // CSS convention: 0° = left to right, 90° = top to bottom
                let rad = deg.to_radians();
                let (sin_t, cos_t) = rad.sin_cos();

                // Start point at center minus half diagonal in the opposite direction
                let dx = cos_t * diag_len * 0.5;
                let dy = sin_t * diag_len * 0.5;
                let center = rect.center();
                let start = egui::Pos2::new(center.x - dx, center.y - dy);
                let end = egui::Pos2::new(center.x + dx, center.y + dy);

                // Interpolate along the line
                let curr = egui::Pos2::new(
                    start.x + (end.x - start.x) * t,
                    start.y + (end.y - start.y) * t,
                );

                // Perpendicular direction for the strip width
                let perp_dx = -sin_t;
                let perp_dy = cos_t;
                let half_w = diag_len * 0.5;

                (
                    egui::Pos2::new(curr.x - perp_dx * half_w, curr.y - perp_dy * half_w),
                    egui::Pos2::new(curr.x + perp_dx * half_w, curr.y + perp_dy * half_w),
                )
            }
        };
        let uv = egui::epaint::WHITE_UV;
        mesh.vertices.push(Vertex { pos: p0, uv, color });
        mesh.vertices.push(Vertex { pos: p1, uv, color });

        // Add two triangles for each segment (except the last stop)
        if i + 1 < n {
            let base = (i * 2) as u32;
            // Triangle 1: top-left, top-right, bottom-left
            mesh.indices.push(base);
            mesh.indices.push(base + 1);
            mesh.indices.push(base + 2);
            // Triangle 2: top-right, bottom-right, bottom-left
            mesh.indices.push(base + 1);
            mesh.indices.push(base + 3);
            mesh.indices.push(base + 2);
        }
    }

    egui::Shape::Mesh(mesh.into())
}

/// Convenience: two-stop gradient from `top` to `bottom` color.
pub fn gradient_rect(rect: egui::Rect, top: egui::Color32, bottom: egui::Color32) -> egui::Shape {
    linear_gradient_rect(rect, &[(0.0, top), (1.0, bottom)], GradientDir::TopToBottom)
}

/// Build a gradient mesh clipped to a sampled closed path.
///
/// This supports editable Illustrator-style gradient fills for circles, ellipses, and arbitrary
/// closed paths without raster snapshots. Concave polygons are triangulated with ear clipping.
pub fn gradient_path_mesh(
    points: &[egui::Pos2],
    stops: &[(f32, egui::Color32)],
    angle_deg: f32,
    radial: bool,
) -> Option<egui::Shape> {
    gradient_path_mesh_with_geometry(points, stops, angle_deg, radial, None, None, None)
}

/// Build a gradient mesh clipped to a sampled closed path with explicit radial geometry.
///
/// For radial gradients, `center`, `focal_point`, and `radius` preserve Illustrator gradient
/// geometry when available. Missing values fall back to the clipped path bounds.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GradientPathGeometry {
    pub center: Option<egui::Pos2>,
    pub focal_point: Option<egui::Pos2>,
    pub radius: Option<f32>,
    pub transform: Option<Transform2D>,
}

pub fn gradient_path_mesh_with_geometry(
    points: &[egui::Pos2],
    stops: &[(f32, egui::Color32)],
    angle_deg: f32,
    radial: bool,
    center: Option<egui::Pos2>,
    focal_point: Option<egui::Pos2>,
    radius: Option<f32>,
) -> Option<egui::Shape> {
    gradient_path_mesh_with_transform(
        points,
        stops,
        angle_deg,
        radial,
        GradientPathGeometry {
            center,
            focal_point,
            radius,
            transform: None,
        },
    )
}

/// Build a gradient mesh clipped to a sampled closed path with optional gradient transform.
pub fn gradient_path_mesh_with_transform(
    points: &[egui::Pos2],
    stops: &[(f32, egui::Color32)],
    angle_deg: f32,
    radial: bool,
    geometry: GradientPathGeometry,
) -> Option<egui::Shape> {
    if points.len() < 3 || stops.is_empty() {
        return None;
    }

    let mut polygon = points.to_vec();
    if polygon.len() > 3 && polygon.first() == polygon.last() {
        polygon.pop();
    }
    if polygon.len() < 3 {
        return None;
    }

    let inverse_transform = geometry.transform.and_then(|t| t.inverse());
    let sample_point =
        |point: egui::Pos2| inverse_transform.map(|t| t.apply(point)).unwrap_or(point);
    let sample_polygon: Vec<egui::Pos2> = polygon.iter().copied().map(sample_point).collect();

    let display_bounds = bounds_for_points(&polygon);
    let sample_bounds = bounds_for_points(&sample_polygon);
    let display_center = geometry.center.unwrap_or_else(|| display_bounds.center());
    let display_focal_point = geometry.focal_point.unwrap_or(display_center);
    let sample_center = geometry
        .center
        .map(sample_point)
        .unwrap_or_else(|| sample_bounds.center());
    let sample_focal_point = geometry
        .focal_point
        .map(sample_point)
        .unwrap_or(sample_center);
    let angle = angle_deg.to_radians();
    let dir = egui::vec2(angle.cos(), angle.sin());
    let projections: Vec<f32> = sample_polygon
        .iter()
        .map(|p| p.to_vec2().dot(dir))
        .collect();
    let min_proj = projections.iter().copied().fold(f32::INFINITY, f32::min);
    let max_proj = projections
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
    let proj_span = (max_proj - min_proj).max(0.001);
    let max_radius = geometry
        .radius
        .unwrap_or_else(|| {
            polygon
                .iter()
                .map(|p| (sample_point(*p) - sample_center).length())
                .fold(0.0f32, f32::max)
        })
        .max(0.001);
    let indices = triangulate_polygon(&polygon)?;

    let mut mesh = egui::epaint::Mesh::default();
    if radial {
        for triangle in indices.chunks_exact(3) {
            let a = polygon[triangle[0] as usize];
            let b = polygon[triangle[1] as usize];
            let c = polygon[triangle[2] as usize];
            let inner = if point_in_triangle(display_focal_point, a, b, c) {
                display_focal_point
            } else if point_in_triangle(display_center, a, b, c) {
                display_center
            } else {
                egui::pos2((a.x + b.x + c.x) / 3.0, (a.y + b.y + c.y) / 3.0)
            };
            let base = mesh.vertices.len() as u32;
            for point in [a, b, c, inner] {
                mesh.vertices.push(egui::epaint::Vertex {
                    pos: point,
                    uv: egui::epaint::WHITE_UV,
                    color: sample_gradient_color(
                        stops,
                        radial_gradient_t(
                            sample_point(point),
                            sample_center,
                            sample_focal_point,
                            max_radius,
                        ),
                    ),
                });
            }
            mesh.indices.extend_from_slice(&[
                base,
                base + 1,
                base + 3,
                base + 1,
                base + 2,
                base + 3,
                base + 2,
                base,
                base + 3,
            ]);
        }
        return Some(egui::Shape::mesh(mesh));
    }

    for (idx, point) in polygon.iter().enumerate() {
        let t = (projections[idx] - min_proj) / proj_span;
        mesh.vertices.push(egui::epaint::Vertex {
            pos: *point,
            uv: egui::epaint::WHITE_UV,
            color: sample_gradient_color(stops, t),
        });
    }
    mesh.indices = indices;
    Some(egui::Shape::mesh(mesh))
}

pub(crate) fn bounds_for_points(points: &[egui::Pos2]) -> egui::Rect {
    let mut min = points[0];
    let mut max = points[0];
    for point in points.iter().skip(1) {
        min.x = min.x.min(point.x);
        min.y = min.y.min(point.y);
        max.x = max.x.max(point.x);
        max.y = max.y.max(point.y);
    }
    egui::Rect::from_min_max(min, max)
}

pub(crate) fn triangulate_polygon(points: &[egui::Pos2]) -> Option<Vec<u32>> {
    if points.len() < 3 {
        return None;
    }
    let ccw = signed_area(points) > 0.0;
    let mut remaining: Vec<usize> = (0..points.len()).collect();
    let mut indices = Vec::with_capacity((points.len() - 2) * 3);
    let mut guard = 0usize;
    while remaining.len() > 3 && guard < points.len() * points.len() {
        guard += 1;
        let mut clipped = false;
        for i in 0..remaining.len() {
            let prev = remaining[(i + remaining.len() - 1) % remaining.len()];
            let curr = remaining[i];
            let next = remaining[(i + 1) % remaining.len()];
            if !is_convex(points[prev], points[curr], points[next], ccw) {
                continue;
            }
            let contains_point = remaining.iter().any(|&idx| {
                idx != prev
                    && idx != curr
                    && idx != next
                    && point_in_triangle(points[idx], points[prev], points[curr], points[next])
            });
            if contains_point {
                continue;
            }
            if ccw {
                indices.extend_from_slice(&[prev as u32, curr as u32, next as u32]);
            } else {
                indices.extend_from_slice(&[prev as u32, next as u32, curr as u32]);
            }
            remaining.remove(i);
            clipped = true;
            break;
        }
        if !clipped {
            return None;
        }
    }
    if remaining.len() == 3 {
        if ccw {
            indices.extend_from_slice(&[
                remaining[0] as u32,
                remaining[1] as u32,
                remaining[2] as u32,
            ]);
        } else {
            indices.extend_from_slice(&[
                remaining[0] as u32,
                remaining[2] as u32,
                remaining[1] as u32,
            ]);
        }
    }
    Some(indices)
}

pub(crate) fn signed_area(points: &[egui::Pos2]) -> f32 {
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
        .map(|(a, b)| a.x * b.y - b.x * a.y)
        .sum::<f32>()
        * 0.5
}

pub(crate) fn is_convex(a: egui::Pos2, b: egui::Pos2, c: egui::Pos2, ccw: bool) -> bool {
    let cross = cross2(b - a, c - b);
    if ccw {
        cross > 0.0
    } else {
        cross < 0.0
    }
}

pub(crate) fn point_in_triangle(
    p: egui::Pos2,
    a: egui::Pos2,
    b: egui::Pos2,
    c: egui::Pos2,
) -> bool {
    let area = |p1: egui::Pos2, p2: egui::Pos2, p3: egui::Pos2| cross2(p2 - p1, p3 - p1);
    let d1 = area(p, a, b);
    let d2 = area(p, b, c);
    let d3 = area(p, c, a);
    let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(has_neg && has_pos)
}

pub(crate) fn cross2(a: egui::Vec2, b: egui::Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

pub(crate) fn radial_gradient_t(
    point: egui::Pos2,
    center: egui::Pos2,
    focal_point: egui::Pos2,
    radius: f32,
) -> f32 {
    let sample = point - focal_point;
    let sample_distance = sample.length();
    if sample_distance <= 0.001 {
        return 0.0;
    }

    let direction = sample / sample_distance;
    let focal_to_center = focal_point - center;
    let b = focal_to_center.dot(direction);
    let c = focal_to_center.dot(focal_to_center) - radius * radius;
    let discriminant = b * b - c;
    if discriminant <= 0.0 {
        return sample_distance / radius.max(0.001);
    }

    let outer_distance = -b + discriminant.sqrt();
    sample_distance / outer_distance.max(0.001)
}

pub(crate) fn sample_gradient_color(stops: &[(f32, egui::Color32)], t: f32) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    let mut sorted = stops.to_vec();
    sorted.sort_by(|a, b| a.0.total_cmp(&b.0));
    if t <= sorted[0].0 {
        return sorted[0].1;
    }
    for pair in sorted.windows(2) {
        let (a_pos, a_color) = pair[0];
        let (b_pos, b_color) = pair[1];
        if t <= b_pos {
            let span = (b_pos - a_pos).max(0.001);
            let local = ((t - a_pos) / span).clamp(0.0, 1.0);
            let [ar, ag, ab, aa] = a_color.to_srgba_unmultiplied();
            let [br, bg, bb, ba] = b_color.to_srgba_unmultiplied();
            let lerp = |x: u8, y: u8| x as f32 + (y as f32 - x as f32) * local;
            return egui::Color32::from_rgba_unmultiplied(
                lerp(ar, br).round() as u8,
                lerp(ag, bg).round() as u8,
                lerp(ab, bb).round() as u8,
                lerp(aa, ba).round() as u8,
            );
        }
    }
    sorted.last().map(|(_, color)| *color).unwrap_or_default()
}
