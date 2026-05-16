use egui::{Color32, Pos2, Rect, Stroke, Vec2};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoteRect {
    pub label: &'static str,
    pub beat: f32,
    pub length: f32,
    pub row: usize,
    pub velocity: f32,
    pub muted: bool,
    pub ghost: bool,
}

impl NoteRect {
    pub fn new(label: &'static str, beat: f32, length: f32, row: usize) -> Self {
        Self {
            label,
            beat,
            length,
            row,
            velocity: 1.0,
            muted: false,
            ghost: false,
        }
    }
    pub fn rect(&self, grid_rect: Rect, grid: &super::canvas::GridCanvas) -> Rect {
        let x = grid_rect.min.x + self.beat * grid.beat_width;
        let w = self.length * grid.beat_width;
        let h = grid.row_height;
        let y = grid_rect.min.y + self.row as f32 * h;
        Rect::from_min_size(Pos2::new(x, y), Vec2::new(w, h))
    }
    pub fn hit_test(&self, grid_rect: Rect, pos: Pos2) -> bool {
        self.rect(grid_rect, &super::canvas::GridCanvas::default())
            .contains(pos)
    }
    pub fn paint(&self, painter: &egui::Painter, grid: &super::canvas::GridCanvas, origin: Pos2) {
        let grid_rect = Rect::from_min_size(origin, grid.size);
        let rect = self.rect(grid_rect, grid).shrink(1.0);
        let mut color = Color32::from_rgb((80.0 + self.velocity * 120.0) as u8, 150, 80);
        if self.muted {
            color = color.gamma_multiply(0.45);
        }
        if self.ghost {
            color = color.gamma_multiply(0.6);
        }
        painter.rect_filled(rect, 2.0, color);
        painter.rect_stroke(
            rect,
            2.0,
            Stroke::new(1.0, Color32::from_black_alpha(80)),
            egui::StrokeKind::Inside,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_test_uses_default_grid_geometry() {
        let note = NoteRect {
            label: "note",
            beat: 1.0,
            length: 1.0,
            row: 0,
            velocity: 0.8,
            muted: false,
            ghost: false,
        };
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 100.0));

        assert!(note.hit_test(rect, Pos2::new(40.0, 10.0)));
    }
}
