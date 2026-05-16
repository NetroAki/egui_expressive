use egui::{Response, Ui};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Breadcrumb navigation item.
///
/// `id` is intentionally public so recovered navigation/layout state can retarget or
/// replace entries when serialized paths drift.
pub struct BreadcrumbItem {
    /// Stable breadcrumb key; may be rewritten during recovery.
    pub id: String,
    /// Display label shown in the breadcrumb trail.
    pub label: String,
}

impl BreadcrumbItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

pub struct Breadcrumbs<'a> {
    items: &'a [BreadcrumbItem],
}

impl<'a> Breadcrumbs<'a> {
    pub fn new(items: &'a [BreadcrumbItem]) -> Self {
        Self { items }
    }

    pub fn joined_labels(&self, separator: &str) -> String {
        self.items
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>()
            .join(separator)
    }
}

impl egui::Widget for Breadcrumbs<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal_wrapped(|ui| {
            for (index, item) in self.items.iter().enumerate() {
                if index > 0 {
                    ui.label("/");
                }
                ui.label(&item.label);
            }
        })
        .response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breadcrumb_item_keeps_id_and_label() {
        let item = BreadcrumbItem::new("settings.audio", "Audio");
        assert_eq!(item.id, "settings.audio");
        assert_eq!(item.label, "Audio");
    }

    #[test]
    fn breadcrumbs_join_labels_and_handle_empty_slices() {
        let empty = Breadcrumbs::new(&[]);
        assert_eq!(empty.joined_labels(" / "), "");

        let items = [
            BreadcrumbItem::new("dashboard", "Dashboard"),
            BreadcrumbItem::new("overview", "Overview"),
        ];
        let breadcrumbs = Breadcrumbs::new(&items);
        assert_eq!(breadcrumbs.joined_labels(" / "), "Dashboard / Overview");
    }
}
