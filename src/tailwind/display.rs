//! Display and overflow utilities for `Tw`.

use crate::layout::GridLayout;
use crate::tailwind::{
    types::{Display, FlexDirection, Items, Justify, Overflow},
    Tw,
};

impl Tw {
    pub fn block(mut self) -> Self {
        self.display = Display::Block;
        self
    }
    pub fn flex(mut self) -> Self {
        self.display = Display::Flex;
        self.flex_direction = FlexDirection::Row;
        self
    }
    pub fn flex_col(mut self) -> Self {
        self.display = Display::Flex;
        self.flex_direction = FlexDirection::Column;
        self
    }
    pub fn grid(mut self) -> Self {
        self.display = Display::Grid;
        self.grid.get_or_insert(GridLayout::columns(1));
        self
    }
    pub fn hidden(mut self) -> Self {
        self.display = Display::Hidden;
        self
    }
    pub fn overflow_hidden(mut self) -> Self {
        self.overflow = Overflow::Hidden;
        self
    }
    pub fn overflow_clip(mut self) -> Self {
        self.overflow = Overflow::Clip;
        self
    }
    pub fn overflow_auto(mut self) -> Self {
        self.overflow = Overflow::Auto;
        self
    }
    pub fn overflow_scroll(mut self) -> Self {
        self.overflow = Overflow::Scroll;
        self
    }
    pub fn justify_start(mut self) -> Self {
        self.justify = Justify::Start;
        self
    }
    pub fn justify_center(mut self) -> Self {
        self.justify = Justify::Center;
        self
    }
    pub fn justify_end(mut self) -> Self {
        self.justify = Justify::End;
        self
    }
    pub fn justify_between(mut self) -> Self {
        self.justify = Justify::Between;
        self
    }
    pub fn items_start(mut self) -> Self {
        self.items = Items::Start;
        self
    }
    pub fn items_center(mut self) -> Self {
        self.items = Items::Center;
        self
    }
    pub fn items_end(mut self) -> Self {
        self.items = Items::End;
        self
    }
    pub fn items_stretch(mut self) -> Self {
        self.items = Items::Stretch;
        self
    }
}
