# Data Widgets

Stage 3 adds read-only, data-heavy widgets for dashboards, admin views, and inspectors. Stage 5 adds pure edit descriptors/adapters that can be layered over those read-only views without turning the widgets into spreadsheet editors.

## Modules

- `src/widgets/data/mod.rs` — docs + re-exports.
- `src/widgets/data/state.rs` — sort/filter/selection/view-status state.
- `src/widgets/data/model.rs` — data-grid rows, columns, cells, and row providers.
- `src/widgets/data/data_table.rs` — `DataTable` read-only virtualized table widget.
- `src/widgets/data/tree_table.rs` — tree-table model, flattening, and read-only widget.
- `src/widgets/data/property_grid.rs` — read-only property/inspector grid.
- `src/widgets/data/editing.rs` — additive data-cell/property edit specs that consume Forms v2 inline-edit commits.
- `src/widgets/data/virtual_window.rs` — pure visible-range helper.

## Scope

- Rendered data-table/tree-table/property-grid widgets remain read-only display surfaces.
- Inline editing is modeled through additive Forms v2 descriptors/adapters, not built into `DataTable` rendering.
- Single-column sorting only.
- Per-column filters supported.
- Row/column selection with fallback behavior; column selection is single-select.
- Column visibility supported.
- Tree-table expansion supported.

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
// or set an initial sort once:
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

## App shell

`examples/data_explorer_dashboard.rs` shows Stage 3 widgets inside the Stage 2 app-shell chrome: sidebar, breadcrumbs, tabs, split panes, and status bar.

## Non-goals

- Built-in spreadsheet-style cell/property editing inside `DataTable`/`PropertyGrid`.
- Multi-column sort.
- Column reordering.
- Column pinning.
- Resize handles.

Stage 9 release boundary: advanced data-grid column interactions are explicitly unsupported rather than partially implemented. Apps that need multi-column sort, reordering, pinning, or resize handles should layer app-owned state and widgets around the read-only model, or wait for a later dedicated data-grid hardening stage. This resolves DEBT-020 for release-readiness purposes without adding spreadsheet scope to the core widget.

## Example

- `examples/data_explorer_dashboard.rs` combines Stage 2 app-shell chrome with the new data widgets.

## Notes

- Large row sets use `egui::ScrollArea::show_rows` in the production widgets; `bounded_visible_range` is the stable public helper / deterministic proof used by custom callers and tests. See the 10k-row smoke test in `src/widgets/data/virtual_window.rs`.
- Sort comparisons and filter `contains` checks are case-folded with Rust `str::to_lowercase`; non-ASCII casing follows standard library behavior.
- `TreeTable` exposes `row_height` / `header_height` builders for parity with `DataTable`.
- `TreeTable` uses a 16.0 indent step and `▾` / `▸` / `•` glyph defaults.
- `PropertyGrid` scrolls vertically by default for long inspectors.
- `flatten_tree_table_rows` is a stable public helper that matches `TreeTableModel::flattened_rows`.
- The large-data proof is intentionally split between the public helper and `show_rows` delegation; widget-level row-count instrumentation remains out of Stage 3 scope.
- `DataGridModel::filtered_sorted_row_indices` intentionally materializes a filtered/sorted index vector per call. Stage 9 release smoke tests cover 4k-row deterministic behavior and document the support boundary: use viewport culling for rendering, avoid calling filtered/sorted materialization more than once per frame for very large tables, and add app-owned caching if the dataset/filter/sort state is stable across frames. This resolves DEBT-021 for the current release boundary without adding cache invalidation complexity to the serde-friendly model.
- Public data structs intentionally keep serde-friendly public fields; field semantics are documented at the type/method/recipe level rather than with one-line field rustdoc on every model field.
- Inline editing is now represented by Forms v2 descriptors/adapters in `src/forms/editing.rs` and `src/widgets/data/editing.rs`; rendered Stage 3 widgets remain read-only by design.
- The example demonstrates loading/empty/error state switching without introducing edit flows.

## Debt pointers

- DEBT-006 — Stage 3 data widgets: `docs/exec-plans/tech-debt-tracker.md`
- DEBT-014 — inline data/property editing adapters owned by Stage 5: `docs/exec-plans/tech-debt-tracker.md`
- DEBT-020 — advanced data-grid column interactions resolved as unsupported release scope: `docs/exec-plans/tech-debt-tracker.md`
- DEBT-021 — filtered/sorted index materialization support boundary resolved through release smoke and docs: `docs/exec-plans/tech-debt-tracker.md`
