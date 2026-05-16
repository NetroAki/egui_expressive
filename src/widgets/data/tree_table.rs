use serde::{Deserialize, Serialize};

use super::{DataCell, DataColumn};

/// Nested node used to build a read-only tree table.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TreeTableNode {
    pub id: String,
    pub label: String,
    pub cells: Vec<DataCell>,
    pub children: Vec<TreeTableNode>,
    pub expanded: bool,
}

impl TreeTableNode {
    /// Creates an expanded tree node with no cells or children.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            cells: Vec::new(),
            children: Vec::new(),
            expanded: true,
        }
    }

    /// Sets leaf cells for the node.
    pub fn with_cells(mut self, cells: impl Into<Vec<DataCell>>) -> Self {
        self.cells = cells.into();
        self
    }
    /// Sets the node children.
    pub fn with_children(mut self, children: impl Into<Vec<TreeTableNode>>) -> Self {
        self.children = children.into();
        self
    }
    /// Marks the node as collapsed by default.
    pub fn collapsed(mut self) -> Self {
        self.expanded = false;
        self
    }
}

/// Flattened tree-table row produced from `TreeTableNode` data.
#[derive(Clone, Debug, PartialEq)]
pub struct TreeTableRow {
    pub id: String,
    pub label: String,
    pub depth: usize,
    pub has_children: bool,
    pub expanded: bool,
    pub cells: Vec<DataCell>,
}

/// Expansion and selection state for `TreeTable`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeTableState {
    pub expanded_rows: Vec<String>,
    pub collapsed_rows: Vec<String>,
    pub selected_row: Option<String>,
}

impl TreeTableState {
    /// Returns the selected row or the first visible row.
    pub fn selected_row_or_first<'a>(&self, rows: &'a [TreeTableRow]) -> Option<&'a TreeTableRow> {
        self.selected_row
            .as_deref()
            .and_then(|wanted| rows.iter().find(|row| row.id == wanted))
            .or_else(|| rows.first())
    }

    /// Resolves the effective expansion state for `row_id`.
    pub fn is_expanded(&self, row_id: &str, default_expanded: bool) -> bool {
        if self.collapsed_rows.iter().any(|id| id == row_id) {
            return false;
        }
        default_expanded || self.expanded_rows.iter().any(|id| id == row_id)
    }

    /// Toggles the stored expansion override for `row_id`.
    pub fn toggle_expanded(&mut self, row_id: &str, currently_expanded: bool) {
        if currently_expanded {
            self.expanded_rows.retain(|id| id != row_id);
            if !self.collapsed_rows.iter().any(|id| id == row_id) {
                self.collapsed_rows.push(row_id.to_owned());
            }
        } else {
            self.collapsed_rows.retain(|id| id != row_id);
            if !self.expanded_rows.iter().any(|id| id == row_id) {
                self.expanded_rows.push(row_id.to_owned());
            }
        }
    }
}

/// In-memory model for `TreeTable`.
///
/// `columns[0]` is the fixed label column and is always rendered by `TreeTable`;
/// later visible columns map to `TreeTableRow.cells` by original column order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TreeTableModel {
    pub columns: Vec<DataColumn>,
    pub nodes: Vec<TreeTableNode>,
}

impl TreeTableModel {
    /// Creates a tree-table model from columns and root nodes.
    pub fn new(columns: impl Into<Vec<DataColumn>>, nodes: impl Into<Vec<TreeTableNode>>) -> Self {
        Self {
            columns: columns.into(),
            nodes: nodes.into(),
        }
    }

    /// Returns a flattened, read-only row view for the current expansion state.
    pub fn flattened_rows(&self, state: &TreeTableState) -> Vec<TreeTableRow> {
        flatten_tree_table_rows(&self.nodes, state)
    }
}

/// Flattens tree nodes into rows for stable read-only rendering.
///
/// This helper is part of the public API because `TreeTableModel::flattened_rows`
/// delegates to it directly and downstream code may want the same flattening
/// semantics without owning the widget.
pub fn flatten_tree_table_rows(
    nodes: &[TreeTableNode],
    state: &TreeTableState,
) -> Vec<TreeTableRow> {
    fn flatten_node(
        node: &TreeTableNode,
        depth: usize,
        state: &TreeTableState,
        rows: &mut Vec<TreeTableRow>,
    ) {
        let expanded = state.is_expanded(&node.id, node.expanded);
        rows.push(TreeTableRow {
            id: node.id.clone(),
            label: node.label.clone(),
            depth,
            has_children: !node.children.is_empty(),
            expanded,
            cells: node.cells.clone(),
        });
        if expanded {
            for child in &node.children {
                flatten_node(child, depth + 1, state, rows);
            }
        }
    }

    let mut rows = Vec::new();
    for node in nodes {
        flatten_node(node, 0, state, &mut rows);
    }
    rows
}

/// Read-only virtualized tree table.
///
/// Uses `show_rows` for virtualization. The first model column is the fixed
/// label column; indent step is `16.0` and expand/leaf glyph defaults are `▾`,
/// `▸`, and `•`.
pub struct TreeTable<'a> {
    model: &'a TreeTableModel,
    state: &'a mut TreeTableState,
    row_height: f32,
    header_height: f32,
}

impl<'a> TreeTable<'a> {
    /// Creates a tree-table widget for a model/state pair.
    pub fn new(model: &'a TreeTableModel, state: &'a mut TreeTableState) -> Self {
        Self {
            model,
            state,
            row_height: 24.0,
            header_height: 26.0,
        }
    }

    /// Sets the row height in logical pixels; clamped to a minimum of 16.0.
    pub fn row_height(mut self, row_height: f32) -> Self {
        self.row_height = row_height.max(16.0);
        self
    }

    /// Sets the header height in logical pixels; clamped to a minimum of 18.0.
    pub fn header_height(mut self, header_height: f32) -> Self {
        self.header_height = header_height.max(18.0);
        self
    }
}

impl<'a> egui::Widget for TreeTable<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let rows = self.model.flattened_rows(self.state);
        let columns = visible_tree_columns(&self.model.columns);
        let header_response = ui
            .horizontal(|ui| {
                for (_, column) in &columns {
                    ui.add_sized(
                        [column.width, self.header_height],
                        egui::Label::new(&column.title),
                    );
                }
            })
            .response;
        let rows_output =
            egui::ScrollArea::vertical().show_rows(ui, self.row_height, rows.len(), |ui, range| {
                for index in range {
                    let row = &rows[index];
                    ui.horizontal(|ui| {
                        for (column_index, column) in &columns {
                            if *column_index == 0 {
                                ui.allocate_ui_with_layout(
                                    egui::vec2(column.width, self.row_height),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        ui.add_space(row.depth as f32 * 16.0);
                                        if row.has_children {
                                            let symbol = if row.expanded { "▾" } else { "▸" };
                                            if ui.small_button(symbol).clicked() {
                                                self.state.toggle_expanded(&row.id, row.expanded);
                                            }
                                        } else {
                                            ui.label("•");
                                        }
                                        let selected = self.state.selected_row.as_deref()
                                            == Some(row.id.as_str());
                                        if ui.selectable_label(selected, &row.label).clicked() {
                                            self.state.selected_row = Some(row.id.clone());
                                        }
                                    },
                                );
                            } else {
                                let value = row
                                    .cells
                                    .get(*column_index - 1)
                                    .map(|cell| cell.value.as_str())
                                    .unwrap_or("");
                                ui.add_sized(
                                    [column.width, self.row_height],
                                    egui::Label::new(value),
                                );
                            }
                        }
                    });
                }
            });
        let rows_response = ui.interact(
            rows_output.inner_rect,
            ui.id().with("tree_table_rows"),
            egui::Sense::hover(),
        );
        header_response.union(rows_response)
    }
}

fn visible_tree_columns(columns: &[DataColumn]) -> Vec<(usize, &DataColumn)> {
    columns
        .iter()
        .enumerate()
        .filter(|(index, column)| *index == 0 || column.visible)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_table_flattens_expanded_rows() {
        let model = TreeTableModel::new(
            vec![DataColumn::new("name", "Name")],
            vec![TreeTableNode::new("root", "Root")
                .with_children(vec![TreeTableNode::new("child", "Child")])],
        );
        let rows = model.flattened_rows(&TreeTableState::default());
        assert_eq!(
            rows.iter().map(|row| row.id.as_str()).collect::<Vec<_>>(),
            vec!["root", "child"]
        );
    }

    #[test]
    fn tree_table_respects_collapsed_nodes_and_fallback_selection() {
        let model = TreeTableModel::new(
            vec![DataColumn::new("name", "Name")],
            vec![TreeTableNode::new("root", "Root")
                .collapsed()
                .with_children(vec![TreeTableNode::new("child", "Child")])],
        );
        let rows = model.flattened_rows(&TreeTableState::default());
        assert_eq!(rows.len(), 1);

        let state = TreeTableState {
            expanded_rows: vec![],
            collapsed_rows: vec![],
            selected_row: Some("missing".into()),
        };
        assert_eq!(
            state
                .selected_row_or_first(&rows)
                .map(|row| row.id.as_str()),
            Some("root")
        );
    }

    #[test]
    fn tree_table_state_can_collapse_default_expanded_nodes() {
        let model = TreeTableModel::new(
            vec![DataColumn::new("name", "Name")],
            vec![TreeTableNode::new("root", "Root")
                .with_children(vec![TreeTableNode::new("child", "Child")])],
        );
        let mut state = TreeTableState::default();
        state.toggle_expanded("root", true);

        let rows = model.flattened_rows(&state);
        assert_eq!(
            rows.iter().map(|row| row.id.as_str()).collect::<Vec<_>>(),
            vec!["root"]
        );
    }

    #[test]
    fn tree_table_large_tree_has_bounded_window() {
        let model = TreeTableModel::new(
            vec![DataColumn::new("label", "Label")],
            vec![TreeTableNode::new("root", "Root").with_children(
                (0..1_000)
                    .map(|index| {
                        TreeTableNode::new(format!("child-{index}"), format!("Child {index}"))
                    })
                    .collect::<Vec<_>>(),
            )],
        );
        let rows = model.flattened_rows(&TreeTableState::default());
        assert_eq!(rows.len(), 1_001);
        assert_eq!(
            crate::widgets::data::bounded_visible_range(rows.len(), 1_000.0, 1_240.0, 20.0, 2),
            48..64
        );
    }

    #[test]
    fn tree_table_row_and_header_heights_are_clamped() {
        let model = TreeTableModel::new(
            vec![DataColumn::new("name", "Name")],
            vec![TreeTableNode::new("root", "Root")],
        );
        let mut state = TreeTableState::default();
        let widget = TreeTable::new(&model, &mut state)
            .row_height(12.0)
            .header_height(14.0);
        assert_eq!(widget.row_height, 16.0);
        assert_eq!(widget.header_height, 18.0);
    }

    #[test]
    fn tree_table_columns_keep_label_and_cell_offsets() {
        let columns = vec![
            DataColumn::new("label", "Label").visible(false),
            DataColumn::new("kind", "Kind").visible(false),
            DataColumn::new("status", "Status"),
        ];
        let visible = visible_tree_columns(&columns);
        assert_eq!(
            visible.iter().map(|(index, _)| *index).collect::<Vec<_>>(),
            vec![0, 2]
        );

        let row = TreeTableNode::new("root", "Root")
            .with_cells(vec![DataCell::new("Folder"), DataCell::new("Ready")]);
        let flattened = flatten_tree_table_rows(&[row], &TreeTableState::default());
        assert_eq!(flattened[0].cells[visible[1].0 - 1].value, "Ready");
    }
}
