use egui::{Color32, Painter, Pos2, Shape, Stroke, Vec2};
use std::f32::consts::PI;

pub(super) fn paint_knob_default(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    normalized: f32,
    track_color: Color32,
    value_color: Color32,
    bipolar: bool,
) {
    painter.circle_filled(center, radius, value_color);

    let min_angle = 225f32.to_radians();
    let sweep = 270f32.to_radians();
    let value_angle = min_angle + normalized * sweep;

    let track_points: Vec<Pos2> = (0..=64)
        .map(|i| {
            let angle = min_angle + (sweep * i as f32) / 64.0;
            center + Vec2::angled(angle) * radius
        })
        .collect();
    painter.add(Shape::line(track_points, Stroke::new(2.0, track_color)));

    let arc_start = if bipolar {
        min_angle + sweep * 0.5
    } else {
        min_angle
    };
    let arc_end = value_angle;
    if (arc_end - arc_start).abs() > f32::EPSILON {
        let lo = arc_start.min(arc_end);
        let hi = arc_start.max(arc_end);
        let value_arc_points: Vec<Pos2> = (0..=32)
            .map(|i| {
                let angle = lo + (hi - lo) * (i as f32) / 32.0;
                center + Vec2::angled(angle) * radius
            })
            .collect();
        painter.add(Shape::line(value_arc_points, Stroke::new(3.0, value_color)));
    }
    if bipolar {
        let detent_angle = min_angle + sweep * 0.5;
        let detent = center + Vec2::angled(detent_angle) * (radius - 4.0);
        painter.circle_filled(detent, 2.0, value_color);
    }

    let indicator_inner = radius * 0.3;
    let indicator_outer = radius * 0.75;
    let line_start = center + Vec2::angled(value_angle) * indicator_inner;
    let line_end = center + Vec2::angled(value_angle) * indicator_outer;
    painter.add(Shape::LineSegment {
        points: [line_start, line_end],
        stroke: Stroke::new(2.5, Color32::WHITE),
    });
}

pub(super) fn paint_knob_flat(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    normalized: f32,
    _track_color: Color32,
    value_color: Color32,
) {
    painter.circle_filled(center, radius, value_color);
    let angle = PI * 0.75 + normalized * PI * 1.5;
    let inner = Pos2::new(
        center.x + angle.cos() * radius * 0.3,
        center.y + angle.sin() * radius * 0.3,
    );
    let outer = Pos2::new(
        center.x + angle.cos() * radius * 0.85,
        center.y + angle.sin() * radius * 0.85,
    );
    painter.line_segment([inner, outer], Stroke::new(2.0, Color32::WHITE));
}

pub(super) fn paint_knob_ring(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    normalized: f32,
    track_color: Color32,
    value_color: Color32,
) {
    painter.circle_stroke(center, radius * 0.9, Stroke::new(2.0, track_color));
    let start_angle = PI * 0.75;
    let sweep = normalized * PI * 1.5;
    let points: Vec<Pos2> = (0..=20)
        .map(|i| {
            let a = start_angle + sweep * (i as f32 / 20.0);
            Pos2::new(
                center.x + a.cos() * radius * 0.9,
                center.y + a.sin() * radius * 0.9,
            )
        })
        .collect();
    if points.len() >= 2 {
        painter.add(Shape::line(points, Stroke::new(2.5, value_color)));
    }
    let angle = start_angle + sweep;
    let dot = Pos2::new(
        center.x + angle.cos() * radius * 0.9,
        center.y + angle.sin() * radius * 0.9,
    );
    painter.circle_filled(dot, 3.0, value_color);
}

pub(super) fn paint_knob_notched(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    normalized: f32,
    track_color: Color32,
    value_color: Color32,
    ticks: usize,
) {
    let start_angle = PI * 0.75;
    let total_sweep = PI * 1.5;
    let active_angle = start_angle + normalized * total_sweep;

    if ticks >= 2 {
        for i in 0..ticks {
            let t = i as f32 / (ticks - 1) as f32;
            let angle = start_angle + t * total_sweep;
            let color = if angle <= active_angle {
                value_color
            } else {
                track_color
            };
            let inner = Pos2::new(
                center.x + angle.cos() * radius * 0.65,
                center.y + angle.sin() * radius * 0.65,
            );
            let outer = Pos2::new(
                center.x + angle.cos() * radius * 0.9,
                center.y + angle.sin() * radius * 0.9,
            );
            painter.line_segment([inner, outer], Stroke::new(2.0, color));
        }
    }
    painter.circle_filled(center, radius * 0.15, value_color);
}
