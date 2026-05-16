//! CSS-grid-inspired helpers for egui layouts.

/// Column/row span for a single grid item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridSpan {
    pub columns: usize,
    pub rows: usize,
}

impl GridSpan {
    pub const fn new(columns: usize, rows: usize) -> Self {
        Self { columns, rows }
    }
}

/// Declarative grid configuration matching common Tailwind grid utilities.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridLayout {
    pub columns: usize,
    pub rows: Option<usize>,
    pub gap_x: f32,
    pub gap_y: f32,
}

impl GridLayout {
    pub fn columns(columns: usize) -> Self {
        Self {
            columns: columns.max(1),
            rows: None,
            gap_x: 0.0,
            gap_y: 0.0,
        }
    }

    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = Some(rows.max(1));
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap_x = gap;
        self.gap_y = gap;
        self
    }

    pub fn gap_x(mut self, gap: f32) -> Self {
        self.gap_x = gap;
        self
    }

    pub fn gap_y(mut self, gap: f32) -> Self {
        self.gap_y = gap;
        self
    }

    pub fn egui_grid(self, id: impl std::hash::Hash) -> egui::Grid {
        egui::Grid::new(id)
            .num_columns(self.columns)
            .spacing(egui::vec2(self.gap_x, self.gap_y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_columns_are_never_zero() {
        assert_eq!(GridLayout::columns(0).columns, 1);
    }
}
