//! Creative editor primitives retained under the legacy `daw_editors` path.
//!
//! New general-purpose editor/canvas interactions live in `crate::editor` and
//! are re-exported through `widgets::editor_tools` for non-DAW callers.

mod color_wheel;
mod controller_link;
mod generator_overlay;
mod mixer_designer;
mod piano_roll;
mod plugin_manager;
mod system_monitor;

pub use color_wheel::{hsv_to_rgb, ColorWheel, ColorWheelState};
pub use controller_link::{ControllerLinkOverlay, ControllerLinkState};
pub use generator_overlay::{GeneratorOverlay, GeneratorSlot};
pub use mixer_designer::{MixerStripDesigner, MixerStripSection};
pub use piano_roll::{PianoRoll, PianoRollNote, PianoRollView};
pub use plugin_manager::{PluginManager, PluginManagerItem};
pub use system_monitor::{SystemMetric, SystemMonitor};
