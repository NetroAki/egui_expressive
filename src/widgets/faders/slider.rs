use super::fader::Fader;
use crate::widgets::knobs::Orientation;
use egui::{Response, Ui, Vec2};
use std::ops::RangeInclusive;

/// Horizontal fader convenience wrapper.
pub struct Slider<'a> {
    inner: Fader<'a>,
}

impl<'a> Slider<'a> {
    pub fn new(value: &'a mut f64, range: RangeInclusive<f64>) -> Self {
        Self {
            inner: Fader::new(value, range)
                .orientation(Orientation::Horizontal)
                .size(Vec2::new(140.0, 24.0)),
        }
    }
    pub fn size(mut self, size: Vec2) -> Self {
        self.inner = self.inner.size(size);
        self
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.inner = self.inner.label(label);
        self
    }
    pub fn default_value(mut self, value: f64) -> Self {
        self.inner = self.inner.default_value(value);
        self
    }
    pub fn marks(mut self, marks: impl Into<Vec<f64>>) -> Self {
        self.inner = self.inner.marks(marks);
        self
    }
    pub fn value_popup(mut self, enabled: bool) -> Self {
        self.inner = self.inner.value_popup(enabled);
        self
    }
}

impl<'a> egui::Widget for Slider<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.inner.ui(ui)
    }
}
