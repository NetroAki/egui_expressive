#![allow(dead_code)]

//! Large canvas viewport culling support (50k+ px).
//!
//! ## Overview
//!
//! This module provides helpers for rendering large virtual canvases that exceed
//! the screen size, using viewport culling to only draw what's visible.
//!
//! - [`ViewportCuller`] - Computes which portions of a logical canvas are visible
//!   on screen, handling pan/zoom transformations.
//! - [`LargeCanvas`] - High-level widget that manages pan/zoom state and calls
//!   a custom painter function with culling context.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use egui_expressive::surface::{LargeCanvas, ViewportCuller};
//!
//! fn my_canvas(ui: &mut egui::Ui) {
//!     LargeCanvas::new(ui.id().with("canvas"), egui::vec2(10000.0, 10000.0))
//!         .zoom_range(0.1, 10.0)
//!         .show(ui, |ui, origin, pan_zoom, culler| {
//!             // Draw your content here, culler can be used for viewport culling
//!         });
//! }
//! ```

use crate::interaction::PanZoom;
use egui::{Id, Pos2, Rect, Response, Ui, Vec2};

// ---------------------------------------------------------------------------
// ViewportCuller
// ---------------------------------------------------------------------------

/// Viewport culling helper that transforms between screen and logical coordinates.
///
/// `ViewportCuller` tracks the relationship between a logical (virtual) canvas
/// and its screen representation, accounting for pan and zoom. It provides
/// efficient culling by determining which content is actually visible.
///
/// # Coordinate Systems
///
/// - **Screen space**: Pixel coordinates on screen (output buffer)
/// - **Logical space**: Virtual coordinates in the canvas (e.g., 0..10000)
///
/// When zoomed and panned, a portion of logical space maps to the screen viewport.
#[derive(Debug, Clone)]
pub struct ViewportCuller {
    /// Visible screen area (in screen coordinates).
    screen_viewport: Rect,
    /// Visible area in logical coordinates.
    logical_viewport: Rect,
    /// Pan/zoom state for coordinate transformation.
    pan_zoom: PanZoom,
    /// Screen origin (top-left of the canvas area).
    origin: Pos2,
}

impl ViewportCuller {
    /// Create a new viewport culler.
    ///
    /// # Arguments
    /// * `screen_viewport` - Visible area in screen coordinates
    /// * `pan_zoom` - Current pan/zoom state
    /// * `origin` - Top-left corner of the canvas in screen coordinates
    pub fn new(screen_viewport: Rect, pan_zoom: PanZoom, origin: Pos2) -> Self {
        // Compute logical viewport from screen viewport using pan/zoom
        let lv_min = pan_zoom.to_logical(screen_viewport.min, origin);
        let lv_max = pan_zoom.to_logical(screen_viewport.max, origin);

        // Ensure min/max are properly ordered
        let logical_viewport = Rect::from_min_max(
            Pos2::new(lv_min.x.min(lv_max.x), lv_min.y.min(lv_max.y)),
            Pos2::new(lv_min.x.max(lv_max.x), lv_min.y.max(lv_max.y)),
        );

        Self {
            screen_viewport,
            logical_viewport,
            pan_zoom,
            origin,
        }
    }

    /// Check if a logical rectangle is visible (intersects the logical viewport).
    ///
    /// Use this for culling - rectangles that return `false` are entirely
    /// off-screen and don't need to be rendered.
    ///
    /// # Arguments
    /// * `logical_rect` - Rectangle in logical coordinates to test
    pub fn is_visible(&self, logical_rect: Rect) -> bool {
        self.logical_viewport.intersects(logical_rect)
    }

    /// Transform a logical position to screen coordinates.
    ///
    /// # Arguments
    /// * `logical` - Position in logical coordinates
    pub fn to_screen(&self, logical: Pos2) -> Pos2 {
        self.pan_zoom.to_screen(logical, self.origin)
    }

    /// Transform a screen position to logical coordinates.
    ///
    /// # Arguments
    /// * `screen` - Position in screen coordinates
    pub fn to_logical(&self, screen: Pos2) -> Pos2 {
        self.pan_zoom.to_logical(screen, self.origin)
    }

    /// Transform a logical rectangle to screen coordinates.
    ///
    /// Converts both corners and returns the minimal screen rectangle
    /// that contains the logical rectangle.
    ///
    /// # Arguments
    /// * `logical` - Rectangle in logical coordinates
    pub fn rect_to_screen(&self, logical: Rect) -> Rect {
        let min_screen = self.to_screen(logical.min);
        let max_screen = self.to_screen(logical.max);

        Rect::from_min_max(
            Pos2::new(
                min_screen.x.min(max_screen.x),
                min_screen.y.min(max_screen.y),
            ),
            Pos2::new(
                min_screen.x.max(max_screen.x),
                min_screen.y.max(max_screen.y),
            ),
        )
    }

    /// Compute the range of visible rows for a virtualized list.
    ///
    /// Assumes rows are laid out vertically with `row_height` spacing,
    /// numbered from 0. Returns the range of row indices that intersect
    /// the logical viewport.
    ///
    /// # Arguments
    /// * `row_height` - Height of each row in logical units
    /// * `total_rows` - Total number of rows in the list
    pub fn visible_rows(&self, row_height: f32, total_rows: usize) -> std::ops::Range<usize> {
        if total_rows == 0 || row_height <= 0.0 {
            return 0..0;
        }

        let viewport_min_y = self.logical_viewport.min.y;
        let viewport_max_y = self.logical_viewport.max.y;

        // Compute first visible row (may be negative if scrolled past start)
        let top = (viewport_min_y / row_height).floor().max(0.0) as usize;

        // Compute last visible row (capped at total)
        let bottom = ((viewport_max_y / row_height).ceil() as usize).min(total_rows);

        top..bottom
    }

    /// Compute the range of visible columns for a virtualized list.
    ///
    /// Assumes columns are laid out horizontally with `col_width` spacing,
    /// numbered from 0. Returns the range of column indices that intersect
    /// the logical viewport.
    ///
    /// # Arguments
    /// * `col_width` - Width of each column in logical units
    /// * `total_cols` - Total number of columns in the list
    pub fn visible_cols(&self, col_width: f32, total_cols: usize) -> std::ops::Range<usize> {
        if total_cols == 0 || col_width <= 0.0 {
            return 0..0;
        }

        let viewport_min_x = self.logical_viewport.min.x;
        let viewport_max_x = self.logical_viewport.max.x;

        // Compute first visible column
        let left = (viewport_min_x / col_width).floor().max(0.0) as usize;

        // Compute last visible column (capped at total)
        let right = ((viewport_max_x / col_width).ceil() as usize).min(total_cols);

        left..right
    }

    /// Get the logical viewport rectangle.
    pub fn logical_viewport(&self) -> Rect {
        self.logical_viewport
    }
}

// ---------------------------------------------------------------------------
// LargeCanvas
// ---------------------------------------------------------------------------

/// A large virtual canvas with automatic pan/zoom and viewport culling.
///
/// `LargeCanvas` manages the complexity of rendering a canvas that is much
/// larger than the screen, including:
///
/// - Persistent pan/zoom state stored in egui memory
/// - Automatic viewport culling computation
/// - Coordinate transformation helpers
///
/// # Example
///
/// ```rust,no_run
/// use egui_expressive::surface::LargeCanvas;
///
/// fn my_canvas(ui: &mut egui::Ui) {
///     LargeCanvas::new(ui.id().with("main_canvas"), egui::vec2(50000.0, 50000.0))
///         .zoom_range(0.01, 100.0)
///         .show(ui, |ui, origin, pan_zoom, culler| {
///             // Draw your large canvas content here
///             // Use culler.is_visible() to skip off-screen content
///             let painter = ui.painter();
///
///             // Example: draw a grid
///             for row in culler.visible_rows(50.0, 1000) {
///                 for col in culler.visible_cols(50.0, 1000) {
///                     let x = col as f32 * 50.0;
///                     let y = row as f32 * 50.0;
///                     let rect = egui::Rect::from_min_size(
///                         culler.to_screen(egui::pos2(x, y)),
///                         egui::vec2(48.0, 48.0)
///                     );
///                     painter.rect_filled(rect, 0.0, egui::Color32::DARK_GRAY);
///                 }
///             }
///         });
/// }
/// ```
#[derive(Debug, Clone)]
pub struct LargeCanvas {
    /// Unique identifier for this canvas.
    id: Id,
    /// Size of the logical canvas in units.
    logical_size: Vec2,
    /// Minimum zoom level.
    min_zoom: f32,
    /// Maximum zoom level.
    max_zoom: f32,
    /// Whether scroll/pan interactions are enabled.
    scroll_enabled: bool,
}

impl LargeCanvas {
    /// Create a new large canvas with the specified logical size.
    ///
    /// # Arguments
    /// * `id` - Unique identifier (use `ui.id().with("name")`)
    /// * `logical_size` - Size of the virtual canvas in logical units
    pub fn new(id: Id, logical_size: Vec2) -> Self {
        Self {
            id,
            logical_size,
            min_zoom: 0.01,
            max_zoom: 100.0,
            scroll_enabled: true,
        }
    }

    /// Set the zoom range (clamp range).
    ///
    /// # Arguments
    /// * `min` - Minimum zoom level (e.g., 0.1 for 10%)
    /// * `max` - Maximum zoom level (e.g., 10.0 for 1000%)
    pub fn zoom_range(mut self, min: f32, max: f32) -> Self {
        self.min_zoom = min;
        self.max_zoom = max;
        self
    }

    /// Enable or disable scroll/pan interactions.
    ///
    /// When disabled, the canvas stays fixed and doesn't respond to
    /// drag or scroll events.
    ///
    /// # Arguments
    /// * `enabled` - Whether scroll/pan is enabled
    pub fn scrollable(mut self, enabled: bool) -> Self {
        self.scroll_enabled = enabled;
        self
    }

    /// Show the canvas and invoke the painter callback.
    ///
    /// This method:
    /// 1. Allocates the available UI area
    /// 2. Loads/initializes pan-zoom state from memory
    /// 3. Handles pointer input for pan/zoom
    /// 4. Creates a `ViewportCuller` with current transform
    /// 5. Calls `f` with the painter context and culler
    ///
    /// # Arguments
    /// * `ui` - egui UI to draw within
    /// * `f` - Callback receiving `(ui, origin, pan_zoom, culler)`
    ///
    /// The callback should use the provided `culler` to skip drawing
    /// content that is outside the visible viewport.
    pub fn show(
        self,
        ui: &mut Ui,
        f: impl FnOnce(&Ui, Pos2, &PanZoom, &ViewportCuller),
    ) -> Response {
        let available_rect = ui.available_rect_before_wrap();

        // Allocate the full area with drag interaction for pan
        let sense = if self.scroll_enabled {
            egui::Sense::drag()
        } else {
            egui::Sense::hover()
        };

        let response = ui.allocate_rect(available_rect, sense);
        let screen_viewport = response.rect;
        let response_rect = response.rect;
        let _ui_clip_rect = ui.clip_rect();

        // Now borrow of ui is released, we can use ctx freely
        let ctx = ui.ctx();

        // Load or initialize pan/zoom state from memory
        let pz_id = self.id.with("__pz");
        let mut pan_zoom: PanZoom = ctx
            .memory(|m| m.data.get_temp(pz_id))
            .unwrap_or_else(PanZoom::new);

        // Handle pan/zoom interactions
        if self.scroll_enabled {
            let handle_id = self.id.with("__pz_handle");
            pan_zoom.handle(
                ctx,
                handle_id,
                &response,
                self.min_zoom..=self.max_zoom,
                true,
            );
        }

        // Save pan/zoom state to memory
        ctx.memory_mut(|m| m.data.insert_temp(pz_id, pan_zoom.clone()));

        // Create viewport culler
        let culler = ViewportCuller::new(screen_viewport, pan_zoom.clone(), response_rect.min);

        // Call the user's painter function
        // Note: we pass the response rect for clip, user should respect it
        f(ui, response_rect.min, &pan_zoom, &culler);

        response
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_culler_creation() {
        let screen_vp = Rect::from_min_size(Pos2::ZERO, Vec2::splat(100.0));
        let pan_zoom = PanZoom::new();
        let origin = Pos2::ZERO;

        let culler = ViewportCuller::new(screen_vp, pan_zoom, origin);

        // At scale 1.0 with no offset, logical and screen should match
        assert!((culler.logical_viewport.min.x - 0.0).abs() < 1e-6);
        assert!((culler.logical_viewport.min.y - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_viewport_culler_visible_rows() {
        let screen_vp = Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 200.0));
        let pan_zoom = PanZoom::new();
        let origin = Pos2::ZERO;

        let culler = ViewportCuller::new(screen_vp, pan_zoom, origin);

        // Row height 50, visible rows should be 0..4 (200/50)
        let rows = culler.visible_rows(50.0, 100);
        assert_eq!(rows.start, 0);
        assert_eq!(rows.end, 4);
    }

    #[test]
    fn test_viewport_culler_visible_cols() {
        let screen_vp = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 100.0));
        let pan_zoom = PanZoom::new();
        let origin = Pos2::ZERO;

        let culler = ViewportCuller::new(screen_vp, pan_zoom, origin);

        // Col width 50, visible cols should be 0..4 (200/50)
        let cols = culler.visible_cols(50.0, 100);
        assert_eq!(cols.start, 0);
        assert_eq!(cols.end, 4);
    }

    #[test]
    fn test_viewport_culler_is_visible() {
        let screen_vp = Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 100.0));
        let pan_zoom = PanZoom::new();
        let origin = Pos2::ZERO;

        let culler = ViewportCuller::new(screen_vp, pan_zoom, origin);

        // Within viewport
        let inside = Rect::from_min_size(Pos2::new(10.0, 10.0), Vec2::splat(20.0));
        assert!(culler.is_visible(inside));

        // Outside viewport
        let outside = Rect::from_min_size(Pos2::new(500.0, 500.0), Vec2::splat(20.0));
        assert!(!culler.is_visible(outside));

        // Partial overlap
        let partial = Rect::from_min_size(Pos2::new(90.0, 90.0), Vec2::splat(50.0));
        assert!(culler.is_visible(partial));
    }

    #[test]
    fn test_viewport_culler_transform_roundtrip() {
        let screen_vp = Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 100.0));
        let pan_zoom = PanZoom::new();
        let origin = Pos2::ZERO;

        let culler = ViewportCuller::new(screen_vp, pan_zoom, origin);

        let logical = Pos2::new(50.0, 75.0);
        let screen = culler.to_screen(logical);
        let back = culler.to_logical(screen);

        assert!((back.x - logical.x).abs() < 1e-6);
        assert!((back.y - logical.y).abs() < 1e-6);
    }

    #[test]
    fn test_large_canvas_builder() {
        let canvas = LargeCanvas::new(egui::Id::new("test"), egui::vec2(1000.0, 2000.0))
            .zoom_range(0.1, 5.0)
            .scrollable(true);

        assert!((canvas.min_zoom - 0.1).abs() < 1e-6);
        assert!((canvas.max_zoom - 5.0).abs() < 1e-6);
        assert!(canvas.scroll_enabled);
    }
}
