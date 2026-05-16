//! Docking / split widgets.

mod overlay;
mod panel;
mod split;

pub use overlay::{DockDropZone, DockOverlay};
pub use panel::{DockPanel, DockPanelId, DockPlacement};
pub use split::{DockZone, ResizableSplit, SplitAxis};
