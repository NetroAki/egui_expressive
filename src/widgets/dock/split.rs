use egui::{Color32, Id, Response, Sense, Ui, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DockZone {
    Left,
    Right,
    Top,
    Bottom,
    Center,
}

pub struct ResizableSplit<'a> {
    id: Id,
    fraction: &'a mut f32,
    axis: SplitAxis,
}

impl<'a> ResizableSplit<'a> {
    pub fn new(id: impl std::hash::Hash, fraction: &'a mut f32, axis: SplitAxis) -> Self {
        Self {
            id: Id::new(id),
            fraction,
            axis,
        }
    }

    pub fn show(
        self,
        ui: &mut Ui,
        first: impl FnOnce(&mut Ui),
        second: impl FnOnce(&mut Ui),
    ) -> Response {
        let available = ui.available_size_before_wrap();
        match self.axis {
            SplitAxis::Horizontal => self.show_horizontal(ui, available, first, second),
            SplitAxis::Vertical => self.show_vertical(ui, available, first, second),
        }
    }

    fn show_horizontal(
        self,
        ui: &mut Ui,
        available: Vec2,
        first: impl FnOnce(&mut Ui),
        second: impl FnOnce(&mut Ui),
    ) -> Response {
        let handle = 6.0;
        let total = available.x.max(handle + 2.0);
        let first_w = ((total - handle) * clamped_split_fraction(*self.fraction)).max(1.0);
        let second_w = (total - handle - first_w).max(1.0);
        let height = available.y.max(24.0);
        let mut handle_response = None;
        ui.horizontal(|ui| {
            ui.allocate_ui(Vec2::new(first_w, height), first);
            let response = resize_handle(ui, self.id.with("handle"), Vec2::new(handle, height));
            if response.dragged() {
                let delta = response.drag_delta().x / (total - handle).max(1.0);
                *self.fraction = clamped_split_fraction(*self.fraction + delta);
            }
            handle_response = Some(response);
            ui.allocate_ui(Vec2::new(second_w, height), second);
        });
        handle_response.unwrap_or_else(|| ui.allocate_response(Vec2::ZERO, Sense::hover()))
    }

    fn show_vertical(
        self,
        ui: &mut Ui,
        available: Vec2,
        first: impl FnOnce(&mut Ui),
        second: impl FnOnce(&mut Ui),
    ) -> Response {
        let handle = 6.0;
        let total = available.y.max(handle + 2.0);
        let first_h = ((total - handle) * clamped_split_fraction(*self.fraction)).max(1.0);
        let second_h = (total - handle - first_h).max(1.0);
        let width = available.x.max(24.0);
        ui.allocate_ui(Vec2::new(width, first_h), first);
        let response = resize_handle(ui, self.id.with("handle"), Vec2::new(width, handle));
        if response.dragged() {
            let delta = response.drag_delta().y / (total - handle).max(1.0);
            *self.fraction = clamped_split_fraction(*self.fraction + delta);
        }
        ui.allocate_ui(Vec2::new(width, second_h), second);
        response
    }
}

fn resize_handle(ui: &mut Ui, id: Id, size: Vec2) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::drag());
    let response = ui.interact(rect, id, Sense::drag()).union(response);
    ui.painter()
        .rect_filled(response.rect, 2.0, Color32::from_gray(70));
    response
}

pub fn clamped_split_fraction(fraction: f32) -> f32 {
    fraction.clamp(0.1, 0.9)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_fraction_clamps_to_safe_bounds() {
        assert_eq!(clamped_split_fraction(-1.0), 0.1);
        assert_eq!(clamped_split_fraction(2.0), 0.9);
        assert_eq!(clamped_split_fraction(0.4), 0.4);
    }
}
