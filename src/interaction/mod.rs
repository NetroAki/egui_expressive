#![allow(dead_code)]

//! Interaction helpers: drag, pan/zoom, and gestures.

use egui::{Context, Modifiers, Pos2, Response, Vec2};

/// Horizontal, vertical, or free drag axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragAxis {
    X,
    Y,
    Free,
}

/// Drag delta information with accumulation and velocity tracking.
#[derive(Debug, Clone, Copy, Default)]
pub struct DragDelta {
    pub delta: Vec2,
    pub total: Vec2,
    pub velocity: Vec2,
    pub started: bool,
    pub released: bool,
    pub modifiers: Modifiers,
}

impl DragDelta {
    /// Construct `DragDelta` from a drag response.
    pub fn from_response(ctx: &Context, id: egui::Id, response: &Response, axis: DragAxis) -> Self {
        let total_id = id.with("__drag_total");
        let vel_id = id.with("__drag_vel");

        let prev_total: Vec2 = ctx
            .memory(|m| m.data.get_temp(total_id))
            .unwrap_or_default();
        let mut velocity: Vec2 = ctx.memory(|m| m.data.get_temp(vel_id)).unwrap_or_default();

        let delta = response.drag_delta();
        let mut total = prev_total;
        let mut started = false;
        let mut released = false;

        if response.drag_started() {
            total = Vec2::ZERO;
            started = true;
        }

        if response.dragged() {
            total += delta;
        }

        if response.drag_stopped() {
            released = true;
            total = Vec2::ZERO;
        }

        // Apply axis constraint before updating velocity
        let constrained_delta = match axis {
            DragAxis::X => Vec2::new(delta.x, 0.0),
            DragAxis::Y => Vec2::new(0.0, delta.y),
            DragAxis::Free => delta,
        };

        // Update velocity with EMA smoothing
        velocity = velocity * 0.8 + constrained_delta * 0.2;

        // Apply axis constraint to total
        let constrained_total = match axis {
            DragAxis::X => Vec2::new(total.x, 0.0),
            DragAxis::Y => Vec2::new(0.0, total.y),
            DragAxis::Free => total,
        };

        // Store updated state
        if response.drag_started() || response.dragged() || response.drag_stopped() {
            ctx.memory_mut(|m| m.data.insert_temp(total_id, constrained_total));
            ctx.memory_mut(|m| m.data.insert_temp(vel_id, velocity));
        }

        let modifiers = ctx.input(|i| i.modifiers);

        Self {
            delta: constrained_delta,
            total: constrained_total,
            velocity,
            started,
            released,
            modifiers,
        }
    }

    /// Scale all vector fields by a factor.
    pub fn scaled(self, factor: f32) -> Self {
        Self {
            delta: self.delta * factor,
            total: self.total * factor,
            velocity: self.velocity * factor,
            started: self.started,
            released: self.released,
            modifiers: self.modifiers,
        }
    }
}

/// Pan/zoom state for viewport navigation.
#[derive(Debug, Clone)]
pub struct PanZoom {
    pub offset: Vec2,
    pub scale: f32,
}

impl PanZoom {
    /// Create a new pan/zoom state at origin with unit scale.
    pub fn new() -> Self {
        Self {
            offset: Vec2::ZERO,
            scale: 1.0,
        }
    }

    /// Handle pan and zoom from a pointer response.
    pub fn handle(
        &mut self,
        ctx: &Context,
        _id: egui::Id,
        response: &Response,
        scale_range: std::ops::RangeInclusive<f32>,
        zoom_to_cursor: bool,
    ) {
        // Pan: primary button drag
        if response.dragged() {
            let drag_delta = response.drag_delta();
            self.offset -= drag_delta / self.scale;
        }

        // Zoom via scroll
        let scroll_delta = ctx.input(|i| i.smooth_scroll_delta);
        let zoom_delta = ctx.input(|i| i.zoom_delta());

        let mut scale_factor = 1.0;

        // Scroll zoom (vertical scroll gives zoom)
        if scroll_delta.y != 0.0 {
            let scroll_factor = (scroll_delta.y * 0.002).exp();
            scale_factor *= scroll_factor;
        }

        // Zoom delta from pinch gesture
        if zoom_delta != 1.0 {
            scale_factor *= zoom_delta;
        }

        if scale_factor != 1.0 {
            let new_scale =
                (self.scale * scale_factor).clamp(*scale_range.start(), *scale_range.end());

            // If zooming to cursor, adjust offset to keep cursor position fixed
            if zoom_to_cursor {
                if let Some(cursor_screen) = ctx.input(|i| i.pointer.hover_pos()) {
                    let cursor_logical = self.to_logical(cursor_screen, Pos2::ZERO);
                    let screen_offset = (cursor_screen - Pos2::ZERO) / new_scale;
                    self.offset = cursor_logical.to_vec2() - screen_offset;
                }
            }

            self.scale = new_scale;
        }
    }

    /// Transform a logical position to screen coordinates.
    pub fn to_screen(&self, logical: Pos2, origin: Pos2) -> Pos2 {
        origin + (logical.to_vec2() - self.offset) * self.scale
    }

    /// Transform a screen position to logical coordinates.
    pub fn to_logical(&self, screen: Pos2, origin: Pos2) -> Pos2 {
        let vec = (screen - origin) / self.scale + self.offset;
        Pos2::new(vec.x, vec.y)
    }
}

impl Default for PanZoom {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute a value delta from pixel drag amount.
pub fn drag_to_value_delta(
    drag_pixels: f32,
    range: std::ops::RangeInclusive<f64>,
    pixels_per_range: f32,
    modifiers: &Modifiers,
) -> f64 {
    let range_span = range.end() - range.start();
    let multiplier = if modifiers.shift { 0.1 } else { 1.0 };
    (drag_pixels as f64 / pixels_per_range as f64) * range_span * multiplier
}

/// Check if a key was pressed this frame with exact modifier matching.
pub fn key_pressed(ctx: &Context, key: egui::Key, modifiers: Modifiers) -> bool {
    ctx.input(|i| i.key_pressed(key) && i.modifiers == modifiers)
}
