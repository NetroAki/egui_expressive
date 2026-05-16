pub enum ContextMenuEntry {
    Item {
        label: String,
        shortcut: Option<String>,
        icon: Option<char>,
        disabled: bool,
        checked: bool,
        callback: Box<dyn FnMut()>,
    },
    Separator,
    Submenu {
        label: String,
        items: Vec<ContextMenuEntry>,
    },
}

pub struct ContextMenuBuilder {
    items: Vec<ContextMenuEntry>,
}

impl ContextMenuBuilder {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
    pub fn item(mut self, label: impl Into<String>, f: impl FnMut() + 'static) -> Self {
        self.items.push(ContextMenuEntry::Item {
            label: label.into(),
            shortcut: None,
            icon: None,
            disabled: false,
            checked: false,
            callback: Box::new(f),
        });
        self
    }
    pub fn rich_item(
        mut self,
        label: impl Into<String>,
        shortcut: Option<String>,
        icon: Option<char>,
        disabled: bool,
        checked: bool,
        f: impl FnMut() + 'static,
    ) -> Self {
        self.items.push(ContextMenuEntry::Item {
            label: label.into(),
            shortcut,
            icon,
            disabled,
            checked,
            callback: Box::new(f),
        });
        self
    }
    pub fn separator(mut self) -> Self {
        self.items.push(ContextMenuEntry::Separator);
        self
    }
    pub fn submenu(mut self, label: impl Into<String>, items: Vec<ContextMenuEntry>) -> Self {
        self.items.push(ContextMenuEntry::Submenu {
            label: label.into(),
            items,
        });
        self
    }
    pub fn show(mut self, ui: &mut egui::Ui) {
        show_entries(ui, &mut self.items);
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

fn show_entries(ui: &mut egui::Ui, entries: &mut [ContextMenuEntry]) {
    for entry in entries {
        match entry {
            ContextMenuEntry::Separator => {
                ui.separator();
            }
            ContextMenuEntry::Submenu { label, items } => {
                ui.menu_button(label.as_str(), |ui| show_entries(ui, items));
            }
            ContextMenuEntry::Item {
                label,
                shortcut,
                icon,
                disabled,
                checked,
                callback,
            } => {
                let mut row = String::new();
                if *checked {
                    row.push_str("✓ ");
                }
                if let Some(icon) = icon {
                    row.push(*icon);
                    row.push(' ');
                }
                row.push_str(label);
                if let Some(shortcut) = shortcut {
                    row.push_str("    ");
                    row.push_str(shortcut);
                }
                if ui.add_enabled(!*disabled, egui::Button::new(row)).clicked() {
                    callback();
                    ui.close();
                }
            }
        };
    }
}

impl Default for ContextMenuBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_menu_tracks_rich_entries() {
        let menu = ContextMenuBuilder::new()
            .rich_item("Rename", Some("F2".into()), Some('R'), false, false, || {})
            .separator()
            .submenu("More", vec![ContextMenuEntry::Separator]);
        assert_eq!(menu.len(), 3);
    }
}
