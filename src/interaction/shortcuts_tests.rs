use super::*;
use crate::interaction::{ActionDef, ActionRegistry, ShortcutBinding};
use egui::Modifiers;

#[test]
fn scoped_shortcuts_reject_same_scope_conflicts() {
    let mut registry = ScopedShortcutRegistry::new();
    registry
        .bind(ScopedShortcutBinding::new(
            ShortcutScope::Global,
            ShortcutBinding::new("save", egui::Key::S, Modifiers::CTRL),
        ))
        .unwrap();

    let conflict = registry
        .bind(ScopedShortcutBinding::new(
            ShortcutScope::Global,
            ShortcutBinding::new("search", egui::Key::S, Modifiers::CTRL),
        ))
        .unwrap_err();

    assert_eq!(conflict.existing_action_id, "save");
    assert_eq!(conflict.requested_action_id, "search");
}

#[test]
fn scoped_shortcuts_allow_cross_scope_override() {
    let mut actions = ActionRegistry::new();
    actions.register(ActionDef::new("global.save", "Save"));
    actions.register(ActionDef::new("modal.submit", "Submit"));

    let mut registry = ScopedShortcutRegistry::new();
    registry
        .bind(ScopedShortcutBinding::new(
            ShortcutScope::Global,
            ShortcutBinding::new("global.save", egui::Key::Enter, Modifiers::NONE),
        ))
        .unwrap();
    registry
        .bind(ScopedShortcutBinding::new(
            ShortcutScope::Modal("confirm".into()),
            ShortcutBinding::new("modal.submit", egui::Key::Enter, Modifiers::NONE),
        ))
        .unwrap();

    assert_eq!(
        registry.resolve_key(
            egui::Key::Enter,
            Modifiers::NONE,
            &[
                ShortcutScope::Global,
                ShortcutScope::Modal("confirm".into()),
            ],
            &actions,
        ),
        ShortcutResolution::Dispatched {
            scope: ShortcutScope::Modal("confirm".into()),
            action_id: "modal.submit".into(),
        }
    );
}

#[test]
fn scoped_shortcuts_fall_through_non_modal_without_binding() {
    let mut actions = ActionRegistry::new();
    actions.register(ActionDef::new("global.save", "Save"));

    let mut registry = ScopedShortcutRegistry::new();
    registry
        .bind(ScopedShortcutBinding::new(
            ShortcutScope::Global,
            ShortcutBinding::new("global.save", egui::Key::S, Modifiers::CTRL),
        ))
        .unwrap();

    assert_eq!(
        registry.resolve_key(
            egui::Key::S,
            Modifiers::CTRL,
            &[
                ShortcutScope::Global,
                ShortcutScope::FocusedPanel("inspector".into()),
            ],
            &actions,
        ),
        ShortcutResolution::Dispatched {
            scope: ShortcutScope::Global,
            action_id: "global.save".into(),
        }
    );
}

#[test]
fn modal_scope_traps_unmatched_shortcuts() {
    let actions = ActionRegistry::new();
    let registry = ScopedShortcutRegistry::new();

    assert_eq!(
        registry.resolve_key(
            egui::Key::S,
            Modifiers::CTRL,
            &[
                ShortcutScope::Global,
                ShortcutScope::Modal("confirm".into()),
            ],
            &actions,
        ),
        ShortcutResolution::Trapped {
            scope: ShortcutScope::Modal("confirm".into()),
        }
    );
}

#[test]
fn disabled_shortcut_consumes_without_dispatch() {
    let mut actions = ActionRegistry::new();
    actions.register(ActionDef::new("delete", "Delete").enabled(false));
    actions.register(ActionDef::new("global.delete", "Global Delete"));

    let mut registry = ScopedShortcutRegistry::new();
    registry
        .bind(ScopedShortcutBinding::new(
            ShortcutScope::Global,
            ShortcutBinding::new("global.delete", egui::Key::Delete, Modifiers::NONE),
        ))
        .unwrap();
    registry
        .bind(ScopedShortcutBinding::new(
            ShortcutScope::FocusedPanel("table".into()),
            ShortcutBinding::new("delete", egui::Key::Delete, Modifiers::NONE),
        ))
        .unwrap();

    assert_eq!(
        registry.resolve_key(
            egui::Key::Delete,
            Modifiers::NONE,
            &[
                ShortcutScope::Global,
                ShortcutScope::FocusedPanel("table".into()),
            ],
            &actions,
        ),
        ShortcutResolution::Disabled {
            scope: ShortcutScope::FocusedPanel("table".into()),
            action_id: "delete".into(),
        }
    );
}

#[test]
fn unknown_shortcut_actions_are_reported_separately() {
    let actions = ActionRegistry::new();
    let mut registry = ScopedShortcutRegistry::new();
    registry
        .bind(ScopedShortcutBinding::new(
            ShortcutScope::Global,
            ShortcutBinding::new("missing", egui::Key::M, Modifiers::CTRL),
        ))
        .unwrap();

    assert_eq!(
        registry.resolve_key(
            egui::Key::M,
            Modifiers::CTRL,
            &[ShortcutScope::Global],
            &actions,
        ),
        ShortcutResolution::Unknown {
            scope: ShortcutScope::Global,
            action_id: "missing".into(),
        }
    );
}

#[test]
fn shortcut_help_items_use_action_labels() {
    let mut actions = ActionRegistry::new();
    actions.register(ActionDef::new("save", "Save File"));

    let mut registry = ScopedShortcutRegistry::new();
    registry
        .bind(ScopedShortcutBinding::new(
            ShortcutScope::Global,
            ShortcutBinding::new("save", egui::Key::S, Modifiers::CTRL),
        ))
        .unwrap();

    let items = registry.help_items(&actions);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].label, "Save File");
    assert_eq!(items[0].shortcut, "Ctrl+S");
}
