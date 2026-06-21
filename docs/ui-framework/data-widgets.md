# Data Widgets

Data widgets provide read-only, data-heavy surfaces for dashboards, admin views,
and inspectors. Editing is modeled through separate form/edit descriptors so the
rendered table and property widgets stay predictable and display-focused.

## Modules

- `src/widgets/data/mod.rs` — docs and re-exports.
- `src/widgets/data/state.rs` — sort/filter/selection/view-status state.
- `src/widgets/data/model.rs` — data-grid rows, columns, cells, and row providers.
- `src/widgets/data/data_table.rs` — `DataTable` read-only table widget.
- `src/widgets/data/tree_table.rs` — tree-table model, flattening, and read-only widget.
- `src/widgets/data/property_grid.rs` — read-only property/inspector grid.
- `src/widgets/data/editing.rs` — additive data-cell/property edit specs that consume form inline-edit commits.
- `src/widgets/data/virtual_window.rs` — pure visible-range helper for app-owned windowed callers.

## Scope

- Rendered data-table/tree-table/property-grid widgets remain read-only display surfaces.
- Inline editing is modeled through additive descriptors/adapters, not built into `DataTable` rendering.
- Single-column sorting only.
- Per-column filters supported.
- Row/column selection with fallback behavior; column selection is single-select.
- Column visibility supported.
- Tree-table expansion supported.
- Built-in widget windowing is not claimed; the current widgets materialize
  filtered/sorted/flattened rows for the current frame. Very large datasets
  should use app-owned caching, pagination, or the `bounded_visible_range` helper.

## Recipes

### Minimal DataTable

```rust
let model = DataGridModel::new(
    vec![DataColumn::new("name", "Name"), DataColumn::new("status", "Status")],
    vec![DataRow::new("alpha", vec![DataCell::new("Alpha"), DataCell::new("Ready")])],
);
let mut state = DataGridState::default();
state.filter.query = "ready".into();
ui.add(DataTable::new(&model, &mut state));
```

Use header-title clicks for sort; use the separate column affordance for selection.
Header clicks cycle ascending/descending for one column and do not clear sort;
clear sorting explicitly with `state.sort = None`.

```rust
state.sort = None; // default: header clicks drive sort
state.sort = Some(DataSortState::new(Some("name".into()), DataSortDirection::Asc));
```

### Minimal TreeTable

```rust
let model = TreeTableModel::new(
    vec![DataColumn::new("label", "Label")],
    vec![TreeTableNode::new("root", "Root")],
);
let mut state = TreeTableState::default();
ui.add(TreeTable::new(&model, &mut state));
```

`TreeTableModel` uses the static `DataColumn.visible` flag only; it does not
carry a separate runtime hidden-columns state. `columns[0]` is the fixed label
column and `TreeTable` always renders it. For later columns, `cells[i]` maps to
`columns[i + 1]` regardless of hidden-column gaps.

```rust
let rows = egui_expressive::flatten_tree_table_rows(&model.nodes, &state);
```

### Minimal PropertyGrid

```rust
let model = PropertyGridModel::new(vec![
    PropertyGridEntry::new("Name", "Dashboard", "General").group("Identity"),
    PropertyGridEntry::new("Rows", "3", "General").group("Metrics"),
]);
ui.add(PropertyGrid::new(&model));
```

### Inline edit adapter

```rust
let mut cell_edit = DataCellEditSpec::new(
    "row-1",
    "gain",
    FormFieldKind::Text,
    FormFieldValue::Text("0 dB".into()),
);
let commit = InlineEditCommit {
    target: InlineEditTarget::data_cell("row-1", "gain"),
    value: FormFieldValue::Text("-6 dB".into()),
};
cell_edit.apply_commit(&commit);
```

`PropertyEditSpec` provides the same commit-target pattern for property-grid values.

### View-status + filter setup

```rust
state.filter.query = search.clone();
state.view_status = if is_loading {
    DataViewStatus::Loading
} else if is_error {
    DataViewStatus::Error("Network timeout".into())
} else if model.rows().is_empty() {
    DataViewStatus::Empty
} else {
    DataViewStatus::Ready
};
```

## Example

- `examples/data_explorer_dashboard.rs` combines app-shell chrome with the data widgets.

## Notes

- `DataGridModel::filtered_sorted_row_indices` intentionally materializes a filtered/sorted index vector per call.
- Sort comparisons and filter `contains` checks are case-folded with Rust `str::to_lowercase`; non-ASCII casing follows standard library behavior.
- `TreeTable` exposes `row_height` / `header_height` builders for parity with `DataTable`.
- `TreeTable` uses a 16.0 indent step and `▾` / `▸` / `•` glyph defaults.
- `PropertyGrid` scrolls vertically by default for long inspectors.
- `flatten_tree_table_rows` is a stable public helper that matches `TreeTableModel::flattened_rows`.
- Public data structs intentionally keep serde-friendly public fields; field semantics are documented at the type/method/recipe level rather than with one-line field rustdoc on every model field.
- Inline editing is represented by descriptors/adapters in `src/forms/editing.rs` and `src/widgets/data/editing.rs`; rendered widgets remain read-only by design.
- The example demonstrates loading/empty/error state switching without introducing edit flows.

## Boundaries

- Built-in spreadsheet-style cell/property editing is out of scope for the rendered widgets.
- Multi-column sort, column reordering, column pinning, and resize handles are not built in.
- Very large data sets should cache filtered/sorted rows, page data, or supply an app-owned windowed surface rather than relying on these widgets to window rows internally.
