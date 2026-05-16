//! Generic app-shell chrome primitives.

mod breadcrumbs;
mod layout_state;
mod sidebar;
mod status_bar;

pub use breadcrumbs::{BreadcrumbItem, Breadcrumbs};
pub use layout_state::{register_app_shell_layout_slot, AppShellLayoutState, AppShellPanelState};
pub use sidebar::{SidebarItem, SidebarNav};
pub use status_bar::{StatusBar, StatusBarItem};
