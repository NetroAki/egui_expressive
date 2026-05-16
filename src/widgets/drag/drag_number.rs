use crate::widgets::drag::vertical_drag::VerticalDrag;
use egui::{Response, Ui};
use std::ops::RangeInclusive;

pub struct DragNumber<'a> {
    value: &'a mut f64,
    range: RangeInclusive<f64>,
    label: Option<String>,
    reset: Option<f64>,
}

impl<'a> DragNumber<'a> {
    pub fn new(value: &'a mut f64, range: RangeInclusive<f64>) -> Self {
        Self {
            value,
            range,
            label: None,
            reset: None,
        }
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn reset_value(mut self, v: f64) -> Self {
        self.reset = Some(v);
        self
    }
}

impl<'a> egui::Widget for DragNumber<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let reset_value = self.reset.unwrap_or(*self.value);
        let inner = ui.vertical(|ui| {
            if let Some(label) = &self.label {
                ui.label(label);
            }
            ui.add(VerticalDrag::new(self.value, self.range).reset_value(reset_value))
        });
        inner.inner
    }
}
