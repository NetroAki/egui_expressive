//! Position, inset, translate, and z-index utility methods for `Tw`.

use egui::vec2;

use crate::layout::{Insets, PositionMode};
use crate::tailwind::builder::Tw;

impl Tw {
    pub fn relative(mut self) -> Self {
        self.position.mode = PositionMode::Relative;
        self
    }

    pub fn absolute(mut self) -> Self {
        self.position.mode = PositionMode::Absolute;
        self
    }

    pub fn fixed(mut self) -> Self {
        self.position.mode = PositionMode::Fixed;
        self
    }

    pub fn sticky(mut self) -> Self {
        self.position.mode = PositionMode::Sticky;
        self
    }

    pub fn inset(mut self, value: f32) -> Self {
        self.position.inset = Insets::all(value);
        self
    }

    pub fn inset_x(mut self, value: f32) -> Self {
        self.position.inset.left = Some(value);
        self.position.inset.right = Some(value);
        self
    }

    pub fn inset_y(mut self, value: f32) -> Self {
        self.position.inset.top = Some(value);
        self.position.inset.bottom = Some(value);
        self
    }

    pub fn top(mut self, value: f32) -> Self {
        self.position.inset.top = Some(value);
        self
    }

    pub fn right(mut self, value: f32) -> Self {
        self.position.inset.right = Some(value);
        self
    }

    pub fn bottom(mut self, value: f32) -> Self {
        self.position.inset.bottom = Some(value);
        self
    }

    pub fn left(mut self, value: f32) -> Self {
        self.position.inset.left = Some(value);
        self
    }

    pub fn translate(mut self, x: f32, y: f32) -> Self {
        self.position.translate = vec2(x, y);
        self
    }

    pub fn translate_x(mut self, x: f32) -> Self {
        self.position.translate.x = x;
        self
    }

    pub fn translate_y(mut self, y: f32) -> Self {
        self.position.translate.y = y;
        self
    }

    pub fn z(mut self, z_index: i32) -> Self {
        self.position.z_index = Some(z_index);
        self
    }
}
