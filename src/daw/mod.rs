//! DAW (Digital Audio Workstation) specific widgets and utilities.
//!
//! These are domain-specific composites for audio software. Enable with the `daw` feature.
//! For general-purpose controls, use the root crate modules directly.

pub use crate::widgets::{
    ChannelStrip, ClipKind, DotState, Fader, Meter, Ruler, StepGrid, TimelineClip, ToggleDot,
    TransportButton, TransportKind, Waveform,
};

pub use crate::draw::{icon_loop, icon_play, icon_record, icon_stop};
