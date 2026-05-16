use super::render::normalized_pair;
use crate::widgets::knobs::Orientation;
use egui::{Pos2, Rect, Response, Sense, Ui, Vec2};
use std::ops::RangeInclusive;

/// Dual-thumb range slider for selections, loop regions, filters, and fade ranges.
pub struct RangeSlider<'a> {
    start: &'a mut f64,
    end: &'a mut f64,
    range: RangeInclusive<f64>,
    size: Vec2,
    orientation: Orientation,
}

impl<'a> RangeSlider<'a> {
    pub fn new(start: &'a mut f64, end: &'a mut f64, range: RangeInclusive<f64>) -> Self {
        Self {
            start,
            end,
            range,
            size: Vec2::new(160.0, 24.0),
            orientation: Orientation::Horizontal,
        }
    }
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
    pub fn normalized_range(&self) -> (f32, f32) {
        normalized_pair(*self.start, *self.end, &self.range)
    }
}

impl<'a> egui::Widget for RangeSlider<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::click_and_drag());
        let min = *self.range.start();
        let max = *self.range.end();
        let span = (max - min).max(f64::EPSILON);
        let (mut a, mut b) = normalized_pair(*self.start, *self.end, &self.range);
        if (response.dragged() || response.clicked()) && response.interact_pointer_pos().is_some() {
            let pos = response.interact_pointer_pos().unwrap();
            let t = match self.orientation {
                Orientation::Horizontal => ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0),
                Orientation::Vertical => {
                    (1.0 - (pos.y - rect.min.y) / rect.height()).clamp(0.0, 1.0)
                }
            };
            if (t - a).abs() <= (t - b).abs() {
                a = t.min(b);
                *self.start = min + span * a as f64;
            } else {
                b = t.max(a);
                *self.end = min + span * b as f64;
            }
        }
        let visuals = ui.visuals();
        ui.painter()
            .rect_filled(rect, 4.0, visuals.widgets.inactive.bg_fill);
        let selected = match self.orientation {
            Orientation::Horizontal => Rect::from_min_max(
                Pos2::new(
                    rect.min.x + a * rect.width(),
                    rect.min.y + rect.height() * 0.35,
                ),
                Pos2::new(
                    rect.min.x + b * rect.width(),
                    rect.max.y - rect.height() * 0.35,
                ),
            ),
            Orientation::Vertical => Rect::from_min_max(
                Pos2::new(
                    rect.min.x + rect.width() * 0.35,
                    rect.max.y - b * rect.height(),
                ),
                Pos2::new(
                    rect.max.x - rect.width() * 0.35,
                    rect.max.y - a * rect.height(),
                ),
            ),
        };
        ui.painter()
            .rect_filled(selected, 3.0, visuals.selection.bg_fill);
        for t in [a, b] {
            let center = match self.orientation {
                Orientation::Horizontal => {
                    Pos2::new(rect.min.x + t * rect.width(), rect.center().y)
                }
                Orientation::Vertical => Pos2::new(rect.center().x, rect.max.y - t * rect.height()),
            };
            ui.painter()
                .circle_filled(center, 5.0, visuals.selection.stroke.color);
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_range_orders_values() {
        let mut start = 0.8;
        let mut end = 0.2;
        let slider = RangeSlider::new(&mut start, &mut end, 0.0..=1.0);

        assert_eq!(slider.normalized_range(), (0.2, 0.8));
    }
}
