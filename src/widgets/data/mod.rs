//! Read-only data-heavy widgets and models.
//!
//! `DataTable`, `TreeTable`, and `PropertyGrid` render materialized model views.
//! For very large datasets, use app-owned caching/windowing with helpers such as
//! [`bounded_visible_range`].

mod data_table;
mod editing;
mod model;
mod property_grid;
mod state;
mod tree_table;
mod virtual_window;

pub use data_table::DataTable;
pub use editing::{DataCellEditSpec, PropertyEditSpec};
pub use model::{DataCell, DataColumn, DataGridModel, DataRow, DataRowProvider};
pub use property_grid::{
    PropertyGrid, PropertyGridCategory, PropertyGridEntry, PropertyGridGroup, PropertyGridModel,
};
pub use state::{
    DataColumnFilter, DataFilterState, DataGridState, DataSelectionState, DataSortDirection,
    DataSortState, DataViewStatus,
};
pub use tree_table::{
    flatten_tree_table_rows, TreeTable, TreeTableModel, TreeTableNode, TreeTableRow, TreeTableState,
};
pub use virtual_window::bounded_visible_range;
