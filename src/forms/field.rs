//! Shared label/help/error shell for native egui form widgets.

use crate::forms::ValidationMessage;

/// Simple field status used by wrappers and app code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldState {
    Default,
    Focused,
    Disabled,
    Invalid,
}

/// Label + optional helper/validation shell around one input control.
#[derive(Clone, Debug)]
pub struct FieldShell {
    pub label: String,
    pub message: Option<ValidationMessage>,
    pub state: FieldState,
}

impl FieldShell {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            message: None,
            state: FieldState::Default,
        }
    }

    pub fn message(mut self, message: ValidationMessage) -> Self {
        self.message = Some(message);
        self
    }

    pub fn state(mut self, state: FieldState) -> Self {
        self.state = state;
        self
    }

    pub fn show<R>(self, ui: &mut egui::Ui, input: impl FnOnce(&mut egui::Ui) -> R) -> R {
        ui.label(egui::RichText::new(self.label).strong());
        let result = input(ui);
        if let Some(message) = self.message {
            let color = message.color();
            ui.label(egui::RichText::new(message.text).color(color).small());
        }
        result
    }
}
