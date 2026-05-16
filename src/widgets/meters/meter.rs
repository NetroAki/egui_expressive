use crate::widgets::knobs::Orientation;
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

use super::{MeterBallistics, MeterMode};

/// Audio level meter widget.
#[derive(Debug, Clone)]
pub struct Meter {
    value: f32,
    peak: Option<f32>,
    mode: MeterMode,
    orientation: Orientation,
    size: Vec2,
    segments: u32,
    clip_threshold: f32,
    channels: usize,
    ballistics: MeterBallistics,
    low_color: Color32,
    mid_color: Color32,
    high_color: Color32,
    label: Option<String>,
}

impl Meter {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            peak: None,
            mode: MeterMode::Peak,
            orientation: Orientation::Vertical,
            size: Vec2::new(16.0, 80.0),
            segments: 0,
            clip_threshold: 0.9,
            channels: 1,
            ballistics: MeterBallistics::default(),
            low_color: Color32::from_rgb(60, 180, 80),
            mid_color: Color32::from_rgb(220, 180, 0),
            high_color: Color32::from_rgb(220, 50, 50),
            label: None,
        }
    }
    pub fn mode(mut self, mode: MeterMode) -> Self {
        self.mode = mode;
        self
    }
    pub fn peak(mut self, peak: f32) -> Self {
        self.peak = Some(peak);
        self
    }
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
    pub fn orientation(mut self, o: Orientation) -> Self {
        self.orientation = o;
        self
    }
    pub fn segments(mut self, n: u32) -> Self {
        self.segments = n;
        self
    }
    pub fn clip_threshold(mut self, t: f32) -> Self {
        self.clip_threshold = t;
        self
    }
    pub fn channels(mut self, channels: usize) -> Self {
        self.channels = channels.max(1);
        self
    }
    pub fn ballistics(mut self, ballistics: MeterBallistics) -> Self {
        self.ballistics = ballistics;
        self
    }
    pub fn gradient(mut self, low: Color32, mid: Color32, high: Color32) -> Self {
        self.low_color = low;
        self.mid_color = mid;
        self.high_color = high;
        self
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
    fn level_color(&self, level: f32) -> Color32 {
        if level < 0.7 {
            self.low_color
        } else if level < self.clip_threshold {
            self.mid_color
        } else {
            self.high_color
        }
    }
}

impl egui::Widget for Meter {
    fn ui(self, ui: &mut Ui) -> Response {
        let response = ui.allocate_rect(
            Rect::from_min_size(ui.cursor().min, self.size),
            Sense::hover(),
        );
        let rect = response.rect;
        let painter = ui.painter();
        painter.rect_filled(rect, 1.0, ui.visuals().extreme_bg_color);
        let label_height = if self.label.is_some() { 12.0 } else { 0.0 };
        let meter_rect = if label_height > 0.0 && self.orientation == Orientation::Vertical {
            Rect::from_min_max(rect.min, Pos2::new(rect.max.x, rect.max.y - label_height))
        } else {
            rect
        };
        let t = self.value.clamp(0.0, 1.0);
        if self.segments == 0 {
            let fill_rect = match self.orientation {
                Orientation::Vertical => {
                    let height = meter_rect.height() * t;
                    Rect::from_min_max(
                        Pos2::new(meter_rect.min.x + 1.0, meter_rect.max.y - height),
                        Pos2::new(meter_rect.max.x - 1.0, meter_rect.max.y - 1.0),
                    )
                }
                Orientation::Horizontal => {
                    let width = meter_rect.width() * t;
                    Rect::from_min_max(
                        Pos2::new(meter_rect.min.x + 1.0, meter_rect.min.y + 1.0),
                        Pos2::new(meter_rect.min.x + width, meter_rect.max.y - 1.0),
                    )
                }
            };
            painter.rect_filled(fill_rect, 0.0, self.level_color(t));
        } else {
            let seg_count = self.segments as f32;
            let seg_gap = 2.0;
            match self.orientation {
                Orientation::Vertical => {
                    let seg_height =
                        (meter_rect.height() - (seg_count - 1.0) * seg_gap) / seg_count;
                    let filled_segs = (t * seg_count).floor() as i32;
                    for i in 0..self.segments as i32 {
                        if i < filled_segs {
                            let y_top =
                                meter_rect.max.y - (i as f32) * (seg_height + seg_gap) - seg_height;
                            let seg_rect = Rect::from_min_size(
                                Pos2::new(meter_rect.min.x + 1.0, y_top),
                                Vec2::new(meter_rect.width() - 2.0, seg_height),
                            );
                            let seg_t = (i as f32 + 1.0) / seg_count;
                            painter.rect_filled(seg_rect, 0.0, self.level_color(seg_t));
                        }
                    }
                }
                Orientation::Horizontal => {
                    let seg_width = (meter_rect.width() - (seg_count - 1.0) * seg_gap) / seg_count;
                    let filled_segs = (t * seg_count).floor() as i32;
                    for i in 0..self.segments as i32 {
                        if i < filled_segs {
                            let x_left = meter_rect.min.x + (i as f32) * (seg_width + seg_gap);
                            let seg_rect = Rect::from_min_size(
                                Pos2::new(x_left, meter_rect.min.y + 1.0),
                                Vec2::new(seg_width, meter_rect.height() - 2.0),
                            );
                            let seg_t = (i as f32 + 1.0) / seg_count;
                            painter.rect_filled(seg_rect, 0.0, self.level_color(seg_t));
                        }
                    }
                }
            }
        }
        if let Some(peak) = self.peak {
            let peak_t = peak.clamp(0.0, 1.0);
            let peak_color = Color32::from_rgb(255, 80, 80);
            match self.orientation {
                Orientation::Vertical => {
                    let peak_y = meter_rect.max.y - peak_t * meter_rect.height();
                    if peak_y > meter_rect.min.y {
                        painter.line_segment(
                            [
                                Pos2::new(meter_rect.min.x, peak_y),
                                Pos2::new(meter_rect.max.x, peak_y),
                            ],
                            Stroke::new(1.5, peak_color),
                        );
                    }
                }
                Orientation::Horizontal => {
                    let peak_x = meter_rect.min.x + peak_t * meter_rect.width();
                    if peak_x < meter_rect.max.x {
                        painter.line_segment(
                            [
                                Pos2::new(peak_x, meter_rect.min.y),
                                Pos2::new(peak_x, meter_rect.max.y),
                            ],
                            Stroke::new(1.5, peak_color),
                        );
                    }
                }
            }
        }
        if let Some(label) = &self.label {
            painter.text(
                Pos2::new(rect.center().x, rect.max.y - 1.0),
                egui::Align2::CENTER_BOTTOM,
                label,
                egui::FontId::proportional(9.0),
                ui.visuals().weak_text_color(),
            );
        }
        response
    }
}
