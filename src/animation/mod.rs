//! Animation primitives: easing, tweens, springs, sequences, transitions.

use egui::{Color32, Context, Id};

mod animated_state;
mod easing;
mod memory;
mod sequence;
mod spring;
mod transition;
mod tween;

#[cfg(test)]
mod tests;

pub use animated_state::*;
pub use easing::*;
pub(crate) use memory::*;
pub use sequence::*;
pub use spring::*;
pub use transition::*;
pub use tween::*;
