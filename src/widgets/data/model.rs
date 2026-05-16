use serde::{Deserialize, Serialize};

use super::state::{DataGridState, DataSortDirection};

/// A single read-only cell value in a data grid.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataCell {
    pub value: String,
}

impl DataCell {
    /// Creates a cell from a string-like value.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

/// A data-grid column definition.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataColumn {
    pub id: String,
    pub title: String,
    pub width: f32,
    pub visible: bool,
    pub sortable: bool,
    pub filterable: bool,
    pub selectable: bool,
}

impl DataColumn {
    /// Creates a visible, sortable, filterable, selectable column.
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            width: 140.0,
            visible: true,
            sortable: true,
            filterable: true,
            selectable: true,
        }
    }

    /// Sets the column width in logical pixels; clamped to a minimum of 24.0.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(24.0);
        self
    }
    /// Marks the column visible or hidden by default.
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
    /// Marks the column sortable or read-only for sort gestures.
    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }
    /// Marks the column filterable for global and column filters.
    pub fn filterable(mut self, filterable: bool) -> Self {
        self.filterable = filterable;
        self
    }
    /// Marks the column selectable in the header affordance.
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }
}

/// A read-only row in a data grid.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataRow {
    pub id: String,
    pub cells: Vec<DataCell>,
}

impl DataRow {
    /// Creates a row with a stable row id and ordered cells.
    pub fn new(id: impl Into<String>, cells: impl Into<Vec<DataCell>>) -> Self {
        Self {
            id: id.into(),
            cells: cells.into(),
        }
    }

    /// Returns the cell at `index`, if present.
    pub fn cell(&self, index: usize) -> Option<&DataCell> {
        self.cells.get(index)
    }
}

/// Provides row and column slices for read-only data-grid views.
pub trait DataRowProvider {
    fn columns(&self) -> &[DataColumn];
    fn rows(&self) -> &[DataRow];
}

/// In-memory model for `DataTable` and related data-grid views.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DataGridModel {
    columns: Vec<DataColumn>,
    rows: Vec<DataRow>,
}

impl DataGridModel {
    /// Creates a model from ordered columns and rows.
    pub fn new(columns: impl Into<Vec<DataColumn>>, rows: impl Into<Vec<DataRow>>) -> Self {
        Self {
            columns: columns.into(),
            rows: rows.into(),
        }
    }

    /// Returns the model columns.
    pub fn columns(&self) -> &[DataColumn] {
        &self.columns
    }

    /// Returns the model rows.
    pub fn rows(&self) -> &[DataRow] {
        &self.rows
    }

    /// Returns the index for `column_id`, if present.
    pub fn column_index(&self, column_id: &str) -> Option<usize> {
        self.columns
            .iter()
            .position(|column| column.id == column_id)
    }

    /// Returns the index for `row_id`, if present.
    pub fn row_index(&self, row_id: &str) -> Option<usize> {
        self.rows.iter().position(|row| row.id == row_id)
    }

    /// Returns visible column indices in model order.
    pub fn visible_column_indices(&self, state: &DataGridState) -> Vec<usize> {
        self.columns
            .iter()
            .enumerate()
            .filter(|(_, column)| state.is_column_visible(column))
            .map(|(index, _)| index)
            .collect()
    }

    /// Returns row indices after Stage 3 filtering and single-column sort.
    ///
    /// Sort comparisons are case-folded with Rust `str::to_lowercase`.
    pub fn filtered_sorted_row_indices(&self, state: &DataGridState) -> Vec<usize> {
        let mut indices: Vec<usize> = self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, row)| self.row_matches_state(row, state))
            .map(|(index, _)| index)
            .collect();

        if let Some(sort) = &state.sort {
            if let Some(column_id) = sort.column_id.as_deref() {
                if let Some(column_index) = self.column_index(column_id) {
                    if self.columns[column_index].sortable
                        && state.is_column_visible(&self.columns[column_index])
                    {
                        indices.sort_by(|left, right| {
                            let lhs = self.rows[*left]
                                .cell(column_index)
                                .map(|cell| cell.value.to_lowercase())
                                .unwrap_or_default();
                            let rhs = self.rows[*right]
                                .cell(column_index)
                                .map(|cell| cell.value.to_lowercase())
                                .unwrap_or_default();
                            let ordering = lhs.cmp(&rhs);
                            match sort.direction {
                                DataSortDirection::Asc => ordering,
                                DataSortDirection::Desc => ordering.reverse(),
                            }
                        });
                    }
                }
            }
        }

        indices
    }

    /// Returns true when `row` matches the global and per-column filters.
    ///
    /// Filter `contains` checks are case-folded with Rust `str::to_lowercase`.
    pub fn row_matches_state(&self, row: &DataRow, state: &DataGridState) -> bool {
        let query = state.filter.query.trim().to_lowercase();
        if !query.is_empty() {
            let mut matched = false;
            for (index, column) in self.columns.iter().enumerate() {
                if !state.is_column_visible(column) || !column.filterable {
                    continue;
                }
                if row
                    .cell(index)
                    .map(|cell| cell.value.to_lowercase().contains(&query))
                    .unwrap_or(false)
                {
                    matched = true;
                    break;
                }
            }
            if !matched {
                return false;
            }
        }

        for filter in &state.filter.column_filters {
            if filter.query.trim().is_empty() {
                continue;
            }
            let Some(index) = self.column_index(&filter.column_id) else {
                continue;
            };
            let column = &self.columns[index];
            if !column.filterable || !state.is_column_visible(column) {
                continue;
            }
            let value = row
                .cell(index)
                .map(|cell| cell.value.to_lowercase())
                .unwrap_or_default();
            if !value.contains(&filter.query.trim().to_lowercase()) {
                return false;
            }
        }

        true
    }

    /// Returns the selected row or the first visible row fallback.
    pub fn selected_row<'a>(&'a self, state: &DataGridState) -> Option<&'a DataRow> {
        state.selected_row_or_first(&self.rows)
    }

    /// Returns the selected column or the first visible column fallback.
    pub fn selected_column<'a>(&'a self, state: &DataGridState) -> Option<&'a DataColumn> {
        state.selected_column_or_first(&self.columns)
    }

    /// Returns visible rows in filtered/sorted order.
    pub fn visible_rows<'a>(&'a self, state: &DataGridState) -> Vec<&'a DataRow> {
        self.filtered_sorted_row_indices(state)
            .into_iter()
            .map(|index| &self.rows[index])
            .collect()
    }
}

impl DataRowProvider for DataGridModel {
    fn columns(&self) -> &[DataColumn] {
        &self.columns
    }
    fn rows(&self) -> &[DataRow] {
        &self.rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::data::state::{
        DataColumnFilter, DataGridState, DataSortDirection, DataSortState,
    };

    fn sample_model() -> DataGridModel {
        DataGridModel::new(
            vec![
                DataColumn::new("name", "Name"),
                DataColumn::new("status", "Status"),
            ],
            vec![
                DataRow::new(
                    "alpha",
                    vec![DataCell::new("Alpha"), DataCell::new("Ready")],
                ),
                DataRow::new("beta", vec![DataCell::new("Beta"), DataCell::new("Paused")]),
                DataRow::new(
                    "gamma",
                    vec![DataCell::new("Gamma"), DataCell::new("Ready")],
                ),
            ],
        )
    }

    #[test]
    fn data_grid_model_filters_rows_by_global_and_column_queries() {
        let model = sample_model();
        let mut state = DataGridState::default();
        state.filter.query = "alp".into();
        assert_eq!(model.filtered_sorted_row_indices(&state), vec![0]);

        state.filter.query.clear();
        state
            .filter
            .column_filters
            .push(DataColumnFilter::new("status", "ready"));
        assert_eq!(model.filtered_sorted_row_indices(&state), vec![0, 2]);
    }

    #[test]
    fn data_grid_model_sorts_single_column() {
        let model = sample_model();
        let state = DataGridState {
            sort: Some(DataSortState::new(
                Some("name".into()),
                DataSortDirection::Desc,
            )),
            ..Default::default()
        };
        let rows = model.filtered_sorted_row_indices(&state);
        assert_eq!(rows, vec![2, 1, 0]);
    }

    #[test]
    fn data_grid_model_uses_column_visibility_and_selection_fallbacks() {
        let model = sample_model();
        let mut state = DataGridState::default();
        state.hide_column("status");
        state.selection.column_id = Some("missing".into());
        state.recover_selection(model.rows(), model.columns());

        assert_eq!(model.visible_column_indices(&state), vec![0]);
        assert_eq!(state.selection.column_id.as_deref(), Some("name"));
    }

    #[test]
    fn data_grid_model_ignores_hidden_column_sort_and_filter() {
        let model = sample_model();
        let mut state = DataGridState::default();
        state.hide_column("status");
        state.sort = Some(DataSortState::new(
            Some("status".into()),
            DataSortDirection::Asc,
        ));
        state
            .filter
            .column_filters
            .push(DataColumnFilter::new("status", "paused"));

        assert_eq!(model.filtered_sorted_row_indices(&state), vec![0, 1, 2]);
    }

    #[test]
    fn data_row_provider_returns_rows_and_columns() {
        let model = sample_model();
        assert_eq!(DataRowProvider::columns(&model).len(), 2);
        assert_eq!(DataRowProvider::rows(&model).len(), 3);
    }
}
