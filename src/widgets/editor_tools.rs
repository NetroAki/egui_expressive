//! Generic creative-editor widget aliases.
//!
//! This module is the DAW-neutral entry point for the existing editor chrome
//! primitives that historically lived under `widgets::daw_editors`. The legacy
//! module remains for compatibility; new editor/canvas behavior should prefer
//! `crate::editor` plus these neutral aliases.

pub use crate::widgets::daw_editors::{
    hsv_to_rgb, ColorWheel, ColorWheelState, ControllerLinkOverlay, ControllerLinkState,
    GeneratorOverlay, GeneratorSlot, MixerStripDesigner, MixerStripSection, PianoRoll,
    PianoRollNote, PianoRollView, PluginManager, PluginManagerItem, SystemMetric, SystemMonitor,
};
