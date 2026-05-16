//! Reusable controls and app/editor widgets.

pub mod app_shell;
pub mod channel_strip;
pub mod controls;
pub mod data;
pub mod daw_editors;
pub mod designer;
pub mod displays;
pub mod dock;
pub mod drag;
pub mod editor_tools;
pub mod faders;
pub mod grid;
pub mod knobs;
pub mod menus;
pub mod meters;
pub mod overlays;
pub mod tabs;
pub mod timeline;
pub mod toolbar;
pub mod transport;
pub mod tree;

pub use app_shell::{
    register_app_shell_layout_slot, AppShellLayoutState, AppShellPanelState, BreadcrumbItem,
    Breadcrumbs, SidebarItem, SidebarNav, StatusBar, StatusBarItem,
};
pub use channel_strip::{ChannelStrip, ChannelStripStyle, SendControl};
pub use controls::{
    CollapsePanel, ColorSwatch, ControlGroup, DotState, SearchField, ToggleDot, ToolButton,
};
pub use data::{
    bounded_visible_range, flatten_tree_table_rows, DataCell, DataCellEditSpec, DataColumn,
    DataColumnFilter, DataFilterState, DataGridModel, DataGridState, DataRow, DataRowProvider,
    DataSelectionState, DataSortDirection, DataSortState, DataTable, DataViewStatus,
    PropertyEditSpec, PropertyGrid, PropertyGridCategory, PropertyGridEntry, PropertyGridGroup,
    PropertyGridModel, TreeTable, TreeTableModel, TreeTableNode, TreeTableRow, TreeTableState,
};
pub use daw_editors::{
    hsv_to_rgb, ColorWheel, ColorWheelState, ControllerLinkOverlay, ControllerLinkState,
    GeneratorOverlay, GeneratorSlot, MixerStripDesigner, MixerStripSection, PianoRoll,
    PianoRollNote, PianoRollView, PluginManager, PluginManagerItem, SystemMetric, SystemMonitor,
};
pub use designer::{DesignerCanvas, DesignerPart, RoutingCable};
pub use displays::{MiniBarGraph, SpectrogramDisplay, SpectrumDisplay, Waveform, WaveformDisplay};
pub use dock::{
    DockDropZone, DockOverlay, DockPanel, DockPanelId, DockPlacement, DockZone, ResizableSplit,
    SplitAxis,
};
pub use drag::{DragNumber, DragReorder, VerticalDrag};
pub use faders::{Fader, RangeSlider, Slider, XYPad};
pub use grid::{GridCanvas, NoteRect, StepCell, StepCellGrid, StepGrid};
pub use knobs::{ContinuousControl, Knob, KnobSize, KnobStyle, Orientation, ResetGesture};
pub use menus::{MenuDef, MenuItemDef, MenuItemKind, TopMenuBar};
pub use meters::{Meter, MeterBallistics, MeterMode};
pub use overlays::{
    CommandPalette, CommandPaletteItem, ContextMenuBuilder, ContextMenuEntry, FloatingPanel,
    FloatingPanelState, ModalOverlay, ProgressOverlay, Toast, ToastLayer,
};
pub use tabs::{TabBar, TabSetState};
pub use timeline::{
    AutomationCurve, AutomationPoint, AutomationSegment, ClipKind, FadeHandle, FadeSide,
    LoopRegion, Ruler, TimelineClip,
};
pub use toolbar::{ToolbarItem, ToolbarItemKind, ToolbarStrip};
pub use transport::{TransportButton, TransportKind};
pub use tree::{TreeNode, TreeView};
