use egui::{Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

/// Simple interactive boolean step-grid.
pub struct StepGrid<'a> {
    cells: &'a mut Vec<Vec<bool>>,
    rows: usize,
    cols: usize,
    size: Vec2,
    cell_size: Vec2,
    active_col: Option<usize>,
    row_colors: Option<Vec<egui::Color32>>,
}

impl<'a> StepGrid<'a> {
    pub fn new(cells: &'a mut Vec<Vec<bool>>, rows: usize, cols: usize) -> Self {
        Self {
            cells,
            rows,
            cols,
            size: Vec2::new(320.0, 180.0),
            cell_size: Vec2::new(24.0, 24.0),
            active_col: None,
            row_colors: None,
        }
    }
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
    pub fn cell_size(mut self, size: Vec2) -> Self {
        self.cell_size = size;
        self
    }
    pub fn active_col(mut self, col: usize) -> Self {
        self.active_col = Some(col);
        self
    }
    pub fn row_colors(mut self, colors: Vec<egui::Color32>) -> Self {
        self.row_colors = Some(colors);
        self
    }
}

impl<'a> egui::Widget for StepGrid<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        if self.cells.len() != self.rows || self.cells.iter().any(|r| r.len() != self.cols) {
            if self.cells.len() < self.rows {
                self.cells.resize_with(self.rows, Vec::new);
            }
            for row in self.cells.iter_mut().take(self.rows) {
                if row.len() < self.cols {
                    row.resize(self.cols, false);
                } else {
                    row.truncate(self.cols);
                }
            }
            self.cells.truncate(self.rows);
        }
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::click_and_drag());
        let cell_w = self.cell_size.x.max(1.0);
        let cell_h = self.cell_size.y.max(1.0);
        let visuals = ui.visuals();
        for r in 0..self.rows {
            for c in 0..self.cols {
                let cell_rect = Rect::from_min_max(
                    Pos2::new(
                        rect.min.x + c as f32 * cell_w,
                        rect.min.y + r as f32 * cell_h,
                    ),
                    Pos2::new(
                        rect.min.x + (c + 1) as f32 * cell_w,
                        rect.min.y + (r + 1) as f32 * cell_h,
                    ),
                );
                let active = self.cells[r][c];
                let row_fill = self
                    .row_colors
                    .as_ref()
                    .and_then(|v| v.get(r).copied())
                    .unwrap_or(visuals.widgets.inactive.bg_fill);
                let fill = if active {
                    visuals.selection.bg_fill
                } else {
                    row_fill
                };
                let fill = if self.active_col == Some(c) {
                    fill.gamma_multiply(1.2)
                } else {
                    fill
                };
                ui.painter().rect_filled(cell_rect.shrink(1.0), 2.0, fill);
                ui.painter().rect_stroke(
                    cell_rect.shrink(1.0),
                    2.0,
                    Stroke::new(1.0, visuals.widgets.inactive.bg_stroke.color),
                    egui::StrokeKind::Inside,
                );
                if let Some(pos) = response
                    .interact_pointer_pos()
                    .filter(|_| response.dragged() || response.clicked())
                {
                    if cell_rect.contains(pos) {
                        self.cells[r][c] = !active;
                    }
                }
            }
        }
        response
    }
}
