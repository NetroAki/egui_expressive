//! Marquee/rubber-band selection helpers.

use egui::{Pos2, Rect};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MarqueeSelection {
    start: Option<Pos2>,
    current: Option<Pos2>,
}

impl MarqueeSelection {
    pub fn begin(&mut self, start: Pos2) {
        self.start = Some(start);
        self.current = Some(start);
    }
    pub fn update(&mut self, current: Pos2) {
        if self.start.is_some() {
            self.current = Some(current);
        }
    }
    pub fn clear(&mut self) {
        self.start = None;
        self.current = None;
    }
    pub fn rect(&self) -> Option<Rect> {
        Some(Self::rect_from_points(self.start?, self.current?))
    }
    pub fn rect_from_points(a: Pos2, b: Pos2) -> Rect {
        Rect::from_min_max(
            Pos2::new(a.x.min(b.x), a.y.min(b.y)),
            Pos2::new(a.x.max(b.x), a.y.max(b.y)),
        )
    }
    pub fn intersecting_ids<K>(&self, items: impl IntoIterator<Item = (K, Rect)>) -> Vec<K>
    where
        K: Clone,
    {
        let Some(rect) = self.rect() else {
            return Vec::new();
        };
        items
            .into_iter()
            .filter_map(|(id, item_rect)| rect.intersects(item_rect).then_some(id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, vec2};

    #[test]
    fn marquee_rect_from_points_normalizes_drag_direction() {
        let rect = MarqueeSelection::rect_from_points(pos2(4.0, 3.0), pos2(1.0, -2.0));

        assert_eq!(rect.min, pos2(1.0, -2.0));
        assert_eq!(rect.max, pos2(4.0, 3.0));
    }

    #[test]
    fn inactive_marquee_returns_no_intersections() {
        let marquee = MarqueeSelection::default();
        let hits =
            marquee.intersecting_ids([(1, Rect::from_min_size(pos2(0.0, 0.0), vec2(1.0, 1.0)))]);

        assert!(hits.is_empty());
    }

    #[test]
    fn marquee_intersecting_ids_finds_overlapping_rects() {
        let mut marquee = MarqueeSelection::default();
        marquee.begin(pos2(0.0, 0.0));
        marquee.update(pos2(3.0, 3.0));

        let hits = marquee.intersecting_ids([
            (
                "inside",
                Rect::from_min_size(pos2(1.0, 1.0), vec2(1.0, 1.0)),
            ),
            (
                "outside",
                Rect::from_min_size(pos2(5.0, 5.0), vec2(1.0, 1.0)),
            ),
        ]);

        assert_eq!(hits, vec!["inside"]);
    }
}
