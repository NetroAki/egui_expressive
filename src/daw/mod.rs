//! Compatibility namespace for audio and creative-editor aliases.
//!
//! Stage 6 keeps this module for existing `daw` feature users, but the public
//! library guidance is to use the generic root modules and `widgets::editor_tools`
//! for non-audio editor canvases. Enable with `daw` or `creative-editors`.

pub use crate::widgets::{
    ChannelStrip, ClipKind, DotState, Fader, Meter, Ruler, StepGrid, TimelineClip, ToggleDot,
    TransportButton, TransportKind, Waveform,
};

pub use crate::draw::{icon_loop, icon_play, icon_record, icon_stop};
