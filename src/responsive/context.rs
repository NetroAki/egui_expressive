//! Helpers for deriving breakpoints from egui viewport or container sizes.

use egui::{Context, Ui};

use super::{BreakpointName, Breakpoints};

/// Classify an arbitrary width with the supplied breakpoints.
pub fn breakpoint_for_width(width: f32, breakpoints: Breakpoints) -> BreakpointName {
    breakpoints.classify(width)
}

/// Current viewport content width in points.
pub fn viewport_width(ctx: &Context) -> f32 {
    ctx.input(|input| input.content_rect().width())
}

/// Current viewport breakpoint.
pub fn viewport_breakpoint(ctx: &Context, breakpoints: Breakpoints) -> BreakpointName {
    breakpoints.classify(viewport_width(ctx))
}

/// Breakpoint for the current container's available width.
pub fn container_breakpoint(ui: &Ui, breakpoints: Breakpoints) -> BreakpointName {
    breakpoints.classify(ui.available_width())
}
