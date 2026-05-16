//! Drawing helpers, effects, gradients, clipping, compositing, and transforms.

use egui::{
    epaint::{PathShape, PathStroke, RectShape, StrokeKind},
    Color32, CornerRadius, Id, LayerId, Order, Pos2, Rect, Shape, Stroke,
};

mod clipping;
mod color_icons;
mod composite_core;
mod composite_hash;
#[cfg(test)]
mod current_render_visual_proof_tests;
mod gradients;
mod painter_builders;
mod patterns;
mod raster_pixels;
mod rasterize;
mod shadows_images;
mod stack_tests;
mod strokes;
mod transform_clip_layout;

pub use clipping::*;
pub use color_icons::*;
pub use composite_core::*;
pub(crate) use composite_hash::*;
pub use gradients::*;
pub use painter_builders::*;
pub use patterns::*;
pub(crate) use raster_pixels::*;
pub(crate) use rasterize::*;
pub use shadows_images::*;
pub use stack_tests::*;
pub use strokes::*;
pub use transform_clip_layout::*;
