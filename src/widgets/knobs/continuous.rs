use crate::interaction::DragAxis;
use egui::{Response, Ui};
use std::ops::RangeInclusive;

use super::style::ResetGesture;

/// The core interaction primitive for any continuous-value control.
/// Handles drag, normalization, shift-for-fine, and double-click-reset.
/// Use this to build knobs, faders, sliders, or any custom control.
///
/// # Example
/// ```rust,ignore
/// let mut ctrl = ContinuousControl::new(&mut value, 0.0..=1.0);
/// let response = ui.allocate_rect(rect, Sense::drag());
/// let t = ctrl.handle(ui, &response); // returns normalized 0.0..=1.0
/// ```
pub struct ContinuousControl<'a> {
    value: &'a mut f64,
    range: RangeInclusive<f64>,
    /// Pixels of drag to traverse the full range. Default: 200.0
    pub sensitivity: f32,
    /// Multiplier applied when Shift is held. Default: 0.1
    pub fine_multiplier: f32,
    /// Value to reset to on double-click. None = no reset.
    pub default_value: Option<f64>,
    /// Gesture that triggers reset when `default_value` is set.
    pub reset_gesture: ResetGesture,
    /// Optional wheel increment in value units. `None` disables wheel editing.
    pub wheel_step: Option<f64>,
    /// Which axis drives the value. Default: Y (drag up = increase).
    pub axis: DragAxis,
}

impl<'a> ContinuousControl<'a> {
    pub fn new(value: &'a mut f64, range: RangeInclusive<f64>) -> Self {
        Self {
            value,
            range,
            sensitivity: 200.0,
            fine_multiplier: 0.1,
            default_value: None,
            reset_gesture: ResetGesture::DoubleClick,
            wheel_step: None,
            axis: DragAxis::Y,
        }
    }

    pub fn sensitivity(mut self, s: f32) -> Self {
        self.sensitivity = s;
        self
    }
    pub fn fine_multiplier(mut self, m: f32) -> Self {
        self.fine_multiplier = m;
        self
    }
    pub fn default_value(mut self, v: f64) -> Self {
        self.default_value = Some(v);
        self
    }
    pub fn reset_gesture(mut self, gesture: ResetGesture) -> Self {
        self.reset_gesture = gesture;
        self
    }
    pub fn wheel_step(mut self, step: f64) -> Self {
        self.wheel_step = Some(step.abs());
        self
    }
    pub fn axis(mut self, a: DragAxis) -> Self {
        self.axis = a;
        self
    }

    /// Process interaction for the given response. Call after `ui.allocate_rect()`.
    /// Returns the normalized value (0.0..=1.0).
    pub fn handle(&mut self, ui: &Ui, response: &Response) -> f32 {
        let min = *self.range.start();
        let max = *self.range.end();
        let range_span = (max - min) as f32;

        if response.dragged() {
            let raw_delta = match self.axis {
                DragAxis::Y => -response.drag_delta().y,
                DragAxis::X => response.drag_delta().x,
                DragAxis::Free => {
                    let d = response.drag_delta();
                    if d.x.abs() > d.y.abs() {
                        d.x
                    } else {
                        -d.y
                    }
                }
            };
            let multiplier = if ui.input(|i| i.modifiers.shift) {
                self.fine_multiplier
            } else {
                1.0
            };
            let speed = range_span / self.sensitivity;
            let delta = (raw_delta as f64) * speed as f64 * multiplier as f64;
            *self.value = (*self.value + delta).clamp(min, max);
        }

        if response.hovered() {
            if let Some(step) = self.wheel_step {
                let scroll_y = ui.input(|i| i.smooth_scroll_delta.y);
                if scroll_y.abs() > f32::EPSILON {
                    let fine = if ui.input(|i| i.modifiers.shift) {
                        self.fine_multiplier as f64
                    } else {
                        1.0
                    };
                    *self.value =
                        (*self.value + scroll_y.signum() as f64 * step * fine).clamp(min, max);
                }
            }
        }

        let reset = match self.reset_gesture {
            ResetGesture::None => false,
            ResetGesture::DoubleClick => response.double_clicked(),
            ResetGesture::MiddleClick => response.clicked_by(egui::PointerButton::Middle),
            ResetGesture::SecondaryClick => response.secondary_clicked(),
        };
        if reset {
            if let Some(default) = self.default_value {
                *self.value = default.clamp(min, max);
            }
        }

        self.normalized()
    }

    /// Returns the current value normalized to 0.0..=1.0.
    pub fn normalized(&self) -> f32 {
        let min = *self.range.start();
        let max = *self.range.end();
        ((*self.value - min) / (max - min)).clamp(0.0, 1.0) as f32
    }
}
