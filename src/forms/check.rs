//! Boolean input wrappers around egui checkbox-style controls.

use crate::forms::{FieldShell, ValidationMessage};

pub struct CheckboxField<'a> {
    label: String,
    value: &'a mut bool,
    message: Option<ValidationMessage>,
}

impl<'a> CheckboxField<'a> {
    pub fn new(label: impl Into<String>, value: &'a mut bool) -> Self {
        Self {
            label: label.into(),
            value,
            message: None,
        }
    }

    pub fn message(mut self, message: ValidationMessage) -> Self {
        self.message = Some(message);
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let label = self.label.clone();
        shell(self.label, self.message).show(ui, |ui| ui.checkbox(self.value, label))
    }
}

pub struct SwitchField<'a> {
    label: String,
    value: &'a mut bool,
    message: Option<ValidationMessage>,
}

impl<'a> SwitchField<'a> {
    pub fn new(label: impl Into<String>, value: &'a mut bool) -> Self {
        Self {
            label: label.into(),
            value,
            message: None,
        }
    }

    pub fn message(mut self, message: ValidationMessage) -> Self {
        self.message = Some(message);
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let label = self.label.clone();
        shell(self.label, self.message).show(ui, |ui| ui.toggle_value(self.value, label))
    }
}

fn shell(label: String, message: Option<ValidationMessage>) -> FieldShell {
    let shell = FieldShell::new(label);
    if let Some(message) = message {
        shell.message(message)
    } else {
        shell
    }
}

#[cfg(test)]
mod tests {
    use crate::forms::{ValidationMessage, ValidationSeverity};

    #[test]
    fn validation_message_tracks_severity() {
        let message = ValidationMessage::error("required");
        assert_eq!(message.severity, ValidationSeverity::Error);
        assert_eq!(message.text, "required");
    }
}
