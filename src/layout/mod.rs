//! Declarative layout DSL for Figma-style Auto Layout in egui.
//!
//! The module is intentionally split by layout concept:
//! helpers, stack macros, flex containers, grid values, and position values.

mod flex;
mod grid;
mod helpers;
mod position;
mod stack;

pub use flex::{FlexAlign, FlexContainer, FlexJustify, FlexSize};
pub use grid::{GridLayout, GridSpan};
pub use helpers::{aspect_ratio_fit, auto_layout, hrule, styled_frame, vrule};
pub use position::{Insets, PositionMode, PositionStyle};
