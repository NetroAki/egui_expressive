//! Tailwind/CSS-recognizable utility styling for egui.
//!
//! `Tw` is a fluent utility builder for common CSS/Tailwind concepts: box model,
//! colors, borders, rounding, typography, sizing, opacity, and elevation.
//! Implementation is split by concern so designers and contributors can quickly
//! find the right file:
//!
//! - [`spacing`] — edge values and Tailwind spacing constants.
//! - [`shadow`] — elevation-to-frame-shadow conversion.
//! - `builder` — the public [`Tw`] style builder implementation.
//! - [`typography`] — text size, weight, and tracking utilities.
//! - [`border`] — border width and corner radius utilities.
//! - [`state`] — state-variant resolver for hover/focus/pressed work.
//! - [`position`] — absolute/relative/inset/translate/z-index utilities.

pub mod border;
pub mod box_model;
mod builder;
pub mod color;
pub mod display;
pub mod effects;
#[cfg(feature = "wgpu")]
pub(crate) mod exact_effects;
pub mod flex_child;
pub mod grid_intent;
pub mod interaction;
pub mod position;
pub mod render;
pub mod responsive;
pub mod shadow;
pub mod sizing;
pub mod spacing;
pub mod state;
pub mod theme_tokens;
pub mod types;
pub mod typography;
pub mod variants;

pub use border::{BorderEdges, BorderSide};
pub use builder::Tw;
pub use responsive::ResponsiveTw;
pub use spacing::{
    Edges, TW_0, TW_1, TW_10, TW_12, TW_16, TW_2, TW_20, TW_24, TW_3, TW_32, TW_4, TW_40, TW_48,
    TW_5, TW_6, TW_64, TW_8,
};
pub use state::{TwVariant, TwVariants};
pub use theme_tokens::{AccentKind, ColorToken, SurfaceLevel, TwThemeVariants};
pub use types::{
    Display, FlexDirection, FontWeight, GradientDirection, Items, Justify, Overflow, RadiusCorners,
    SelectionStyle, Size, TwBackdropSource, TwDropShadow, TwGradient, TwRing, TwTransition,
};
