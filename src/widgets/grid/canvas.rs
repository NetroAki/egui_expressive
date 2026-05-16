use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

/// Canvas for beat/row grid rendering.
pub struct GridCanvas {
    pub beats_per_bar: usize,
    pub beat_width: f32,
    pub row_height: f32,
    pub subdivisions: usize,
    pub rows: usize,
    pub size: Vec2,
}

impl Default for GridCanvas {
    fn default() -> Self {
        Self {
            beats_per_bar: 4,
            beat_width: 32.0,
            row_height: 18.0,
            subdivisions: 4,
            rows: 8,
            size: Vec2::new(640.0, 180.0),
        }
    }
}

impl GridCanvas {
    pub fn new(beats_per_bar: usize, beat_width: f32, row_height: f32) -> Self {
        Self {
            beats_per_bar,
            beat_width,
            row_height,
            subdivisions: 4,
            rows: 8,
            size: Vec2::new(640.0, 180.0),
        }
    }
    pub fn subdivisions(mut self, subdivisions: usize) -> Self {
        self.subdivisions = subdivisions.max(1);
        self
    }
    pub fn beat_to_x(&self, beat: f32, rect: Rect) -> f32 {
        rect.min.x + beat.max(0.0) * self.beat_width
    }
    pub fn row_to_y(&self, row: usize, rect: Rect) -> f32 {
        rect.min.y + row as f32 * self.row_height
    }
    pub fn snap_beat(&self, beat: f32) -> f32 {
        (beat * self.subdivisions as f32).round() / self.subdivisions as f32
    }
    pub fn paint_grid(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        start_bar: usize,
        bars: usize,
        beats_per_bar: usize,
        colors: [Color32; 2],
    ) {
        let major = colors[1];
        let minor = colors[0];
        let total_beats = bars * beats_per_bar;
        for i in 0..=total_beats * self.subdivisions {
            let x = rect.min.x + i as f32 * (self.beat_width / self.subdivisions as f32);
            let is_bar = i % (self.subdivisions * beats_per_bar) == 0;
            painter.line_segment(
                [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                Stroke::new(1.0, if is_bar { major } else { minor }),
            );
        }
        for r in 0..=self.rows {
            let y = self.row_to_y(r, rect);
            painter.line_segment(
                [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
                Stroke::new(1.0, minor),
            );
        }
        let _ = start_bar;
    }
}

impl egui::Widget for GridCanvas {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::hover());
        self.paint_grid(
            ui.painter(),
            rect,
            0,
            self.beats_per_bar,
            self.beats_per_bar,
            [Color32::from_gray(35), Color32::from_gray(55)],
        );
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helpers_snap_to_current_subdivision() {
        let grid = GridCanvas::default();

        assert_eq!(grid.snap_beat(0.62), 0.5);
    }
}
