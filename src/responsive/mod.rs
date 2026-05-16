//! Responsive viewport and container helpers.
//!
//! This module gives Rust/egui code the familiar Tailwind breakpoint vocabulary
//! (`sm`, `md`, `lg`, `xl`, `2xl`) while staying immediate-mode and explicit.

mod breakpoints;
mod context;
mod value;

pub use breakpoints::{BreakpointName, Breakpoints};
pub use context::{
    breakpoint_for_width, container_breakpoint, viewport_breakpoint, viewport_width,
};
pub use value::Responsive;
