use super::*;

/// Build a bilinear gradient mesh patch as an `egui::Shape::Mesh`.
pub fn mesh_gradient_patch(
    corners: [egui::Pos2; 4],
    colors: [egui::Color32; 4],
    subdivisions: usize,
) -> egui::Shape {
    use egui::epaint::Mesh;

    let subdivisions = subdivisions.clamp(1, 64);
    let stride = subdivisions + 1;
    let mut mesh = Mesh::default();

    for y in 0..=subdivisions {
        let v = y as f32 / subdivisions as f32;
        for x in 0..=subdivisions {
            let u = x as f32 / subdivisions as f32;
            mesh.colored_vertex(bilerp_pos(corners, u, v), bilerp_color(colors, u, v));
        }
    }

    for y in 0..subdivisions {
        for x in 0..subdivisions {
            let a = (y * stride + x) as u32;
            let b = a + 1;
            let c = ((y + 1) * stride + x) as u32;
            let d = c + 1;
            mesh.add_triangle(a, b, c);
            mesh.add_triangle(b, d, c);
        }
    }

    egui::Shape::Mesh(mesh.into())
}

/// Build an editable procedural pattern fill clipped to a sampled path.
///
/// Illustrator pattern swatch internals are not exposed consistently through CEP/UXP, so exporters
/// pass the swatch name as a stable seed and keep the result as vector shapes instead of rasterizing
/// the artboard. The returned shapes are deterministic, editable, and clipped by point sampling to
/// the provided polygon.
pub fn pattern_fill_path(
    points: &[egui::Pos2],
    seed: u32,
    foreground: egui::Color32,
    background: egui::Color32,
    cell_size: f32,
    mark_size: f32,
) -> Vec<egui::Shape> {
    if points.len() < 3 {
        return Vec::new();
    }

    let mut polygon = points.to_vec();
    if polygon.len() > 3 && polygon.first() == polygon.last() {
        polygon.pop();
    }
    if polygon.len() < 3 {
        return Vec::new();
    }

    let bounds = bounds_for_points(&polygon);
    if bounds.is_negative() || bounds.width() <= 0.0 || bounds.height() <= 0.0 {
        return Vec::new();
    }

    let cell_size = cell_size.max(2.0);
    let mark_size = mark_size.clamp(0.5, cell_size * 0.45);
    let stroke = Stroke::new(mark_size.max(0.5), foreground);
    let cols = (bounds.width() / cell_size).ceil() as u32 + 1;
    let rows = (bounds.height() / cell_size).ceil() as u32 + 1;
    let mut shapes = Vec::with_capacity((cols * rows) as usize + 1);

    if background != egui::Color32::TRANSPARENT {
        shapes.push(egui::Shape::Path(PathShape {
            points: polygon.clone(),
            closed: true,
            fill: background,
            stroke: PathStroke::NONE,
        }));
    }

    for row in 0..rows {
        for col in 0..cols {
            let jitter = hash_noise(seed, col, row) as f32 / 255.0 - 0.5;
            let center = egui::pos2(
                bounds.min.x + (col as f32 + 0.5 + jitter * 0.18) * cell_size,
                bounds.min.y + (row as f32 + 0.5 - jitter * 0.18) * cell_size,
            );
            if !point_in_polygon(center, &polygon) {
                continue;
            }

            let half = cell_size * 0.28;
            match (seed.wrapping_add(row).wrapping_add(col)) % 3 {
                0 => {
                    let a = center + egui::vec2(-half, -half);
                    let b = center + egui::vec2(half, half);
                    if point_in_polygon(a, &polygon) && point_in_polygon(b, &polygon) {
                        shapes.push(egui::Shape::line_segment([a, b], stroke));
                    }
                }
                1 => {
                    let a = center + egui::vec2(-half, half);
                    let b = center + egui::vec2(half, -half);
                    if point_in_polygon(a, &polygon) && point_in_polygon(b, &polygon) {
                        shapes.push(egui::Shape::line_segment([a, b], stroke));
                    }
                }
                _ => {
                    let radius = mark_size.max(1.0);
                    let inside = [
                        center + egui::vec2(radius, 0.0),
                        center + egui::vec2(-radius, 0.0),
                        center + egui::vec2(0.0, radius),
                        center + egui::vec2(0.0, -radius),
                    ]
                    .into_iter()
                    .all(|point| point_in_polygon(point, &polygon));
                    if inside {
                        shapes.push(egui::Shape::circle_filled(center, radius, foreground));
                    }
                }
            }
        }
    }

    shapes
}

pub(crate) fn bilerp_pos(corners: [egui::Pos2; 4], u: f32, v: f32) -> egui::Pos2 {
    let top = egui::pos2(
        corners[0].x + (corners[1].x - corners[0].x) * u,
        corners[0].y + (corners[1].y - corners[0].y) * u,
    );
    let bottom = egui::pos2(
        corners[3].x + (corners[2].x - corners[3].x) * u,
        corners[3].y + (corners[2].y - corners[3].y) * u,
    );
    egui::pos2(
        top.x + (bottom.x - top.x) * v,
        top.y + (bottom.y - top.y) * v,
    )
}

pub(crate) fn bilerp_color(colors: [egui::Color32; 4], u: f32, v: f32) -> egui::Color32 {
    let [tl, tr, br, bl] = colors.map(|c| c.to_srgba_unmultiplied());
    let channel = |idx: usize| {
        let top = tl[idx] as f32 + (tr[idx] as f32 - tl[idx] as f32) * u;
        let bottom = bl[idx] as f32 + (br[idx] as f32 - bl[idx] as f32) * u;
        (top + (bottom - top) * v).round().clamp(0.0, 255.0) as u8
    };
    egui::Color32::from_rgba_unmultiplied(channel(0), channel(1), channel(2), channel(3))
}

/// Deterministic procedural noise overlay for code-only Illustrator grain/noise effects.
///
/// This emits tiny translucent rectangles instead of loading a raster texture. It is intended as
/// the CPU/immediate-mode fallback for Illustrator appearance stacks; GPU builds can replace the
/// same semantic primitive with a shader pass.
pub fn noise_rect(rect: egui::Rect, seed: u32, cell_size: f32, opacity: f32) -> Vec<egui::Shape> {
    let cell_size = cell_size.max(1.0);
    let opacity = opacity.clamp(0.0, 1.0);
    if rect.is_negative() || opacity <= 0.0 {
        return Vec::new();
    }

    let cols = (rect.width() / cell_size).ceil().max(1.0) as u32;
    let rows = (rect.height() / cell_size).ceil().max(1.0) as u32;
    let mut shapes = Vec::with_capacity((cols * rows) as usize);

    for row in 0..rows {
        for col in 0..cols {
            let x = rect.min.x + col as f32 * cell_size;
            let y = rect.min.y + row as f32 * cell_size;
            let r = egui::Rect::from_min_max(
                egui::pos2(x, y),
                egui::pos2(
                    (x + cell_size).min(rect.max.x),
                    (y + cell_size).min(rect.max.y),
                ),
            );
            let value = hash_noise(seed, col, row);
            let alpha = (value as f32 / 255.0 * opacity * 255.0).round() as u8;
            shapes.push(egui::Shape::rect_filled(
                r,
                0.0,
                egui::Color32::from_white_alpha(alpha),
            ));
        }
    }

    shapes
}

pub(crate) fn hash_noise(seed: u32, x: u32, y: u32) -> u8 {
    let mut n = seed ^ x.wrapping_mul(0x9E37_79B9) ^ y.wrapping_mul(0x85EB_CA6B);
    n ^= n >> 16;
    n = n.wrapping_mul(0x7FEB_352D);
    n ^= n >> 15;
    n = n.wrapping_mul(0x846C_A68B);
    n ^= n >> 16;
    (n & 0xFF) as u8
}
