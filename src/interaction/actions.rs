use std::collections::{HashMap, VecDeque};

use egui::{Context, Modifiers};

use super::key_pressed;

/// Stable command/action description for toolbars, menus, palettes, shortcuts, and app glue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionDef {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub enabled: bool,
}

impl ActionDef {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: None,
            enabled: true,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Additive action registry used by menus, toolbars, and command palettes.
#[derive(Clone, Debug, Default)]
pub struct ActionRegistry {
    actions: HashMap<String, ActionDef>,
    migrations: HashMap<String, String>,
}

impl ActionRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn register(&mut self, action: ActionDef) {
        self.actions.insert(action.id.clone(), action);
    }
    pub fn migrate(&mut self, old_id: impl Into<String>, new_id: impl Into<String>) {
        self.migrations.insert(old_id.into(), new_id.into());
    }
    pub fn resolve<'a>(&'a self, id: &'a str) -> &'a str {
        self.migrations.get(id).map(String::as_str).unwrap_or(id)
    }
    pub fn get(&self, id: &str) -> Option<&ActionDef> {
        self.actions.get(self.resolve(id))
    }
    pub fn iter(&self) -> impl Iterator<Item = &ActionDef> {
        self.actions.values()
    }

    /// Resolve an action id and report whether it can be dispatched.
    pub fn dispatch_status(&self, id: &str) -> ActionDispatchStatus {
        match self.get(id) {
            Some(action) if action.enabled => ActionDispatchStatus::Ready(action.id.clone()),
            Some(action) => ActionDispatchStatus::Disabled(action.id.clone()),
            None => ActionDispatchStatus::Unknown(id.to_owned()),
        }
    }
}

/// Result of resolving an action id through the registry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionDispatchStatus {
    Ready(String),
    Disabled(String),
    Unknown(String),
}

/// Keyboard shortcut binding for command palettes, menus, and global shortcuts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShortcutBinding {
    pub action_id: String,
    pub key: egui::Key,
    pub modifiers: Modifiers,
}

impl ShortcutBinding {
    pub fn new(action_id: impl Into<String>, key: egui::Key, modifiers: Modifiers) -> Self {
        Self {
            action_id: action_id.into(),
            key,
            modifiers,
        }
    }

    pub fn matches(&self, ctx: &Context) -> bool {
        key_pressed(ctx, self.key, self.modifiers)
    }
}

/// Shortcut registry with conflict-aware additive insertion.
#[derive(Clone, Debug, Default)]
pub struct ShortcutRegistry {
    bindings: Vec<ShortcutBinding>,
}

impl ShortcutRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn bind(&mut self, binding: ShortcutBinding) -> Option<ShortcutBinding> {
        if let Some(index) = self
            .bindings
            .iter()
            .position(|b| b.key == binding.key && b.modifiers == binding.modifiers)
        {
            Some(std::mem::replace(&mut self.bindings[index], binding))
        } else {
            self.bindings.push(binding);
            None
        }
    }
    pub fn triggered(&self, ctx: &Context) -> Option<&str> {
        self.bindings
            .iter()
            .find(|b| b.matches(ctx))
            .map(|b| b.action_id.as_str())
    }
    pub fn iter(&self) -> impl Iterator<Item = &ShortcutBinding> {
        self.bindings.iter()
    }
}

/// Interaction-local semantic hint for custom-painted primitives.
///
/// Prefer `crate::accessibility::AccessibilityMeta` for public accessibility
/// metadata. This type remains local to action/shortcut glue so
/// `crate::interaction::AccessibilityMeta` is not exported as an ambiguous
/// alternative to the crate-level accessibility type.
#[derive(Clone, Debug, PartialEq)]
pub struct InteractionAccessibilityMeta {
    pub role: &'static str,
    pub label: String,
    pub value: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub keyboard_hint: Option<String>,
}

impl InteractionAccessibilityMeta {
    pub fn new(role: &'static str, label: impl Into<String>) -> Self {
        Self {
            role,
            label: label.into(),
            value: None,
            min: None,
            max: None,
            keyboard_hint: None,
        }
    }
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.min = Some(min);
        self.max = Some(max);
        self
    }
    pub fn keyboard_hint(mut self, hint: impl Into<String>) -> Self {
        self.keyboard_hint = Some(hint.into());
        self
    }
}

/// Lightweight viewport/window message bridge for detached panel coordination.
#[derive(Clone, Debug, Default)]
pub struct ViewportMessageBridge<M> {
    queue: VecDeque<M>,
}

impl<M> ViewportMessageBridge<M> {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
    pub fn push(&mut self, message: M) {
        self.queue.push_back(message);
    }
    pub fn pop(&mut self) -> Option<M> {
        self.queue.pop_front()
    }
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

/// Normalize a value from `range` to 0.0..=1.0.
pub fn normalize(value: f64, range: &std::ops::RangeInclusive<f64>) -> f32 {
    let min = *range.start();
    let max = *range.end();
    ((value - min) / (max - min)).clamp(0.0, 1.0) as f32
}

/// Denormalize a 0.0..=1.0 value back to `range`.
pub fn denormalize(t: f32, range: &std::ops::RangeInclusive<f64>) -> f64 {
    let min = *range.start();
    let max = *range.end();
    min + (t as f64) * (max - min)
}

#[cfg(test)]
mod primitive_infra_tests {
    use super::*;

    #[test]
    fn action_registry_resolves_migrations() {
        let mut registry = ActionRegistry::new();
        registry.register(ActionDef::new("transport.play", "Play"));
        registry.migrate("play", "transport.play");
        assert_eq!(registry.resolve("play"), "transport.play");
        assert_eq!(registry.get("play").unwrap().label, "Play");
    }

    #[test]
    fn shortcut_registry_replaces_conflicts() {
        let mut registry = ShortcutRegistry::new();
        let old = registry.bind(ShortcutBinding::new("a", egui::Key::A, Modifiers::NONE));
        assert!(old.is_none());
        let old = registry.bind(ShortcutBinding::new("b", egui::Key::A, Modifiers::NONE));
        assert_eq!(old.unwrap().action_id, "a");
        assert_eq!(registry.iter().next().unwrap().action_id, "b");
    }

    #[test]
    fn viewport_bridge_fifo() {
        let mut bridge = ViewportMessageBridge::new();
        bridge.push(1);
        bridge.push(2);
        assert_eq!(bridge.pop(), Some(1));
        assert_eq!(bridge.pop(), Some(2));
        assert!(bridge.is_empty());
    }
}
