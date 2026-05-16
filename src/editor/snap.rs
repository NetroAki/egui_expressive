//! Coordinate snapping for editor surfaces.

use egui::{Pos2, Rect};

/// A pure snapping helper for editor coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapGrid {
    pub enabled: bool,
    pub x_step: Option<f32>,
    pub y_step: Option<f32>,
    pub origin: Pos2,
}

impl Default for SnapGrid {
    fn default() -> Self {
        Self::disabled()
    }
}

impl SnapGrid {
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            x_step: None,
            y_step: None,
            origin: Pos2::ZERO,
        }
    }

    pub fn uniform(step: f32) -> Self {
        Self::new(Some(step), Some(step))
    }

    pub fn new(x_step: Option<f32>, y_step: Option<f32>) -> Self {
        Self {
            enabled: true,
            x_step: x_step.filter(|step| *step > f32::EPSILON),
            y_step: y_step.filter(|step| *step > f32::EPSILON),
            origin: Pos2::ZERO,
        }
    }

    pub fn origin(mut self, origin: Pos2) -> Self {
        self.origin = origin;
        self
    }

    pub fn snap_x(&self, x: f32) -> f32 {
        snap_scalar(self.enabled, x, self.origin.x, self.x_step)
    }

    pub fn snap_y(&self, y: f32) -> f32 {
        snap_scalar(self.enabled, y, self.origin.y, self.y_step)
    }

    pub fn snap_pos(&self, pos: Pos2) -> Pos2 {
        Pos2::new(self.snap_x(pos.x), self.snap_y(pos.y))
    }

    pub fn snap_rect_min(&self, rect: Rect) -> Rect {
        Rect::from_min_size(self.snap_pos(rect.min), rect.size())
    }
}

fn snap_scalar(enabled: bool, value: f32, origin: f32, step: Option<f32>) -> f32 {
    if !enabled {
        return value;
    }
    let Some(step) = step else {
        return value;
    };
    if step <= f32::EPSILON {
        return value;
    }
    origin + ((value - origin) / step).round() * step
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snap_grid_snaps_positions_and_preserves_disabled_values() {
        let snap = SnapGrid::uniform(0.25);
        assert!((snap.snap_x(1.13) - 1.25).abs() < 0.0001);
        assert!((snap.snap_y(1.12) - 1.0).abs() < 0.0001);
        assert_eq!(SnapGrid::disabled().snap_x(1.13), 1.13);
    }
}
