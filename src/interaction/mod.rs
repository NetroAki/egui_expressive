//! Interaction helpers for commands, shortcuts, drag, gestures, focus, history, and feedback.
//!
//! This module is intentionally a wiring and re-export hub. Keep behavior in
//! focused submodules so command/focus/undo/feedback architecture stays easy to
//! audit and extend.

pub mod actions;
pub mod drag;
pub mod feedback;
pub mod focus;
pub mod gestures;
pub mod history;
pub mod shortcuts;

pub use actions::{
    denormalize, normalize, ActionDef, ActionDispatchStatus, ActionRegistry, ShortcutBinding,
    ShortcutRegistry, ViewportMessageBridge,
};
pub use drag::{drag_to_value_delta, key_pressed, DragAxis, DragDelta, PanZoom};
pub use feedback::{
    FeedbackMessage, FeedbackProgress, FeedbackQueue, FeedbackSeverity, FeedbackToast,
};
pub use focus::{next_focus_in_order, FocusDirection, FocusScope};
pub use gestures::{
    LongPressEvent, LongPressGesture, SwipeDirection, SwipeEvent, SwipeGesture, TapEvent,
    TapGesture,
};
pub use history::{UndoEntry, UndoStack};
pub use shortcuts::{
    format_shortcut, ScopedShortcutBinding, ScopedShortcutRegistry, ShortcutConflict,
    ShortcutHelpItem, ShortcutResolution, ShortcutScope,
};
