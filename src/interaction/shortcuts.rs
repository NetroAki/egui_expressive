use egui::{Context, Modifiers};

use super::{ActionDispatchStatus, ActionRegistry, ShortcutBinding};

/// Shortcut resolution scope ordered from broadest to most specific.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ShortcutScope {
    Global,
    FocusedPanel(String),
    Overlay(String),
    Modal(String),
}

impl ShortcutScope {
    fn priority(&self) -> u8 {
        match self {
            Self::Global => 0,
            Self::FocusedPanel(_) => 1,
            Self::Overlay(_) => 2,
            Self::Modal(_) => 3,
        }
    }

    fn traps_unmatched_keys(&self) -> bool {
        matches!(self, Self::Modal(_))
    }
}

/// Shortcut binding tied to a deterministic scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopedShortcutBinding {
    pub scope: ShortcutScope,
    pub binding: ShortcutBinding,
}

impl ScopedShortcutBinding {
    pub fn new(scope: ShortcutScope, binding: ShortcutBinding) -> Self {
        Self { scope, binding }
    }
}

/// Conflict returned when a scope already owns a key combo for another action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShortcutConflict {
    pub scope: ShortcutScope,
    pub key: egui::Key,
    pub modifiers: Modifiers,
    pub existing_action_id: String,
    pub requested_action_id: String,
}

/// Deterministic result of resolving a scoped shortcut.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShortcutResolution {
    Dispatched {
        scope: ShortcutScope,
        action_id: String,
    },
    Disabled {
        scope: ShortcutScope,
        action_id: String,
    },
    Unknown {
        scope: ShortcutScope,
        action_id: String,
    },
    Trapped {
        scope: ShortcutScope,
    },
    NoMatch,
}

/// Discoverable shortcut row for help overlays and command palettes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShortcutHelpItem {
    pub scope: ShortcutScope,
    pub action_id: String,
    pub label: String,
    pub shortcut: String,
}

/// Scoped shortcut registry: modal > overlay > focused panel > global.
#[derive(Clone, Debug, Default)]
pub struct ScopedShortcutRegistry {
    bindings: Vec<ScopedShortcutBinding>,
}

impl ScopedShortcutRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind(
        &mut self,
        binding: ScopedShortcutBinding,
    ) -> Result<Option<ScopedShortcutBinding>, ShortcutConflict> {
        if let Some(index) = self.bindings.iter().position(|existing| {
            existing.scope == binding.scope
                && existing.binding.key == binding.binding.key
                && existing.binding.modifiers == binding.binding.modifiers
        }) {
            let existing = &self.bindings[index];
            if existing.binding.action_id != binding.binding.action_id {
                return Err(ShortcutConflict {
                    scope: binding.scope,
                    key: binding.binding.key,
                    modifiers: binding.binding.modifiers,
                    existing_action_id: existing.binding.action_id.clone(),
                    requested_action_id: binding.binding.action_id,
                });
            }
            return Ok(Some(std::mem::replace(&mut self.bindings[index], binding)));
        }

        self.bindings.push(binding);
        Ok(None)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ScopedShortcutBinding> {
        self.bindings.iter()
    }

    pub fn help_items(&self, actions: &ActionRegistry) -> Vec<ShortcutHelpItem> {
        self.bindings
            .iter()
            .filter_map(|binding| {
                let action = actions.get(&binding.binding.action_id)?;
                Some(ShortcutHelpItem {
                    scope: binding.scope.clone(),
                    action_id: action.id.clone(),
                    label: action.label.clone(),
                    shortcut: format_shortcut(binding.binding.key, binding.binding.modifiers),
                })
            })
            .collect()
    }

    pub fn resolve_key(
        &self,
        key: egui::Key,
        modifiers: Modifiers,
        active_scopes: &[ShortcutScope],
        actions: &ActionRegistry,
    ) -> ShortcutResolution {
        let mut scopes = active_scopes.to_vec();
        scopes.sort_by_key(|scope| std::cmp::Reverse(scope.priority()));

        for scope in scopes {
            if let Some(binding) = self.bindings.iter().find(|binding| {
                binding.scope == scope
                    && binding.binding.key == key
                    && binding.binding.modifiers == modifiers
            }) {
                return match actions.dispatch_status(&binding.binding.action_id) {
                    ActionDispatchStatus::Ready(action_id) => {
                        ShortcutResolution::Dispatched { scope, action_id }
                    }
                    ActionDispatchStatus::Disabled(action_id) => {
                        ShortcutResolution::Disabled { scope, action_id }
                    }
                    ActionDispatchStatus::Unknown(action_id) => {
                        ShortcutResolution::Unknown { scope, action_id }
                    }
                };
            }

            if scope.traps_unmatched_keys() {
                return ShortcutResolution::Trapped { scope };
            }
        }

        ShortcutResolution::NoMatch
    }

    pub fn resolve_pressed(
        &self,
        ctx: &Context,
        active_scopes: &[ShortcutScope],
        actions: &ActionRegistry,
    ) -> ShortcutResolution {
        if active_scopes
            .iter()
            .any(ShortcutScope::traps_unmatched_keys)
        {
            if let Some((key, modifiers)) = pressed_key_event(ctx) {
                return self.resolve_key(key, modifiers, active_scopes, actions);
            }
        }

        self.bindings
            .iter()
            .find(|binding| binding.binding.matches(ctx))
            .map(|binding| {
                self.resolve_key(
                    binding.binding.key,
                    binding.binding.modifiers,
                    active_scopes,
                    actions,
                )
            })
            .unwrap_or(ShortcutResolution::NoMatch)
    }
}

fn pressed_key_event(ctx: &Context) -> Option<(egui::Key, Modifiers)> {
    ctx.input(|input| {
        input.events.iter().find_map(|event| match event {
            egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } => Some((*key, *modifiers)),
            _ => None,
        })
    })
}

pub fn format_shortcut(key: egui::Key, modifiers: Modifiers) -> String {
    let mut parts = Vec::new();
    if modifiers.ctrl {
        parts.push("Ctrl".to_owned());
    }
    if modifiers.command {
        parts.push("Cmd".to_owned());
    }
    if modifiers.alt {
        parts.push("Alt".to_owned());
    }
    if modifiers.shift {
        parts.push("Shift".to_owned());
    }
    parts.push(format!("{:?}", key));
    parts.join("+")
}

#[cfg(test)]
#[path = "shortcuts_tests.rs"]
mod tests;
