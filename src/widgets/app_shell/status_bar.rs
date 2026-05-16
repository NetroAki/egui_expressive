use egui::{Response, Ui};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusBarItem {
    pub label: String,
    pub value: Option<String>,
}

impl StatusBarItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: None,
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn display_text(&self) -> String {
        match &self.value {
            Some(value) => format!("{}: {value}", self.label),
            None => self.label.clone(),
        }
    }
}

pub struct StatusBar<'a> {
    items: &'a [StatusBarItem],
}

impl<'a> StatusBar<'a> {
    pub fn new(items: &'a [StatusBarItem]) -> Self {
        Self { items }
    }

    pub fn display_texts(&self) -> impl Iterator<Item = String> + '_ {
        self.items.iter().map(StatusBarItem::display_text)
    }
}

impl egui::Widget for StatusBar<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal_wrapped(|ui| {
            for text in self.display_texts() {
                ui.label(text);
            }
        })
        .response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_bar_item_formats_value() {
        let item = StatusBarItem::new("CPU").value("12%");
        assert_eq!(item.value.as_deref(), Some("12%"));
        assert_eq!(item.display_text(), "CPU: 12%");
    }

    #[test]
    fn status_bar_display_texts_preserve_missing_values() {
        let items = [
            StatusBarItem::new("Ready"),
            StatusBarItem::new("Sync").value("ok"),
        ];
        let bar = StatusBar::new(&items);

        assert_eq!(
            bar.display_texts().collect::<Vec<_>>(),
            vec!["Ready", "Sync: ok"]
        );
    }
}
