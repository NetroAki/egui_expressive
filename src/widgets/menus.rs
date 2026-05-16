use crate::interaction::ActionDef;
use egui::{Response, RichText, Ui};

#[derive(Clone, Debug, PartialEq)]
pub enum MenuItemKind {
    Action,
    Separator,
    Check { checked: bool },
    Submenu { items: Vec<MenuItemDef> },
}

#[derive(Clone, Debug, PartialEq)]
pub struct MenuItemDef {
    pub id: String,
    pub label: String,
    pub shortcut: Option<String>,
    pub icon: Option<char>,
    pub disabled: bool,
    pub kind: MenuItemKind,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MenuDef {
    pub label: String,
    pub items: Vec<MenuItemDef>,
}

pub struct TopMenuBar<'a> {
    menus: &'a [MenuDef],
    activated: Option<&'a mut Option<String>>,
}

impl MenuItemDef {
    pub fn action(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            shortcut: None,
            icon: None,
            disabled: false,
            kind: MenuItemKind::Action,
        }
    }
    pub fn from_action(action: &ActionDef) -> Self {
        Self::action(action.id.clone(), action.label.clone()).disabled(!action.enabled)
    }
    pub fn separator() -> Self {
        Self {
            id: "separator".to_owned(),
            label: String::new(),
            shortcut: None,
            icon: None,
            disabled: true,
            kind: MenuItemKind::Separator,
        }
    }
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }
    pub fn icon(mut self, icon: char) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn checked(mut self, checked: bool) -> Self {
        self.kind = MenuItemKind::Check { checked };
        self
    }
    pub fn submenu(mut self, items: Vec<MenuItemDef>) -> Self {
        self.kind = MenuItemKind::Submenu { items };
        self
    }
}

impl MenuDef {
    pub fn actions(label: impl Into<String>, actions: impl IntoIterator<Item = ActionDef>) -> Self {
        Self {
            label: label.into(),
            items: actions
                .into_iter()
                .map(|action| MenuItemDef::from_action(&action))
                .collect(),
        }
    }
}

impl<'a> TopMenuBar<'a> {
    pub fn new(menus: &'a [MenuDef]) -> Self {
        Self {
            menus,
            activated: None,
        }
    }

    pub fn activated(mut self, activated: &'a mut Option<String>) -> Self {
        self.activated = Some(activated);
        self
    }
}

impl<'a> egui::Widget for TopMenuBar<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut activated = self.activated;
        ui.horizontal(|ui| {
            for menu in self.menus {
                ui.menu_button(&menu.label, |ui| {
                    show_items(ui, &menu.items, &mut activated)
                });
            }
        })
        .response
    }
}

fn show_items(ui: &mut Ui, items: &[MenuItemDef], activated: &mut Option<&mut Option<String>>) {
    for item in items {
        match &item.kind {
            MenuItemKind::Separator => {
                ui.separator();
            }
            MenuItemKind::Submenu { items } => {
                ui.menu_button(row_text(item), |ui| show_items(ui, items, activated));
            }
            MenuItemKind::Action | MenuItemKind::Check { .. } => {
                ui.add_enabled_ui(!item.disabled, |ui| {
                    let mut label = row_text(item);
                    if let Some(shortcut) = &item.shortcut {
                        label.push_str(&format!("\t{}", shortcut));
                    }
                    let text = if matches!(item.kind, MenuItemKind::Check { checked: true }) {
                        RichText::new(format!("✓ {}", label))
                    } else {
                        RichText::new(label)
                    };
                    if ui.button(text).clicked() {
                        set_activated(activated, item.id.clone());
                        ui.close();
                    }
                });
            }
        };
    }
}

fn set_activated(activated: &mut Option<&mut Option<String>>, id: String) {
    if let Some(target) = activated {
        **target = Some(id);
    }
}

fn row_text(item: &MenuItemDef) -> String {
    match item.icon {
        Some(icon) => format!("{}  {}", icon, item.label),
        None => item.label.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_items_capture_shortcuts_checks_submenus() {
        let item = MenuItemDef::action("save", "Save")
            .shortcut("Ctrl+S")
            .checked(true)
            .submenu(vec![MenuItemDef::separator()]);
        assert!(matches!(item.kind, MenuItemKind::Submenu { .. }));
        assert_eq!(item.shortcut.as_deref(), Some("Ctrl+S"));
    }

    #[test]
    fn menu_items_can_be_built_from_actions() {
        let item = MenuItemDef::from_action(&ActionDef::new("save", "Save").enabled(false));
        assert_eq!(item.id, "save");
        assert_eq!(item.label, "Save");
        assert!(item.disabled);
    }

    #[test]
    fn top_menu_bar_can_surface_activated_action_ids() {
        let menus = vec![MenuDef::actions(
            "File",
            vec![ActionDef::new("save", "Save")],
        )];
        let mut activated = None;
        let bar = TopMenuBar::new(&menus).activated(&mut activated);

        assert_eq!(bar.menus[0].items[0].id, "save");
        assert!(bar.activated.is_some());
    }
}
