use crate::{
    accessibility::{AccessibilityMeta, AccessibilityRole},
    interaction::{FeedbackSeverity, FeedbackToast},
};

use egui::{Context, Response, Ui};

#[derive(Clone, Debug)]
pub struct Toast {
    pub message: String,
    pub seconds_left: f32,
    pub accessibility: Option<AccessibilityMeta>,
}

impl Toast {
    pub fn new(message: impl Into<String>, seconds_left: f32) -> Self {
        Self {
            message: message.into(),
            seconds_left,
            accessibility: None,
        }
    }

    pub fn from_feedback(toast: &FeedbackToast) -> Self {
        let role = match toast.message.severity {
            FeedbackSeverity::Warning | FeedbackSeverity::Error => AccessibilityRole::Alert,
            FeedbackSeverity::Info | FeedbackSeverity::Success => AccessibilityRole::Status,
        };
        Self::new(toast.message.message.clone(), toast.seconds_left)
            .accessibility(toast.message.accessibility_meta(role))
    }

    pub fn accessibility(mut self, meta: AccessibilityMeta) -> Self {
        self.accessibility = Some(meta);
        self
    }
}

pub struct ToastLayer<'a> {
    pub toasts: &'a mut Vec<Toast>,
}
impl<'a> ToastLayer<'a> {
    pub fn new(toasts: &'a mut Vec<Toast>) -> Self {
        Self { toasts }
    }
    /// Render toasts as a floating overlay and request repaint while visible.
    pub fn show(mut self, ctx: &Context) {
        self.advance(ctx.input(|i| i.stable_dt));
        if self.toasts.is_empty() {
            return;
        }

        egui::Area::new(egui::Id::new("neutra_toast_layer"))
            .order(egui::Order::Tooltip)
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-16.0, 16.0))
            .show(ctx, |ui| {
                ui.vertical(|ui| self.paint(ui));
            });
        ctx.request_repaint();
    }

    fn advance(&mut self, seconds: f32) {
        for toast in self.toasts.iter_mut() {
            toast.seconds_left -= seconds;
        }
        self.toasts.retain(|toast| toast.seconds_left > 0.0);
    }

    fn paint(&self, ui: &mut Ui) {
        for toast in self.toasts.iter() {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.label(&toast.message);
            });
            ui.add_space(4.0);
        }
    }
}
impl<'a> egui::Widget for ToastLayer<'a> {
    /// Render toasts in the caller's current `Ui` flow.
    ///
    /// Prefer [`ToastLayer::show`] for app-level floating overlays because it
    /// uses an `egui::Area` and requests repaint while toasts are visible.
    fn ui(mut self, ui: &mut Ui) -> Response {
        self.advance(ui.input(|i| i.stable_dt));
        self.paint(ui);
        ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toast_can_be_created_from_feedback_toast() {
        let feedback = FeedbackToast::new("save", "Saved", 2.0);

        let toast = Toast::from_feedback(&feedback);

        assert_eq!(toast.message, "Saved");
        assert_eq!(toast.seconds_left, 2.0);
        assert_eq!(toast.accessibility.unwrap().role.as_str(), "status");
    }

    #[test]
    fn error_feedback_toast_uses_alert_role() {
        let feedback = FeedbackToast::new("error", "Failed", 2.0).severity(FeedbackSeverity::Error);

        let toast = Toast::from_feedback(&feedback);

        assert_eq!(toast.accessibility.unwrap().role.as_str(), "alert");
    }

    #[test]
    fn toast_layer_advance_expires_finished_toasts() {
        let mut toasts = vec![Toast::new("Done", 0.5), Toast::new("Keep", 2.0)];
        let mut layer = ToastLayer::new(&mut toasts);

        layer.advance(1.0);

        assert_eq!(layer.toasts.len(), 1);
        assert_eq!(layer.toasts[0].message, "Keep");
    }
}
