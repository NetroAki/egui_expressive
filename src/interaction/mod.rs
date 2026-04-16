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

// ---------------------------------------------------------------------------
// Value utilities
// ---------------------------------------------------------------------------

/// Normalize a value from `range` to 0.0..=1.0.
pub fn normalize(value: f64, range: &std::ops::RangeInclusive<f64>) -> f32 {
    let min = *range.start();
    let max = *range.end();
    ((value - min) / (max - min)).clamp(0.0, 1.0) as f32
}

/// Denormalize a 0.0..=1.0 value back to `range`.
pub fn denormalize(t: f32, range: &std::ops::RangeInclusive<f64>) -> f64 {
    let min = *range.start();
    let max = *range.end();
    min + (t as f64) * (max - min)
}

// ---------------------------------------------------------------------------
// Gesture recognizers
// ---------------------------------------------------------------------------

/// Output of a tap gesture.
#[derive(Clone, Debug)]
pub struct TapEvent {
    pub pos: egui::Pos2,
    pub count: u32, // 1 = single tap, 2 = double tap
}

/// Output of a long-press gesture.
#[derive(Clone, Debug)]
pub struct LongPressEvent {
    pub pos: egui::Pos2,
}

/// Output of a swipe gesture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwipeDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, Debug)]
pub struct SwipeEvent {
    pub direction: SwipeDirection,
    pub velocity: egui::Vec2,
}

/// Recognizes a tap (click without significant drag).
///
/// Returns `Some(TapEvent)` on the frame the tap completes.
pub struct TapGesture {
    /// Maximum drag distance to still count as a tap. Default: 5.0px
    pub max_drag: f32,
    /// Number of taps required. Default: 1
    pub count: u32,
}

impl Default for TapGesture {
    fn default() -> Self {
        Self {
            max_drag: 5.0,
            count: 1,
        }
    }
}

impl TapGesture {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn double() -> Self {
        Self {
            count: 2,
            ..Default::default()
        }
    }
    pub fn max_drag(mut self, px: f32) -> Self {
        self.max_drag = px;
        self
    }

    /// Process a response. Returns `Some(TapEvent)` when the gesture fires.
    pub fn recognize(&self, response: &egui::Response) -> Option<TapEvent> {
        let fired = if self.count == 2 {
            response.double_clicked()
        } else {
            response.clicked()
        };
        if fired && response.drag_delta().length() < self.max_drag {
            Some(TapEvent {
                pos: response
                    .interact_pointer_pos()
                    .unwrap_or(response.rect.center()),
                count: self.count,
            })
        } else {
            None
        }
    }
}

/// Recognizes a long press (pointer held without significant movement).
pub struct LongPressGesture {
    /// Duration in seconds before firing. Default: 0.5s
    pub duration: f32,
    /// Maximum movement to still count as held. Default: 5.0px
    pub max_drag: f32,
}

impl Default for LongPressGesture {
    fn default() -> Self {
        Self {
            duration: 0.5,
            max_drag: 5.0,
        }
    }
}

impl LongPressGesture {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn duration(mut self, secs: f32) -> Self {
        self.duration = secs;
        self
    }

    /// Process a response. Returns `Some(LongPressEvent)` when the gesture fires.
    /// Requires an `Id` for storing press-start time in egui memory.
    pub fn recognize(
        &self,
        response: &egui::Response,
        ctx: &egui::Context,
        id: egui::Id,
    ) -> Option<LongPressEvent> {
        let press_start_id = id.with("__lp_start");
        let fired_id = id.with("__lp_fired");

        if response.is_pointer_button_down_on() && response.drag_delta().length() < self.max_drag {
            // Record press start time
            let start: f64 = ctx.data(|d| d.get_temp(press_start_id)).unwrap_or_else(|| {
                let t = ctx.input(|i| i.time);
                ctx.data_mut(|d| d.insert_temp(press_start_id, t));
                t
            });

            let elapsed = ctx.input(|i| i.time) - start;
            let already_fired: bool = ctx.data(|d| d.get_temp(fired_id)).unwrap_or(false);

            if elapsed >= self.duration as f64 && !already_fired {
                ctx.data_mut(|d| d.insert_temp(fired_id, true));
                return Some(LongPressEvent {
                    pos: response
                        .interact_pointer_pos()
                        .unwrap_or(response.rect.center()),
                });
            }
        } else {
            // Reset on release
            ctx.data_mut(|d| {
                d.remove::<f64>(press_start_id);
                d.remove::<bool>(fired_id);
            });
        }
        None
    }
}

/// Recognizes a swipe (fast drag in a cardinal direction).
pub struct SwipeGesture {
    /// Minimum velocity (px/s) to count as a swipe. Default: 200.0
    pub min_velocity: f32,
    /// Minimum distance (px). Default: 30.0
    pub min_distance: f32,
}

impl Default for SwipeGesture {
    fn default() -> Self {
        Self {
            min_velocity: 200.0,
            min_distance: 30.0,
        }
    }
}

impl SwipeGesture {
    pub fn new() -> Self {
        Self::default()
    }

    /// Process a response. Returns `Some(SwipeEvent)` on drag release.
    pub fn recognize(
        &self,
        response: &egui::Response,
        id: egui::Id,
        ctx: &egui::Context,
    ) -> Option<SwipeEvent> {
        let total_id = id.with("__sw_total");

        if response.dragged() {
            let prev: egui::Vec2 = ctx.data(|d| d.get_temp(total_id)).unwrap_or_default();
            ctx.data_mut(|d| d.insert_temp(total_id, prev + response.drag_delta()));
        }

        if response.drag_stopped() {
            let total: egui::Vec2 = ctx.data(|d| d.get_temp(total_id)).unwrap_or_default();
            ctx.data_mut(|d| d.remove::<egui::Vec2>(total_id));

            let dist = total.length();
            if dist < self.min_distance {
                return None;
            }

            let dt = ctx.input(|i| i.stable_dt);
            let velocity = if dt > 0.0 {
                total / dt
            } else {
                egui::Vec2::ZERO
            };
            if velocity.length() < self.min_velocity {
                return None;
            }

            let direction = if total.x.abs() > total.y.abs() {
                if total.x > 0.0 {
                    SwipeDirection::Right
                } else {
                    SwipeDirection::Left
                }
            } else {
                if total.y > 0.0 {
                    SwipeDirection::Down
                } else {
                    SwipeDirection::Up
                }
            };

            return Some(SwipeEvent {
                direction,
                velocity,
            });
        }
        None
    }
}

// ---------------------------------------------------------------------------
// FocusScope — keyboard navigation
// ---------------------------------------------------------------------------

/// Manages keyboard focus across a group of widgets.
///
/// Tab moves focus forward, Shift+Tab moves backward.
/// Widgets register themselves with `register()` and check `is_focused()`.
///
/// # Example
/// ```rust,ignore
/// let scope = FocusScope::new(ui.id().with("form"));
/// scope.register(ui, email_id);
/// scope.register(ui, password_id);
/// scope.handle_tab(ui);
/// if scope.is_focused(ui, email_id) {
///     // draw focus ring
/// }
/// ```
pub struct FocusScope {
    id: egui::Id,
}

impl FocusScope {
    pub fn new(id: impl std::hash::Hash) -> Self {
        Self {
            id: egui::Id::new(id),
        }
    }

    /// Register a widget ID in this scope's tab order.
    /// Call in the order you want Tab to cycle through.
    pub fn register(&self, ctx: &egui::Context, widget_id: egui::Id) {
        let order_id = self.id.with("__fs_order");
        let mut order: Vec<egui::Id> = ctx.data(|d| d.get_temp(order_id)).unwrap_or_default();
        if !order.contains(&widget_id) {
            order.push(widget_id);
            ctx.data_mut(|d| d.insert_temp(order_id, order));
        }
    }

    /// Process Tab/Shift+Tab to advance focus. Call once per frame.
    pub fn handle_tab(&self, ctx: &egui::Context) {
        let tab_pressed = ctx.input(|i| i.key_pressed(egui::Key::Tab));
        if !tab_pressed {
            return;
        }

        let shift = ctx.input(|i| i.modifiers.shift);
        let order_id = self.id.with("__fs_order");
        let focused_id = self.id.with("__fs_focused");

        let order: Vec<egui::Id> = ctx.data(|d| d.get_temp(order_id)).unwrap_or_default();
        if order.is_empty() {
            return;
        }

        let current: Option<egui::Id> = ctx.data(|d| d.get_temp(focused_id));
        let next = match current {
            None => order[0],
            Some(cur) => {
                let pos = order.iter().position(|&id| id == cur).unwrap_or(0);
                if shift {
                    order[(pos + order.len() - 1) % order.len()]
                } else {
                    order[(pos + 1) % order.len()]
                }
            }
        };
        ctx.data_mut(|d| d.insert_temp(focused_id, next));
    }

    /// Returns true if the given widget ID currently has focus in this scope.
    pub fn is_focused(&self, ctx: &egui::Context, widget_id: egui::Id) -> bool {
        let focused_id = self.id.with("__fs_focused");
        ctx.data(|d| d.get_temp::<egui::Id>(focused_id))
            .map(|id| id == widget_id)
            .unwrap_or(false)
    }

    /// Programmatically set focus to a widget.
    pub fn focus(&self, ctx: &egui::Context, widget_id: egui::Id) {
        ctx.data_mut(|d| d.insert_temp(self.id.with("__fs_focused"), widget_id));
    }

    /// Clear focus from all widgets in this scope.
    pub fn clear_focus(&self, ctx: &egui::Context) {
        ctx.data_mut(|d| d.remove::<egui::Id>(self.id.with("__fs_focused")));
    }
}
