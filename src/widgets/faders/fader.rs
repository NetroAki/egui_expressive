use crate::interaction::DragAxis;
use crate::widgets::faders::render::draw_meter_in_track;
use crate::widgets::knobs::{ContinuousControl, Orientation, ResetGesture};
use egui::{Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};
use std::ops::RangeInclusive;

/// Linear slider / fader widget.
pub struct Fader<'a> {
    value: &'a mut f64,
    range: RangeInclusive<f64>,
    size: Vec2,
    orientation: Orientation,
    label: Option<String>,
    meter_value: Option<f32>,
    meter_value_r: Option<f32>,
    meter_segmented: bool,
    default_value: Option<f64>,
    marks: Vec<f64>,
    value_popup: bool,
}

impl<'a> Fader<'a> {
    pub fn new(value: &'a mut f64, range: RangeInclusive<f64>) -> Self {
        Self {
            value,
            range,
            size: Vec2::new(40.0, 120.0),
            orientation: Orientation::Vertical,
            label: None,
            meter_value: None,
            meter_value_r: None,
            meter_segmented: false,
            default_value: None,
            marks: Vec::new(),
            value_popup: false,
        }
    }
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
    pub fn orientation(mut self, o: Orientation) -> Self {
        self.orientation = o;
        self
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn meter_value(mut self, v: f32) -> Self {
        self.meter_value = Some(v.clamp(0.0, 1.0));
        self
    }
    pub fn stereo_meter(mut self, l: f32, r: f32) -> Self {
        self.meter_value = Some(l.clamp(0.0, 1.0));
        self.meter_value_r = Some(r.clamp(0.0, 1.0));
        self
    }
    pub fn meter_segmented(mut self, segmented: bool) -> Self {
        self.meter_segmented = segmented;
        self
    }
    pub fn default_value(mut self, v: f64) -> Self {
        self.default_value = Some(v);
        self
    }
    pub fn marks(mut self, marks: impl Into<Vec<f64>>) -> Self {
        self.marks = marks.into();
        self
    }
    pub fn value_popup(mut self, enabled: bool) -> Self {
        self.value_popup = enabled;
        self
    }
}

impl<'a> egui::Widget for Fader<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let response = ui.allocate_rect(
            Rect::from_min_size(ui.cursor().min, self.size),
            Sense::click_and_drag(),
        );
        let rect = response.rect;
        let painter = ui.painter();
        let hovered = response.hovered();
        let visuals = ui.visuals();
        let track_bg = visuals.widgets.inactive.bg_fill;
        let thumb_normal = visuals.widgets.inactive.bg_fill.gamma_multiply(1.3);
        let thumb_hovered = visuals.widgets.hovered.bg_fill;
        let label_color = visuals.text_color();

        let track_width = 6.0;
        let (thumb_width, thumb_height) = match self.orientation {
            Orientation::Vertical => (self.size.x - 4.0, track_width + 4.0),
            Orientation::Horizontal => (track_width + 4.0, self.size.y - 4.0),
        };

        let track_rect = match self.orientation {
            Orientation::Vertical => Rect::from_min_max(
                Pos2::new(rect.center().x - track_width / 2.0, rect.min.y + 4.0),
                Pos2::new(rect.center().x + track_width / 2.0, rect.max.y - 4.0),
            ),
            Orientation::Horizontal => Rect::from_min_max(
                Pos2::new(rect.min.x + 4.0, rect.center().y - track_width / 2.0),
                Pos2::new(rect.max.x - 4.0, rect.center().y + track_width / 2.0),
            ),
        };
        painter.rect_filled(track_rect, 2.0, track_bg);

        let min = *self.range.start();
        let max = *self.range.end();
        let span = (max - min).max(f64::EPSILON);
        for mark in &self.marks {
            let mt = ((*mark - min) / span).clamp(0.0, 1.0) as f32;
            match self.orientation {
                Orientation::Vertical => {
                    let y = track_rect.max.y - mt * track_rect.height();
                    painter.line_segment(
                        [
                            Pos2::new(track_rect.min.x - 4.0, y),
                            Pos2::new(track_rect.max.x + 4.0, y),
                        ],
                        Stroke::new(1.0, visuals.widgets.inactive.bg_stroke.color),
                    );
                }
                Orientation::Horizontal => {
                    let x = track_rect.min.x + mt * track_rect.width();
                    painter.line_segment(
                        [
                            Pos2::new(x, track_rect.min.y - 4.0),
                            Pos2::new(x, track_rect.max.y + 4.0),
                        ],
                        Stroke::new(1.0, visuals.widgets.inactive.bg_stroke.color),
                    );
                }
            }
        }

        if let Some(level) = self.meter_value {
            if let Some(level_r) = self.meter_value_r {
                let left_track = Rect::from_min_max(
                    track_rect.min,
                    Pos2::new(track_rect.center().x, track_rect.max.y),
                );
                let right_track = Rect::from_min_max(
                    Pos2::new(track_rect.center().x, track_rect.min.y),
                    track_rect.max,
                );
                draw_meter_in_track(
                    painter,
                    left_track,
                    level,
                    self.meter_segmented,
                    self.orientation,
                );
                draw_meter_in_track(
                    painter,
                    right_track,
                    level_r,
                    self.meter_segmented,
                    self.orientation,
                );
            } else {
                draw_meter_in_track(
                    painter,
                    track_rect,
                    level,
                    self.meter_segmented,
                    self.orientation,
                );
            }
        }

        let axis = match self.orientation {
            Orientation::Vertical => DragAxis::Y,
            Orientation::Horizontal => DragAxis::X,
        };
        let track_len = match self.orientation {
            Orientation::Vertical => rect.height() - 8.0 - thumb_height,
            Orientation::Horizontal => rect.width() - 8.0 - thumb_width,
        };
        let t = {
            let mut ctrl = ContinuousControl::new(self.value, self.range.clone())
                .axis(axis)
                .sensitivity(track_len.max(1.0))
                .reset_gesture(ResetGesture::MiddleClick);
            if let Some(default) = self.default_value {
                ctrl = ctrl.default_value(default);
            }
            ctrl.handle(ui, &response)
        };

        match self.orientation {
            Orientation::Vertical => {
                let thumb_y =
                    rect.max.y - 4.0 - thumb_height - t * (rect.height() - 8.0 - thumb_height);
                let thumb_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x + 2.0, thumb_y),
                    Vec2::new(thumb_width, thumb_height),
                );
                painter.rect_filled(
                    thumb_rect,
                    2.0,
                    if hovered { thumb_hovered } else { thumb_normal },
                );
            }
            Orientation::Horizontal => {
                let thumb_x = rect.min.x + 4.0 + t * (rect.width() - 8.0 - thumb_width);
                let thumb_rect = Rect::from_min_size(
                    Pos2::new(thumb_x, rect.min.y + 2.0),
                    Vec2::new(thumb_width, thumb_height),
                );
                painter.rect_filled(
                    thumb_rect,
                    2.0,
                    if hovered { thumb_hovered } else { thumb_normal },
                );
            }
        }

        if let Some(label) = &self.label {
            let label_pos = match self.orientation {
                Orientation::Vertical => Pos2::new(rect.max.x + 4.0, rect.center().y),
                Orientation::Horizontal => Pos2::new(rect.center().x, rect.min.y - 4.0),
            };
            painter.text(
                label_pos,
                egui::Align2::LEFT_CENTER,
                label,
                egui::FontId::proportional(10.0),
                label_color,
            );
        }

        if self.value_popup && (response.hovered() || response.dragged()) {
            response.on_hover_text(format!("{:.3}", *self.value))
        } else {
            response
        }
    }
}
