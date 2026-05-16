#[derive(Clone, Debug, Default, PartialEq)]
pub struct StepCell {
    pub on: bool,
    pub velocity: f32,
    pub accent: bool,
    pub ghost: bool,
    pub extended: bool,
}

/// 2D step-cell storage used by sequencers and clip grids.
pub struct StepCellGrid<'a> {
    cells: &'a mut Vec<Vec<StepCell>>,
    rows: usize,
    cols: usize,
    cell_size: egui::Vec2,
    active_col: Option<usize>,
    row_colors: Option<Vec<egui::Color32>>,
}

impl<'a> StepCellGrid<'a> {
    pub fn new(cells: &'a mut Vec<Vec<StepCell>>, rows: usize, cols: usize) -> Self {
        ensure_step_cells(cells, rows, cols);
        Self {
            cells,
            rows,
            cols,
            cell_size: egui::Vec2::new(24.0, 24.0),
            active_col: None,
            row_colors: None,
        }
    }
    pub fn cell_size(mut self, size: egui::Vec2) -> Self {
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
    pub fn rows(&self) -> usize {
        self.rows
    }
    pub fn cols(&self) -> usize {
        self.cols
    }
    pub fn get(&self, row: usize, col: usize) -> Option<&StepCell> {
        self.cells.get(row).and_then(|r| r.get(col))
    }
    pub fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut StepCell> {
        self.cells.get_mut(row).and_then(|r| r.get_mut(col))
    }
}

impl<'a> egui::Widget for StepCellGrid<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(
            egui::Vec2::new(
                self.cols as f32 * self.cell_size.x,
                self.rows as f32 * self.cell_size.y,
            ),
            egui::Sense::click_and_drag(),
        );
        let visuals = ui.visuals();
        for r in 0..self.rows {
            for c in 0..self.cols {
                let cell_rect = egui::Rect::from_min_size(
                    egui::Pos2::new(
                        rect.min.x + c as f32 * self.cell_size.x,
                        rect.min.y + r as f32 * self.cell_size.y,
                    ),
                    self.cell_size,
                );
                let cell = &mut self.cells[r][c];
                let fill = if cell.on {
                    visuals.selection.bg_fill
                } else {
                    self.row_colors
                        .as_ref()
                        .and_then(|v| v.get(r).copied())
                        .unwrap_or(visuals.widgets.inactive.bg_fill)
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
                    egui::Stroke::new(1.0, visuals.widgets.inactive.bg_stroke.color),
                    egui::StrokeKind::Inside,
                );
                if let Some(pos) = response
                    .interact_pointer_pos()
                    .filter(|_| response.dragged() || response.clicked())
                {
                    if cell_rect.contains(pos) {
                        cell.on = !cell.on;
                    }
                }
            }
        }
        response
    }
}

pub(crate) fn ensure_step_cells(cells: &mut Vec<Vec<StepCell>>, rows: usize, cols: usize) {
    if cells.len() < rows {
        cells.resize_with(rows, Vec::new);
    }
    for row in cells.iter_mut().take(rows) {
        if row.len() < cols {
            row.resize_with(cols, StepCell::default);
        } else {
            row.truncate(cols);
        }
    }
    cells.truncate(rows);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_step_cells_expands_rows_and_columns() {
        let mut cells = vec![vec![StepCell::default(); 2]];

        ensure_step_cells(&mut cells, 2, 3);

        assert_eq!(cells.len(), 2);
        assert_eq!(cells[0].len(), 3);
    }
}
