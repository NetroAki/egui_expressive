use super::*;

/// Stroke cap style.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StrokeCap {
    Butt,
    Round,
    Square,
}

/// Stroke join style.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StrokeJoin {
    Miter,
    Round,
    Bevel,
}

/// Dash pattern for dashed strokes.
#[derive(Clone, Debug)]
pub struct DashPattern {
    pub dashes: Vec<f32>,
    pub offset: f32,
}

/// Rich stroke with support for dashes, caps, and joins.
#[derive(Clone, Debug)]
pub struct RichStroke {
    pub width: f32,
    pub color: egui::Color32,
    pub dash: Option<DashPattern>,
    pub cap: StrokeCap,
    pub join: StrokeJoin,
}

impl RichStroke {
    /// Create a solid (non-dashed) stroke.
    pub fn solid(width: f32, color: egui::Color32) -> Self {
        Self {
            width,
            color,
            dash: None,
            cap: StrokeCap::Butt,
            join: StrokeJoin::Miter,
        }
    }

    /// Create a dashed stroke with equal dash and gap lengths.
    pub fn dashed(width: f32, color: egui::Color32, dash: f32, gap: f32) -> Self {
        Self {
            width,
            color,
            dash: Some(DashPattern {
                dashes: vec![dash, gap],
                offset: 0.0,
            }),
            cap: StrokeCap::Butt,
            join: StrokeJoin::Miter,
        }
    }
}

/// Render a path with a rich stroke (supports dashes).
pub fn dashed_path(painter: &egui::Painter, points: &[Pos2], stroke: &RichStroke) {
    for shape in dashed_path_shapes(points, stroke) {
        painter.add(shape);
    }
}

pub fn dashed_path_shapes(points: &[Pos2], stroke: &RichStroke) -> Vec<egui::Shape> {
    let mut shapes = Vec::new();
    if points.len() < 2 {
        return shapes;
    }
    match &stroke.dash {
        None => {
            draw_dash(&mut shapes, points, stroke);
        }
        Some(pattern) => {
            let total_len: f32 = points.windows(2).map(|w| (w[1] - w[0]).length()).sum();
            if total_len <= 0.0 {
                return shapes;
            }
            let cycle_len: f32 = pattern.dashes.iter().sum();
            if cycle_len <= 0.0 {
                return shapes;
            }

            let mut dist = pattern.offset % cycle_len;
            let mut phase = 0usize;
            let mut drawing = true;

            let mut d = dist;
            while d >= pattern.dashes[phase] {
                d -= pattern.dashes[phase];
                phase = (phase + 1) % pattern.dashes.len();
                drawing = !drawing;
            }
            dist = d;

            let mut current_dash = Vec::new();
            let mut current_pos = points[0];

            if drawing {
                current_dash.push(current_pos);
            }

            for i in 0..points.len() - 1 {
                let seg_vec = points[i + 1] - points[i];
                let seg_len = seg_vec.length();
                if seg_len <= 0.0 {
                    continue;
                }
                let seg_dir = seg_vec / seg_len;

                let mut walked = 0.0f32;
                while walked < seg_len {
                    let remaining_in_phase = pattern.dashes[phase] - dist;
                    let step = remaining_in_phase.min(seg_len - walked);
                    let next_pos = points[i] + seg_dir * (walked + step);

                    current_pos = next_pos;
                    walked += step;
                    dist += step;

                    if dist >= pattern.dashes[phase] {
                        if drawing {
                            current_dash.push(current_pos);
                            draw_dash(&mut shapes, &current_dash, stroke);
                            current_dash.clear();
                        }
                        dist = 0.0;
                        phase = (phase + 1) % pattern.dashes.len();
                        drawing = !drawing;
                        if drawing {
                            current_dash.push(current_pos);
                        }
                    } else if walked >= seg_len && drawing {
                        current_dash.push(current_pos);
                    }
                }
            }
            if drawing && current_dash.len() > 1 {
                draw_dash(&mut shapes, &current_dash, stroke);
            }
        }
    }
    shapes
}

pub(crate) fn draw_dash(shapes: &mut Vec<egui::Shape>, dash_points: &[Pos2], stroke: &RichStroke) {
    if dash_points.len() < 2 {
        return;
    }
    if stroke.join == StrokeJoin::Bevel && dash_points.len() > 2 {
        for pair in dash_points.windows(2) {
            shapes.push(egui::Shape::line_segment(
                [pair[0], pair[1]],
                Stroke::new(stroke.width, stroke.color),
            ));
        }
    } else {
        shapes.push(egui::Shape::line(
            dash_points.to_vec(),
            Stroke::new(stroke.width, stroke.color),
        ));
    }

    if stroke.cap == StrokeCap::Round {
        shapes.push(egui::Shape::circle_filled(
            dash_points[0],
            stroke.width * 0.5,
            stroke.color,
        ));
        shapes.push(egui::Shape::circle_filled(
            *dash_points.last().unwrap(),
            stroke.width * 0.5,
            stroke.color,
        ));
    } else if stroke.cap == StrokeCap::Square {
        let d0 = dash_points[1] - dash_points[0];
        let len0 = d0.length();
        if len0 > 0.0 {
            let dir0 = d0 / len0;
            let p0 = dash_points[0] - dir0 * (stroke.width * 0.5);
            shapes.push(egui::Shape::line_segment(
                [p0, dash_points[0]],
                Stroke::new(stroke.width, stroke.color),
            ));
        }
        let n = dash_points.len();
        let d1 = dash_points[n - 1] - dash_points[n - 2];
        let len1 = d1.length();
        if len1 > 0.0 {
            let dir1 = d1 / len1;
            let p1 = dash_points[n - 1] + dir1 * (stroke.width * 0.5);
            shapes.push(egui::Shape::line_segment(
                [dash_points[n - 1], p1],
                Stroke::new(stroke.width, stroke.color),
            ));
        }
    }

    if stroke.join == StrokeJoin::Round {
        for &p in &dash_points[1..dash_points.len() - 1] {
            shapes.push(egui::Shape::circle_filled(
                p,
                stroke.width * 0.5,
                stroke.color,
            ));
        }
    }
}

// ─── 2D Transform ─────────────────────────────────────────────────────────────

pub fn rounded_rect_path(rect: egui::Rect, rounding: f32) -> Vec<egui::Pos2> {
    let mut points = Vec::new();
    let r = rounding.min(rect.width() * 0.5).min(rect.height() * 0.5);
    if r <= 0.0 {
        return vec![
            rect.min,
            egui::pos2(rect.max.x, rect.min.y),
            rect.max,
            egui::pos2(rect.min.x, rect.max.y),
        ];
    }
    let n = adaptive_arc_segments(r);
    let c = egui::pos2(rect.max.x - r, rect.min.y + r);
    for i in 0..=n {
        let a = std::f32::consts::FRAC_PI_2 * (i as f32 / n as f32);
        points.push(c + egui::vec2(a.sin() * r, -a.cos() * r));
    }
    let c = egui::pos2(rect.max.x - r, rect.max.y - r);
    for i in 0..=n {
        let a = std::f32::consts::FRAC_PI_2 * (i as f32 / n as f32);
        points.push(c + egui::vec2(a.cos() * r, a.sin() * r));
    }
    let c = egui::pos2(rect.min.x + r, rect.max.y - r);
    for i in 0..=n {
        let a = std::f32::consts::FRAC_PI_2 * (i as f32 / n as f32);
        points.push(c + egui::vec2(-a.sin() * r, a.cos() * r));
    }
    let c = egui::pos2(rect.min.x + r, rect.min.y + r);
    for i in 0..=n {
        let a = std::f32::consts::FRAC_PI_2 * (i as f32 / n as f32);
        points.push(c + egui::vec2(-a.cos() * r, -a.sin() * r));
    }
    points
}

pub(crate) fn adaptive_arc_segments(radius: f32) -> usize {
    ((radius * std::f32::consts::FRAC_PI_2) / 3.0)
        .ceil()
        .clamp(8.0, 32.0) as usize
}
