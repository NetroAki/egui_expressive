use serde::{Deserialize, Serialize};

use super::model::{DataColumn, DataRow};

/// Sort direction for a single data-grid column.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataSortDirection {
    Asc,
    Desc,
}

/// Sort state for a single active column.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataSortState {
    pub column_id: Option<String>,
    pub direction: DataSortDirection,
}

impl DataSortState {
    /// Creates a sort state for the given column id and direction.
    pub fn new(column_id: impl Into<Option<String>>, direction: DataSortDirection) -> Self {
        Self {
            column_id: column_id.into(),
            direction,
        }
    }
}

/// Column-level filter text for a data grid.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataColumnFilter {
    pub column_id: String,
    pub query: String,
}

impl DataColumnFilter {
    /// Creates a column filter.
    pub fn new(column_id: impl Into<String>, query: impl Into<String>) -> Self {
        Self {
            column_id: column_id.into(),
            query: query.into(),
        }
    }
}

/// Global and per-column filter state for a data grid.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataFilterState {
    pub query: String,
    pub column_filters: Vec<DataColumnFilter>,
}

/// Single-row and single-column selection state.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataSelectionState {
    pub row_id: Option<String>,
    pub column_id: Option<String>,
}

/// Loading, empty, or error status for a data view.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataViewStatus {
    #[default]
    Ready,
    Empty,
    Loading,
    Error(String),
}

/// Full state for a read-only data grid.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataGridState {
    pub view_status: DataViewStatus,
    pub sort: Option<DataSortState>,
    pub filter: DataFilterState,
    pub selection: DataSelectionState,
    pub hidden_columns: Vec<String>,
}

impl Default for DataGridState {
    fn default() -> Self {
        Self {
            view_status: DataViewStatus::Ready,
            sort: None,
            filter: DataFilterState::default(),
            selection: DataSelectionState::default(),
            hidden_columns: Vec::new(),
        }
    }
}

impl DataGridState {
    /// Creates a loading state.
    pub fn loading() -> Self {
        Self {
            view_status: DataViewStatus::Loading,
            ..Self::default()
        }
    }

    /// Creates an empty state.
    pub fn empty() -> Self {
        Self {
            view_status: DataViewStatus::Empty,
            ..Self::default()
        }
    }

    /// Creates an error state.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            view_status: DataViewStatus::Error(message.into()),
            ..Self::default()
        }
    }

    /// Returns true when the grid is loading.
    pub fn is_loading(&self) -> bool {
        matches!(self.view_status, DataViewStatus::Loading)
    }

    /// Returns true when the grid is empty.
    pub fn is_empty(&self) -> bool {
        matches!(self.view_status, DataViewStatus::Empty)
    }

    /// Returns true when the grid is in an error state.
    pub fn is_error(&self) -> bool {
        matches!(self.view_status, DataViewStatus::Error(_))
    }

    /// Returns the current error message, if any.
    pub fn error_message(&self) -> Option<&str> {
        match &self.view_status {
            DataViewStatus::Error(message) => Some(message),
            _ => None,
        }
    }

    /// Returns true when `column` is visible after runtime hides.
    pub fn is_column_visible(&self, column: &DataColumn) -> bool {
        column.visible && !self.hidden_columns.iter().any(|id| id == &column.id)
    }

    /// Returns visible column ids in model order.
    pub fn visible_column_ids(&self, columns: &[DataColumn]) -> Vec<String> {
        columns
            .iter()
            .filter(|column| self.is_column_visible(column))
            .map(|column| column.id.clone())
            .collect()
    }

    /// Selects the given row id.
    pub fn select_row(&mut self, row_id: impl Into<String>) {
        self.selection.row_id = Some(row_id.into());
    }

    /// Marks a single column as selected.
    ///
    /// Stage 3 keeps column selection single-select; `column_id` is the source of truth.
    pub fn select_column(&mut self, column_id: impl Into<String>) {
        self.selection.column_id = Some(column_id.into());
    }

    /// Hides `column_id` at runtime if it is not already hidden.
    pub fn hide_column(&mut self, column_id: impl Into<String>) {
        let id = column_id.into();
        if !self.hidden_columns.iter().any(|existing| existing == &id) {
            self.hidden_columns.push(id);
        }
    }

    /// Shows `column_id` by removing it from the hidden list.
    pub fn show_column(&mut self, column_id: &str) {
        self.hidden_columns.retain(|existing| existing != column_id);
    }

    /// Returns the selected row or the first row fallback.
    pub fn selected_row_or_first<'a>(&self, rows: &'a [DataRow]) -> Option<&'a DataRow> {
        self.selection
            .row_id
            .as_deref()
            .and_then(|wanted| rows.iter().find(|row| row.id == wanted))
            .or_else(|| rows.first())
    }

    /// Returns the selected visible column or the first visible column fallback.
    pub fn selected_column_or_first<'a>(
        &self,
        columns: &'a [DataColumn],
    ) -> Option<&'a DataColumn> {
        self.selection
            .column_id
            .as_deref()
            .and_then(|wanted| {
                columns
                    .iter()
                    .find(|column| column.id == wanted && self.is_column_visible(column))
            })
            .or_else(|| columns.iter().find(|column| self.is_column_visible(column)))
    }

    /// Restores selection to existing visible rows/columns.
    ///
    /// If the stored selection is missing, the first available row and visible
    /// column are auto-selected so the view always has a stable fallback.
    pub fn recover_selection(&mut self, rows: &[DataRow], columns: &[DataColumn]) {
        self.selection.row_id = self
            .selection
            .row_id
            .as_deref()
            .and_then(|wanted| {
                rows.iter()
                    .find(|row| row.id == wanted)
                    .map(|row| row.id.clone())
            })
            .or_else(|| rows.first().map(|row| row.id.clone()));
        self.selection.column_id = self
            .selection
            .column_id
            .as_deref()
            .and_then(|wanted| {
                columns
                    .iter()
                    .find(|column| column.id == wanted && self.is_column_visible(column))
                    .map(|column| column.id.clone())
            })
            .or_else(|| {
                columns
                    .iter()
                    .find(|column| self.is_column_visible(column))
                    .map(|column| column.id.clone())
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::data::model::{DataCell, DataGridModel, DataRow};

    #[test]
    fn data_grid_state_recovers_row_and_column_fallbacks() {
        let columns = vec![DataColumn::new("name", "Name")];
        let rows = vec![DataRow::new("alpha", vec![DataCell::new("Alpha")])];
        let mut state = DataGridState::default();

        state.selection.row_id = Some("missing".into());
        state.selection.column_id = Some("missing".into());
        state.recover_selection(&rows, &columns);

        assert_eq!(state.selection.row_id.as_deref(), Some("alpha"));
        assert_eq!(state.selection.column_id.as_deref(), Some("name"));
    }

    #[test]
    fn data_grid_state_tracks_loading_empty_and_error() {
        assert!(DataGridState::loading().is_loading());
        assert!(DataGridState::empty().is_empty());
        assert_eq!(DataGridState::error("boom").error_message(), Some("boom"));
    }

    #[test]
    fn data_grid_state_filters_hidden_columns() {
        let columns = vec![
            DataColumn::new("name", "Name"),
            DataColumn::new("status", "Status"),
        ];
        let mut state = DataGridState::default();
        state.hide_column("status");
        assert_eq!(state.visible_column_ids(&columns), vec!["name".to_string()]);
    }

    #[test]
    fn data_grid_state_selects_existing_rows() {
        let model = DataGridModel::new(
            vec![DataColumn::new("name", "Name")],
            vec![DataRow::new("alpha", vec![DataCell::new("Alpha")])],
        );
        let mut state = DataGridState::default();
        state.select_row("alpha");
        assert_eq!(
            state
                .selected_row_or_first(model.rows())
                .map(|row| row.id.as_str()),
            Some("alpha")
        );
    }
}
