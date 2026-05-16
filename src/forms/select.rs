//! Select/dropdown wrapper around egui `ComboBox`.

use crate::forms::{FieldShell, ValidationMessage};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectOption<T> {
    pub value: T,
    pub label: String,
}

impl<T> SelectOption<T> {
    pub fn new(value: T, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
        }
    }
}

pub struct SelectField<'a, T> {
    label: String,
    selected: &'a mut T,
    options: Vec<SelectOption<T>>,
    message: Option<ValidationMessage>,
}

impl<'a, T> SelectField<'a, T>
where
    T: Clone + PartialEq,
{
    pub fn new(label: impl Into<String>, selected: &'a mut T) -> Self {
        Self {
            label: label.into(),
            selected,
            options: Vec::new(),
            message: None,
        }
    }

    pub fn options(mut self, options: impl IntoIterator<Item = SelectOption<T>>) -> Self {
        self.options = options.into_iter().collect();
        self
    }

    pub fn message(mut self, message: ValidationMessage) -> Self {
        self.message = Some(message);
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let selected_label = self
            .options
            .iter()
            .find(|option| option.value == *self.selected)
            .map(|option| option.label.as_str())
            .unwrap_or("Select…")
            .to_owned();
        let id_label = self.label.clone();
        let selected = self.selected;
        let options = self.options;
        shell(self.label, self.message).show(ui, |ui| {
            egui::ComboBox::from_id_salt(id_label)
                .selected_text(selected_label)
                .show_ui(ui, |ui| {
                    for option in options {
                        ui.selectable_value(selected, option.value, option.label);
                    }
                })
                .response
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
