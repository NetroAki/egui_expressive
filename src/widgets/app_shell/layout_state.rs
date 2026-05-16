use crate::state::{PersistenceRegistry, PersistenceSlot};
use crate::widgets::dock::{DockPanel, DockPanelId, DockPlacement, DockZone};
use serde::{Deserialize, Serialize};

pub const APP_SHELL_LAYOUT_SLOT: &str = "layout.panels";
const DEFAULT_LAYOUT_MAX_BYTES: usize = 16 * 1024;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// Serializable app-shell panel state.
///
/// `panel` stays public so recovered/deserialized layouts can replace whole panel
/// records when IDs drift and layout repair needs to rebuild state.
pub struct AppShellPanelState {
    /// Persisted dock panel; may be replaced during layout recovery.
    pub panel: DockPanel,
    /// Visibility flag tracked separately from panel identity.
    pub visible: bool,
    /// Optional tab-set key used to reconnect recovered panels.
    pub tab_set: Option<String>,
}

impl AppShellPanelState {
    pub fn new(panel: DockPanel) -> Self {
        Self {
            panel,
            visible: true,
            tab_set: None,
        }
    }

    pub fn with_tab_set(mut self, tab_set: impl Into<String>) -> Self {
        self.tab_set = Some(tab_set.into());
        self
    }

    pub fn id(&self) -> &DockPanelId {
        self.panel.id()
    }

    pub fn placement(&self) -> &DockPlacement {
        self.panel.placement()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppShellLayoutState {
    pub panels: Vec<AppShellPanelState>,
    pub split_fraction: f32,
    pub sidebar_collapsed: bool,
}

impl Default for AppShellLayoutState {
    fn default() -> Self {
        Self {
            panels: Vec::new(),
            split_fraction: 0.3,
            sidebar_collapsed: false,
        }
    }
}

impl AppShellLayoutState {
    pub fn with_panels(panels: impl Into<Vec<AppShellPanelState>>) -> Self {
        Self {
            panels: panels.into(),
            ..Self::default()
        }
    }

    pub fn recovered(mut self, known_panels: &[DockPanelId]) -> Self {
        self.split_fraction = self.split_fraction.clamp(0.1, 0.9);
        self.panels
            .retain(|panel| known_panels.iter().any(|known| known == panel.id()));
        for panel in &mut self.panels {
            panel.panel.recover_placement(DockZone::Center);
        }
        self
    }

    pub fn visible_panels(&self) -> impl Iterator<Item = &AppShellPanelState> {
        self.panels.iter().filter(|panel| panel.visible)
    }
}

pub fn register_app_shell_layout_slot(registry: &mut PersistenceRegistry) {
    registry
        .register(PersistenceSlot::new(APP_SHELL_LAYOUT_SLOT, DEFAULT_LAYOUT_MAX_BYTES).version(2));
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, vec2};

    #[test]
    fn app_shell_layout_recovers_unknown_panels_and_split_bounds() {
        let known = DockPanelId::new("main");
        let state = AppShellLayoutState {
            panels: vec![
                AppShellPanelState::new(DockPanel::new(
                    "main",
                    "Main",
                    DockPlacement::docked(DockZone::Center),
                ))
                .with_tab_set("main-tabs"),
                AppShellPanelState::new(DockPanel::new(
                    DockPanelId::new("removed"),
                    "Removed",
                    DockPlacement::docked(DockZone::Left),
                )),
            ],
            split_fraction: 2.0,
            sidebar_collapsed: true,
        }
        .recovered(std::slice::from_ref(&known));
        assert_eq!(state.panels.len(), 1);
        assert_eq!(state.panels[0].id(), &known);
        assert_eq!(state.panels[0].tab_set.as_deref(), Some("main-tabs"));
        assert_eq!(state.split_fraction, 0.9);
    }

    #[test]
    fn app_shell_layout_recovers_invalid_floating_geometry() {
        let known = DockPanelId::new("floating");
        let state =
            AppShellLayoutState::with_panels(vec![AppShellPanelState::new(DockPanel::new(
                "floating",
                "Floating",
                DockPlacement::floating(pos2(f32::INFINITY, 0.0), vec2(32.0, f32::NAN)),
            ))])
            .recovered(std::slice::from_ref(&known));

        assert_eq!(
            state.panels[0].placement(),
            &DockPlacement::docked(DockZone::Center)
        );
    }

    #[test]
    fn app_shell_layout_filters_visible_panels() {
        let mut state = AppShellLayoutState::with_panels(vec![AppShellPanelState::new(
            DockPanel::new("main", "Main", DockPlacement::docked(DockZone::Center)),
        )]);
        state.panels[0].visible = false;
        assert_eq!(state.visible_panels().count(), 0);
    }

    #[test]
    fn app_shell_layout_registers_bounded_persistence_slot() {
        let mut registry = PersistenceRegistry::new();
        register_app_shell_layout_slot(&mut registry);
        assert!(registry.validate(APP_SHELL_LAYOUT_SLOT, DEFAULT_LAYOUT_MAX_BYTES));
        assert!(!registry.validate(APP_SHELL_LAYOUT_SLOT, DEFAULT_LAYOUT_MAX_BYTES + 1));
        assert_eq!(registry.get(APP_SHELL_LAYOUT_SLOT).unwrap().version, 2);
    }
}
