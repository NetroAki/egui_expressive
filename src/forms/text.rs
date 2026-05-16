//! Text field and text area wrappers around egui `TextEdit`.

use crate::forms::{FieldShell, ValidationMessage};

pub struct TextField<'a> {
    label: String,
    value: &'a mut String,
    hint: Option<String>,
    message: Option<ValidationMessage>,
    enabled: bool,
}

impl<'a> TextField<'a> {
    pub fn new(label: impl Into<String>, value: &'a mut String) -> Self {
        Self {
            label: label.into(),
            value,
            hint: None,
            message: None,
            enabled: true,
        }
    }

    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn message(mut self, message: ValidationMessage) -> Self {
        self.message = Some(message);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let shell = shell(self.label, self.message);
        shell.show(ui, |ui| {
            ui.add_enabled_ui(self.enabled, |ui| {
                let mut edit = egui::TextEdit::singleline(self.value).desired_width(f32::INFINITY);
                if let Some(hint) = self.hint {
                    edit = edit.hint_text(hint);
                }
                ui.add(edit)
            })
            .inner
        })
    }
}

pub struct TextAreaField<'a> {
    label: String,
    value: &'a mut String,
    rows: usize,
    message: Option<ValidationMessage>,
}

impl<'a> TextAreaField<'a> {
    pub fn new(label: impl Into<String>, value: &'a mut String) -> Self {
        Self {
            label: label.into(),
            value,
            rows: 4,
            message: None,
        }
    }

    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows.max(1);
        self
    }

    pub fn message(mut self, message: ValidationMessage) -> Self {
        self.message = Some(message);
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let shell = shell(self.label, self.message);
        shell.show(ui, |ui| {
            ui.add(egui::TextEdit::multiline(self.value).desired_rows(self.rows))
        })
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
