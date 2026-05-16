//! Flex/gap convenience utilities for `Tw`.

use egui::Vec2;

use crate::tailwind::{types::Size, Tw};

impl Tw {
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = Some(Vec2::splat(gap));
        self
    }
    pub fn gap_x(mut self, gap: f32) -> Self {
        self.gap.get_or_insert(Vec2::ZERO).x = gap;
        self
    }
    pub fn gap_y(mut self, gap: f32) -> Self {
        self.gap.get_or_insert(Vec2::ZERO).y = gap;
        self
    }
    pub fn space_x(mut self, space: f32) -> Self {
        self.space.get_or_insert(Vec2::ZERO).x = space;
        self
    }
    pub fn space_y(mut self, space: f32) -> Self {
        self.space.get_or_insert(Vec2::ZERO).y = space;
        self
    }
    pub fn divide_x(mut self, width: f32) -> Self {
        self.divide.get_or_insert(Vec2::ZERO).x = width;
        self
    }
    pub fn divide_y(mut self, width: f32) -> Self {
        self.divide.get_or_insert(Vec2::ZERO).y = width;
        self
    }
    pub fn flex_wrap(mut self) -> Self {
        self.flex_wrap = true;
        self
    }
    pub fn flex_nowrap(mut self) -> Self {
        self.flex_wrap = false;
        self
    }
    pub fn flex_1(mut self) -> Self {
        self.width = Size::Full;
        self
    }
    pub fn shrink_0(mut self) -> Self {
        if let Size::Px(width) = self.width {
            self.min_width = Some(width);
        }
        if let Size::Px(height) = self.height {
            self.min_height = Some(height);
        }
        self
    }
    pub fn flex_basis(mut self, basis: f32) -> Self {
        self.width = Size::Px(basis);
        self
    }
}
