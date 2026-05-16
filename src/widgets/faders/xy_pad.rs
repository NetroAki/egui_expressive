use crate::interaction::normalize;
use egui::{Pos2, Response, Sense, Stroke, Ui, Vec2};
use std::ops::RangeInclusive;

/// Two-dimensional parameter pad with normalized X/Y outputs.
pub struct XYPad<'a> {
    x: &'a mut f64,
    y: &'a mut f64,
    x_range: RangeInclusive<f64>,
    y_range: RangeInclusive<f64>,
    size: Vec2,
    label: Option<String>,
}

impl<'a> XYPad<'a> {
    pub fn new(
        x: &'a mut f64,
        y: &'a mut f64,
        x_range: RangeInclusive<f64>,
        y_range: RangeInclusive<f64>,
    ) -> Self {
        Self {
            x,
            y,
            x_range,
            y_range,
            size: Vec2::splat(120.0),
            label: None,
        }
    }
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl<'a> egui::Widget for XYPad<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::click_and_drag());
        if (response.dragged() || response.clicked()) && response.interact_pointer_pos().is_some() {
            let pos = response.interact_pointer_pos().unwrap();
            let tx = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0) as f64;
            let ty = (1.0 - (pos.y - rect.min.y) / rect.height()).clamp(0.0, 1.0) as f64;
            *self.x = *self.x_range.start() + (*self.x_range.end() - *self.x_range.start()) * tx;
            *self.y = *self.y_range.start() + (*self.y_range.end() - *self.y_range.start()) * ty;
        }
        let visuals = ui.visuals();
        ui.painter()
            .rect_filled(rect, 6.0, visuals.extreme_bg_color);
        ui.painter().rect_stroke(
            rect,
            6.0,
            Stroke::new(1.0, visuals.widgets.inactive.bg_stroke.color),
            egui::StrokeKind::Inside,
        );
        let tx = normalize(*self.x, &self.x_range);
        let ty = normalize(*self.y, &self.y_range);
        let pos = Pos2::new(
            rect.min.x + tx * rect.width(),
            rect.max.y - ty * rect.height(),
        );
        ui.painter().line_segment(
            [Pos2::new(pos.x, rect.min.y), Pos2::new(pos.x, rect.max.y)],
            Stroke::new(1.0, visuals.widgets.inactive.bg_stroke.color),
        );
        ui.painter().line_segment(
            [Pos2::new(rect.min.x, pos.y), Pos2::new(rect.max.x, pos.y)],
            Stroke::new(1.0, visuals.widgets.inactive.bg_stroke.color),
        );
        ui.painter()
            .circle_filled(pos, 5.0, visuals.selection.bg_fill);
        if let Some(label) = &self.label {
            ui.painter().text(
                rect.left_top() + Vec2::new(6.0, 4.0),
                egui::Align2::LEFT_TOP,
                label,
                egui::FontId::proportional(10.0),
                visuals.weak_text_color(),
            );
        }
        response
    }
}
