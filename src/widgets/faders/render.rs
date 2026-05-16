use crate::widgets::knobs::Orientation;
use egui::{Color32, Painter, Rect};

pub(super) fn normalized_pair(a: f64, b: f64, range: &std::ops::RangeInclusive<f64>) -> (f32, f32) {
    let min = *range.start();
    let max = *range.end();
    let span = (max - min).max(f64::EPSILON);
    let a = ((a - min) / span).clamp(0.0, 1.0) as f32;
    let b = ((b - min) / span).clamp(0.0, 1.0) as f32;
    (a.min(b), a.max(b))
}

fn meter_level_color(level: f32) -> Color32 {
    if level > 0.85 {
        Color32::from_rgb(220, 70, 70)
    } else if level > 0.65 {
        Color32::from_rgb(220, 180, 60)
    } else {
        Color32::from_rgb(80, 180, 120)
    }
}

pub(super) fn draw_meter_in_track(
    painter: &Painter,
    track_rect: Rect,
    level: f32,
    segmented: bool,
    orientation: Orientation,
) {
    let meter_color = meter_level_color(level);
    if segmented {
        let segments = 20usize;
        let active = (level * segments as f32) as usize;
        for seg in 0..segments {
            let t_seg = seg as f32 / segments as f32;
            let seg_rect = if orientation == Orientation::Vertical {
                let y_top =
                    track_rect.max.y - track_rect.height() * ((seg + 1) as f32 / segments as f32);
                let y_bot = track_rect.max.y - track_rect.height() * (seg as f32 / segments as f32);
                Rect::from_min_max(
                    egui::Pos2::new(track_rect.min.x + 1.0, y_top + 1.0),
                    egui::Pos2::new(track_rect.max.x - 1.0, y_bot - 1.0),
                )
            } else {
                let x_left = track_rect.min.x + track_rect.width() * (seg as f32 / segments as f32);
                let x_right =
                    track_rect.min.x + track_rect.width() * ((seg + 1) as f32 / segments as f32);
                Rect::from_min_max(
                    egui::Pos2::new(x_left + 1.0, track_rect.min.y + 1.0),
                    egui::Pos2::new(x_right - 1.0, track_rect.max.y - 1.0),
                )
            };
            let seg_color = if seg < active {
                meter_level_color(t_seg)
            } else {
                Color32::from_gray(40)
            };
            painter.rect_filled(seg_rect, egui::CornerRadius::ZERO, seg_color);
        }
    } else {
        let fill_rect = if orientation == Orientation::Vertical {
            let fill_h = track_rect.height() * level;
            Rect::from_min_max(
                egui::Pos2::new(track_rect.min.x + 1.0, track_rect.max.y - fill_h),
                egui::Pos2::new(track_rect.max.x - 1.0, track_rect.max.y),
            )
        } else {
            let fill_w = track_rect.width() * level;
            Rect::from_min_max(
                egui::Pos2::new(track_rect.min.x, track_rect.min.y + 1.0),
                egui::Pos2::new(track_rect.min.x + fill_w, track_rect.max.y - 1.0),
            )
        };
        painter.rect_filled(fill_rect, egui::CornerRadius::ZERO, meter_color);
    }
}
