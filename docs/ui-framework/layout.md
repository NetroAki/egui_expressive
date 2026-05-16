# Layout and App Shell

Stage 2 adds a reusable app-shell layer for dashboard/editor-style egui apps.

## App-Shell Modules

- `src/widgets/dock/` owns dock-specific value types, split widgets, and dock-zone overlay geometry.
- `src/widgets/app_shell/` owns generic chrome: layout persistence state, sidebars, status bars, and breadcrumbs.
- `src/widgets/tabs.rs` owns tab selection and tab-set fallback behavior.
- `src/state/mod.rs` can register bounded persistence slots; Stage 2 uses the existing `layout.panels` convention instead of inventing a second registry.

## Persistent Layout State

`AppShellLayoutState` derives `Serialize` and `Deserialize`. It stores dynamic app-shell state:

- panel IDs and placements
- split fractions
- floating geometry (floating panels under 80x48 logical pixels are treated as invalid recovery state and redocked)
- sidebar collapse state
- tab-set references, not selected-tab semantics

Selected tabs remain owned by `TabSetState` in `src/widgets/tabs.rs`.

```rust
use egui_expressive::{
    register_app_shell_layout_slot, AppShellLayoutState, AppShellPanelState,
    DockPanel, DockPlacement, DockZone,
};
use egui_expressive::state::PersistenceRegistry;

let panel = AppShellPanelState::new(DockPanel::new(
    "main",
    "Main",
    DockPlacement::docked(DockZone::Center),
))
.with_tab_set("main-tabs");

let mut registry = PersistenceRegistry::new();
register_app_shell_layout_slot(&mut registry);
let layout = AppShellLayoutState::with_panels(vec![panel]);
```

## Dashboard Composition

Use `ResizableSplit` for shell regions, `SidebarNav` for navigation, `Breadcrumbs` for page location, `TabBar`/`TabSetState` for content tabs, and `StatusBar` for low-priority runtime state.

```rust
use egui_expressive::{ResizableSplit, SidebarNav, SplitAxis, TabBar, TabSetState};

let mut split_fraction = 0.3;
let mut selected_nav = "overview".to_owned();
let mut tabs = TabSetState::new(0);
let items = [];
let labels = vec!["Summary".to_owned(), "Activity".to_owned()];

ResizableSplit::new("dashboard", &mut split_fraction, SplitAxis::Horizontal).show(
    ui,
    |ui| { ui.add(SidebarNav::new(&mut selected_nav, &items)); },
    |ui| { ui.add(TabBar::new(&mut tabs.selected, labels)); },
);
```

See `examples/responsive_dashboard.rs` for the canonical Stage 2 proof.

## Surface Relationship

`src/surface/mod.rs` remains the large-canvas/viewport-culling layer. Stage 2 documents how shell layouts can host surfaces; deeper editor/surface integration remains Stage 6.
