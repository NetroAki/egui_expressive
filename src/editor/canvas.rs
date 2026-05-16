//! Thin canvas adapter for editor surfaces.

use crate::editor::{Axis, SnapGrid};
use crate::surface::{LargeCanvas, ViewportCuller};
use egui::{Color32, Id, Pos2, Rect, Response, Stroke, Ui, Vec2};

#[derive(Debug, Clone)]
pub struct EditorCanvas {
    id: Id,
    logical_size: Vec2,
    snap_grid: SnapGrid,
    x_axis: Option<Axis>,
    y_axis: Option<Axis>,
    min_zoom: f32,
    max_zoom: f32,
    scroll_enabled: bool,
    draw_grid: bool,
}

impl EditorCanvas {
    pub fn new(id: Id, logical_size: Vec2) -> Self {
        Self {
            id,
            logical_size,
            snap_grid: SnapGrid::disabled(),
            x_axis: None,
            y_axis: None,
            min_zoom: 0.01,
            max_zoom: 100.0,
            scroll_enabled: true,
            draw_grid: true,
        }
    }
    pub fn snap_grid(mut self, snap_grid: SnapGrid) -> Self {
        self.snap_grid = snap_grid;
        self
    }
    pub fn x_axis(mut self, axis: Axis) -> Self {
        self.x_axis = Some(axis);
        self
    }
    pub fn y_axis(mut self, axis: Axis) -> Self {
        self.y_axis = Some(axis);
        self
    }
    pub fn zoom_range(mut self, min: f32, max: f32) -> Self {
        self.min_zoom = min;
        self.max_zoom = max;
        self
    }
    pub fn scrollable(mut self, enabled: bool) -> Self {
        self.scroll_enabled = enabled;
        self
    }
    pub fn draw_grid(mut self, enabled: bool) -> Self {
        self.draw_grid = enabled;
        self
    }

    pub fn show(self, ui: &mut Ui, f: impl FnOnce(EditorCanvasContext<'_>)) -> Response {
        LargeCanvas::new(self.id, self.logical_size)
            .zoom_range(self.min_zoom, self.max_zoom)
            .scrollable(self.scroll_enabled)
            .show(ui, |ui, origin, pan_zoom, culler| {
                let context = EditorCanvasContext {
                    ui,
                    origin,
                    pan_zoom,
                    culler,
                    snap_grid: &self.snap_grid,
                    x_axis: self.x_axis.as_ref(),
                    y_axis: self.y_axis.as_ref(),
                };
                if self.draw_grid {
                    context.paint_grid();
                }
                f(context);
            })
    }
}

#[derive(Clone, Copy)]
pub struct EditorCanvasContext<'a> {
    pub ui: &'a Ui,
    pub origin: Pos2,
    pub pan_zoom: &'a crate::interaction::PanZoom,
    pub culler: &'a ViewportCuller,
    pub snap_grid: &'a SnapGrid,
    pub x_axis: Option<&'a Axis>,
    pub y_axis: Option<&'a Axis>,
}

impl EditorCanvasContext<'_> {
    pub fn to_screen(&self, logical: Pos2) -> Pos2 {
        self.culler.to_screen(logical)
    }
    pub fn rect_to_screen(&self, logical: Rect) -> Rect {
        self.culler.rect_to_screen(logical)
    }
    pub fn paint_grid(&self) {
        let painter = self.ui.painter();
        let viewport = self.culler.logical_viewport();
        let minor = Color32::from_rgba_unmultiplied(90, 100, 120, 55);
        let major = Color32::from_rgba_unmultiplied(120, 140, 170, 95);
        if let Some(x_step) = self.snap_grid.x_step.filter(|_| self.snap_grid.enabled) {
            let start = (viewport.left() / x_step).floor() as i32;
            let end = (viewport.right() / x_step).ceil() as i32;
            for i in start..=end {
                let x = i as f32 * x_step;
                let a = self.culler.to_screen(Pos2::new(x, viewport.top()));
                let b = self.culler.to_screen(Pos2::new(x, viewport.bottom()));
                let is_major = i % 4 == 0;
                painter.line_segment(
                    [a, b],
                    Stroke::new(
                        if is_major { 1.0 } else { 0.5 },
                        if is_major { major } else { minor },
                    ),
                );
            }
        }
        if let Some(y_step) = self.snap_grid.y_step.filter(|_| self.snap_grid.enabled) {
            let start = (viewport.top() / y_step).floor() as i32;
            let end = (viewport.bottom() / y_step).ceil() as i32;
            for i in start..=end {
                let y = i as f32 * y_step;
                let a = self.culler.to_screen(Pos2::new(viewport.left(), y));
                let b = self.culler.to_screen(Pos2::new(viewport.right(), y));
                let is_major = i % 4 == 0;
                painter.line_segment(
                    [a, b],
                    Stroke::new(
                        if is_major { 1.0 } else { 0.5 },
                        if is_major { major } else { minor },
                    ),
                );
            }
        }
    }
}
