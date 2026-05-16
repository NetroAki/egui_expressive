use crate::interaction::DragAxis;
use egui::{Pos2, Rect, Response, Sense, Ui, Vec2};
use std::ops::RangeInclusive;

use super::{
    render::{paint_knob_flat, paint_knob_notched, paint_knob_ring},
    ContinuousControl, KnobSize, KnobStyle, ResetGesture,
};

/// Rotary knob widget for continuous values.
pub struct Knob<'a> {
    value: &'a mut f64,
    range: RangeInclusive<f64>,
    size: f32,
    label: Option<String>,
    default_value: Option<f64>,
    knob_style: KnobStyle,
    knob_size_preset: Option<KnobSize>,
    bipolar: bool,
    wheel_step: Option<f64>,
    value_popup: bool,
}

impl<'a> Knob<'a> {
    /// Create a new Knob.
    pub fn new(value: &'a mut f64, range: RangeInclusive<f64>) -> Self {
        Self {
            value,
            range,
            size: 48.0,
            label: None,
            default_value: None,
            knob_style: KnobStyle::Default,
            knob_size_preset: None,
            bipolar: false,
            wheel_step: None,
            value_popup: false,
        }
    }

    /// Set the knob size (square dimension).
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Set the label shown below the knob.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the default value (reset on double-click).
    pub fn default_value(mut self, v: f64) -> Self {
        self.default_value = Some(v);
        self
    }

    pub fn style(mut self, style: KnobStyle) -> Self {
        self.knob_style = style;
        self
    }

    pub fn preset_size(mut self, size: KnobSize) -> Self {
        self.knob_size_preset = Some(size);
        self
    }

    /// Enable bipolar rendering around a center detent (0.5 normalized).
    pub fn bipolar(mut self, bipolar: bool) -> Self {
        self.bipolar = bipolar;
        self
    }

    /// Enable scroll-wheel editing with the given step in value units.
    pub fn wheel_step(mut self, step: f64) -> Self {
        self.wheel_step = Some(step.abs());
        self
    }

    /// Show the current value as hover text while interacting.
    pub fn value_popup(mut self, enabled: bool) -> Self {
        self.value_popup = enabled;
        self
    }
}

impl<'a> egui::Widget for Knob<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut diameter = self.size;
        if let Some(preset) = self.knob_size_preset {
            diameter = preset.to_px();
        }
        let size = Vec2::splat(diameter);
        let response = ui.allocate_rect(
            Rect::from_center_size(ui.cursor().center(), size),
            Sense::click_and_drag(),
        );

        let t = {
            let mut ctrl = ContinuousControl::new(self.value, self.range.clone())
                .axis(DragAxis::Y)
                .reset_gesture(ResetGesture::MiddleClick);
            if let Some(dv) = self.default_value {
                ctrl = ctrl.default_value(dv);
            }
            if let Some(step) = self.wheel_step {
                ctrl = ctrl.wheel_step(step);
            }
            ctrl.handle(ui, &response)
        };

        let rect = response.rect;
        let center = rect.center();
        let radius = diameter / 2.0 - 2.0;

        let painter = ui.painter();

        let visuals = ui.visuals();
        let track_color = visuals.widgets.inactive.bg_stroke.color;
        let value_color = visuals.selection.stroke.color;
        let label_color = visuals.text_color();

        match self.knob_style {
            KnobStyle::Default => {
                super::render::paint_knob_default(
                    painter,
                    center,
                    radius,
                    t,
                    track_color,
                    value_color,
                    self.bipolar,
                );
            }
            KnobStyle::Flat => {
                paint_knob_flat(painter, center, radius, t, track_color, value_color);
            }
            KnobStyle::Ring => {
                paint_knob_ring(painter, center, radius, t, track_color, value_color);
            }
            KnobStyle::Notched => {
                paint_knob_notched(painter, center, radius, t, track_color, value_color, 13);
            }
        }

        if let Some(label) = &self.label {
            let label_pos = Pos2::new(center.x, rect.max.y - 2.0);
            painter.text(
                label_pos,
                egui::Align2::CENTER_CENTER,
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
