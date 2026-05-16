use crate::interaction::DragAxis;
use crate::widgets::knobs::{ContinuousControl, ResetGesture};
use egui::{Response, Sense, Ui};
use std::ops::RangeInclusive;

pub struct VerticalDrag<'a> {
    value: &'a mut f64,
    range: RangeInclusive<f64>,
    step: f64,
    reset: Option<f64>,
}

impl<'a> VerticalDrag<'a> {
    pub fn new(value: &'a mut f64, range: RangeInclusive<f64>) -> Self {
        Self {
            value,
            range,
            step: 1.0,
            reset: None,
        }
    }
    pub fn step(mut self, step: f64) -> Self {
        self.step = step.abs();
        self
    }
    pub fn reset_value(mut self, value: f64) -> Self {
        self.reset = Some(value);
        self
    }
}

impl<'a> egui::Widget for VerticalDrag<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let resp = ui.allocate_response(egui::Vec2::new(44.0, 20.0), Sense::click_and_drag());
        if resp.double_clicked() {
            if let Some(reset) = self.reset {
                *self.value = reset;
            }
        }
        let mut ctrl = ContinuousControl::new(self.value, self.range.clone())
            .axis(DragAxis::Y)
            .reset_gesture(ResetGesture::MiddleClick)
            .wheel_step(self.step);
        ctrl.handle(ui, &resp);
        resp
    }
}
