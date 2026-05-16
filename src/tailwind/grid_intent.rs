//! Grid utility methods for `Tw`.

use crate::layout::GridLayout;
use crate::tailwind::{types::Display, Tw};

impl Tw {
    pub fn grid_cols(mut self, columns: usize) -> Self {
        self.display = Display::Grid;
        let mut grid = self.grid.unwrap_or_else(|| GridLayout::columns(columns));
        grid.columns = columns.max(1);
        self.grid = Some(grid);
        self
    }
    pub fn grid_rows(mut self, rows: usize) -> Self {
        self.grid = Some(
            self.grid
                .unwrap_or_else(|| GridLayout::columns(1))
                .rows(rows),
        );
        self
    }
    pub fn col_span(mut self, span: usize) -> Self {
        self.col_span = Some(span.max(1));
        self
    }
    pub fn row_span(mut self, span: usize) -> Self {
        self.row_span = Some(span.max(1));
        self
    }
}
