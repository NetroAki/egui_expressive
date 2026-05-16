//! Canvas item hit-testing, movement, and resize helpers.

use egui::{Id, Pos2, Rect, Vec2};

use crate::editor::SnapGrid;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResizeEdges {
    pub left: bool,
    pub right: bool,
    pub top: bool,
    pub bottom: bool,
}

impl ResizeEdges {
    pub const NONE: Self = Self {
        left: false,
        right: false,
        top: false,
        bottom: false,
    };
    pub const ALL: Self = Self {
        left: true,
        right: true,
        top: true,
        bottom: true,
    };
    pub const HORIZONTAL: Self = Self {
        left: true,
        right: true,
        top: false,
        bottom: false,
    };
    pub const VERTICAL: Self = Self {
        left: false,
        right: false,
        top: true,
        bottom: true,
    };

    pub fn is_empty(self) -> bool {
        !self.left && !self.right && !self.top && !self.bottom
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, vec2};

    fn item() -> CanvasItem<u64> {
        CanvasItem::rect(1, Rect::from_min_size(pos2(0.0, 0.0), vec2(2.0, 1.0)))
            .resize_edges(ResizeEdges::ALL)
            .min_size(vec2(1.0, 1.0))
    }

    #[test]
    fn hit_test_distinguishes_edge_body_and_miss() {
        let item = item();
        assert_eq!(
            item.hit_test(pos2(2.0, 0.5), 0.2),
            CanvasItemHit::Edge(ResizeEdges {
                right: true,
                ..ResizeEdges::NONE
            })
        );
        assert_eq!(item.hit_test(pos2(1.0, 0.5), 0.2), CanvasItemHit::Body);
        assert_eq!(item.hit_test(pos2(4.0, 4.0), 0.2), CanvasItemHit::None);
    }

    #[test]
    fn resize_clamps_left_edge_to_min_size() {
        let resized = item().resized_rect(
            ResizeEdges {
                left: true,
                ..ResizeEdges::NONE
            },
            vec2(1.5, 0.0),
            &SnapGrid::disabled(),
        );

        assert_eq!(resized.min.x, 1.0);
        assert_eq!(resized.width(), 1.0);
    }

    #[test]
    fn move_respects_axis_locks() {
        let locked = item().lock_x(true).lock_y(true);
        assert_eq!(
            locked.moved_rect(vec2(3.0, 4.0), &SnapGrid::disabled()),
            locked.rect
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasItemHit {
    None,
    Body,
    Edge(ResizeEdges),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanvasItem<K = Id> {
    pub id: K,
    pub rect: Rect,
    pub min_size: Vec2,
    pub resize_edges: ResizeEdges,
    pub lock_x: bool,
    pub lock_y: bool,
    pub selectable: bool,
}

impl<K> CanvasItem<K> {
    pub fn rect(id: K, rect: Rect) -> Self {
        Self {
            id,
            rect,
            min_size: Vec2::splat(1.0),
            resize_edges: ResizeEdges::NONE,
            lock_x: false,
            lock_y: false,
            selectable: true,
        }
    }

    pub fn min_size(mut self, min_size: Vec2) -> Self {
        self.min_size = min_size;
        self
    }
    pub fn resize_edges(mut self, edges: ResizeEdges) -> Self {
        self.resize_edges = edges;
        self
    }
    pub fn resizable_x(mut self, enabled: bool) -> Self {
        self.resize_edges.left = enabled;
        self.resize_edges.right = enabled;
        self
    }
    pub fn resizable_y(mut self, enabled: bool) -> Self {
        self.resize_edges.top = enabled;
        self.resize_edges.bottom = enabled;
        self
    }
    pub fn lock_x(mut self, lock: bool) -> Self {
        self.lock_x = lock;
        self
    }
    pub fn lock_y(mut self, lock: bool) -> Self {
        self.lock_y = lock;
        self
    }
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    pub fn hit_test(&self, pos: Pos2, tolerance: f32) -> CanvasItemHit {
        if !self.rect.expand(tolerance).contains(pos) {
            return CanvasItemHit::None;
        }
        let mut edges = ResizeEdges::NONE;
        if self.resize_edges.left && (pos.x - self.rect.left()).abs() <= tolerance {
            edges.left = true;
        }
        if self.resize_edges.right && (pos.x - self.rect.right()).abs() <= tolerance {
            edges.right = true;
        }
        if self.resize_edges.top && (pos.y - self.rect.top()).abs() <= tolerance {
            edges.top = true;
        }
        if self.resize_edges.bottom && (pos.y - self.rect.bottom()).abs() <= tolerance {
            edges.bottom = true;
        }
        if !edges.is_empty() {
            CanvasItemHit::Edge(edges)
        } else if self.rect.contains(pos) {
            CanvasItemHit::Body
        } else {
            CanvasItemHit::None
        }
    }

    pub fn moved_rect(&self, delta: Vec2, snap: &SnapGrid) -> Rect {
        let delta = Vec2::new(
            if self.lock_x { 0.0 } else { delta.x },
            if self.lock_y { 0.0 } else { delta.y },
        );
        snap.snap_rect_min(self.rect.translate(delta))
    }

    pub fn resized_rect(&self, edges: ResizeEdges, delta: Vec2, snap: &SnapGrid) -> Rect {
        let mut min = self.rect.min;
        let mut max = self.rect.max;
        if !self.lock_x {
            if edges.left {
                min.x += delta.x;
            }
            if edges.right {
                max.x += delta.x;
            }
        }
        if !self.lock_y {
            if edges.top {
                min.y += delta.y;
            }
            if edges.bottom {
                max.y += delta.y;
            }
        }
        if edges.left || edges.top {
            min = snap.snap_pos(min);
        }
        if edges.right || edges.bottom {
            max = snap.snap_pos(max);
        }
        if max.x - min.x < self.min_size.x {
            if edges.left && !edges.right {
                min.x = max.x - self.min_size.x;
            } else {
                max.x = min.x + self.min_size.x;
            }
        }
        if max.y - min.y < self.min_size.y {
            if edges.top && !edges.bottom {
                min.y = max.y - self.min_size.y;
            } else {
                max.y = min.y + self.min_size.y;
            }
        }
        Rect::from_min_max(min, max)
    }
}
