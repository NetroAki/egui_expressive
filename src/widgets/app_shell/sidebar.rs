use egui::{Response, Ui};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SidebarItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
}

impl SidebarItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
        }
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn display_text(&self, collapsed: bool) -> &str {
        if collapsed {
            self.icon.as_deref().unwrap_or(&self.label)
        } else {
            &self.label
        }
    }
}

pub struct SidebarNav<'a> {
    selected: &'a mut String,
    items: &'a [SidebarItem],
    collapsed: bool,
}

impl<'a> SidebarNav<'a> {
    pub fn new(selected: &'a mut String, items: &'a [SidebarItem]) -> Self {
        Self {
            selected,
            items,
            collapsed: false,
        }
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }
}

impl egui::Widget for SidebarNav<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            for item in self.items {
                let selected = self.selected == &item.id;
                let text = item.display_text(self.collapsed);
                if ui.selectable_label(selected, text).clicked() {
                    *self.selected = item.id.clone();
                }
            }
        })
        .response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sidebar_item_tracks_optional_icon() {
        let item = SidebarItem::new("home", "Home").icon("⌂");
        assert_eq!(item.icon.as_deref(), Some("⌂"));
        assert_eq!(item.display_text(false), "Home");
        assert_eq!(item.display_text(true), "⌂");
    }
}
