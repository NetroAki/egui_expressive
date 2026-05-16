//! Width, height, and min/max sizing utilities for `Tw`.

use crate::tailwind::{types::Size, Tw};

impl Tw {
    pub fn w(mut self, v: f32) -> Self {
        self.width = Size::Px(v);
        self
    }
    pub fn h(mut self, v: f32) -> Self {
        self.height = Size::Px(v);
        self
    }
    pub fn w_full(mut self) -> Self {
        self.width = Size::Full;
        self
    }
    pub fn h_full(mut self) -> Self {
        self.height = Size::Full;
        self
    }
    pub fn w_pct(mut self, percent: f32) -> Self {
        self.width = Size::Percent(percent);
        self
    }
    pub fn h_pct(mut self, percent: f32) -> Self {
        self.height = Size::Percent(percent);
        self
    }
    pub fn w_vw(mut self, percent: f32) -> Self {
        self.width = Size::ViewportWidth(percent);
        self
    }
    pub fn h_vh(mut self, percent: f32) -> Self {
        self.height = Size::ViewportHeight(percent);
        self
    }
    pub fn max_w_vw(mut self, percent: f32) -> Self {
        self.max_width = Some(Size::ViewportWidth(percent));
        self
    }
    pub fn min_h_screen(mut self) -> Self {
        self.min_height = Some(f32::INFINITY);
        self
    }
    pub fn min_w(mut self, v: f32) -> Self {
        self.min_width = Some(v);
        self
    }
    pub fn min_h(mut self, v: f32) -> Self {
        self.min_height = Some(v);
        self
    }
    pub fn max_w(mut self, v: f32) -> Self {
        self.max_width = Some(Size::Px(v));
        self
    }
    pub fn max_h(mut self, v: f32) -> Self {
        self.max_height = Some(Size::Px(v));
        self
    }
}
