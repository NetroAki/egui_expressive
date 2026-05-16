//! Pointer and cursor utilities for `Tw`.

use egui::CursorIcon;

use crate::tailwind::Tw;

impl Tw {
    pub fn pointer_events_none(mut self) -> Self {
        self.pointer_events = false;
        self
    }
    pub fn pointer_events_auto(mut self) -> Self {
        self.pointer_events = true;
        self
    }
    pub fn cursor(mut self, cursor: CursorIcon) -> Self {
        self.cursor = Some(cursor);
        self
    }
    pub fn cursor_pointer(self) -> Self {
        self.cursor(CursorIcon::PointingHand)
    }
    pub fn cursor_move(self) -> Self {
        self.cursor(CursorIcon::Grab)
    }
    pub fn cursor_resize_ew(self) -> Self {
        self.cursor(CursorIcon::ResizeHorizontal)
    }
    pub fn cursor_resize_se(self) -> Self {
        self.cursor(CursorIcon::ResizeNwSe)
    }
}
