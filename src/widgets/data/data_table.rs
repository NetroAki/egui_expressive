use egui::{Color32, RichText, ScrollArea, Ui, Widget};

use super::{DataGridModel, DataGridState, DataViewStatus};

/// Read-only, virtualized data grid with sortable and selectable rows.
///
/// Header clicks cycle a sortable column between ascending and descending;
/// callers clear sorting explicitly by setting `DataGridState::sort` to `None`.
pub struct DataTable<'a> {
    model: &'a DataGridModel,
    state: &'a mut DataGridState,
    row_height: f32,
    header_height: f32,
}

impl<'a> DataTable<'a> {
    /// Creates a table widget for a model/state pair.
    pub fn new(model: &'a DataGridModel, state: &'a mut DataGridState) -> Self {
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

    fn status_text(&self) -> Option<String> {
        match &self.state.view_status {
            DataViewStatus::Ready => None,
            DataViewStatus::Empty => Some("No records".to_owned()),
            DataViewStatus::Loading => Some("Loading…".to_owned()),
            DataViewStatus::Error(message) => Some(format!("Error: {message}")),
        }
    }
}

impl<'a> Widget for DataTable<'a> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let columns = self.model.columns();
        let rows = self.model.rows();
        self.state.recover_selection(rows, columns);

        let visible_columns: Vec<_> = columns
            .iter()
            .enumerate()
            .filter(|(_, column)| self.state.is_column_visible(column))
            .collect();
        let visible_rows = self.model.filtered_sorted_row_indices(self.state);

        if let Some(text) = self.status_text() {
            return match &self.state.view_status {
                DataViewStatus::Error(_) => {
                    ui.label(RichText::new(text).strong().color(Color32::RED))
                }
                DataViewStatus::Loading => ui.label(RichText::new(text).italics()),
                _ => ui.label(text),
            };
        }

        if visible_columns.is_empty() {
            return ui.label("No visible columns");
        }

        let header_response = ui
            .allocate_ui_with_layout(
                egui::vec2(ui.available_width(), self.header_height),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    for (_, column) in &visible_columns {
                        let selected =
                            self.state.selection.column_id.as_deref() == Some(&column.id);
                        if column.selectable {
                            let select_label = if selected { "◉" } else { "○" };
                            if ui
                                .add_sized(
                                    [18.0, self.header_height],
                                    egui::Button::new(select_label).frame(false),
                                )
                                .clicked()
                            {
                                self.state.select_column(column.id.clone());
                            }
                        } else {
                            ui.add_sized([18.0, self.header_height], egui::Label::new(""));
                        }

                        let title = if selected {
                            RichText::new(&column.title).strong()
                        } else {
                            RichText::new(&column.title)
                        };
                        if ui
                            .add_sized(
                                [column.width - 18.0, self.header_height],
                                egui::Button::new(title).frame(false),
                            )
                            .clicked()
                            && column.sortable
                        {
                            let next = match self
                                .state
                                .sort
                                .as_ref()
                                .and_then(|sort| sort.column_id.as_deref())
                            {
                                Some(id) if id == column.id => {
                                    match self.state.sort.as_ref().map(|sort| &sort.direction) {
                                        Some(super::DataSortDirection::Asc) => {
                                            super::DataSortDirection::Desc
                                        }
                                        _ => super::DataSortDirection::Asc,
                                    }
                                }
                                _ => super::DataSortDirection::Asc,
                            };
                            self.state.sort =
                                Some(super::DataSortState::new(Some(column.id.clone()), next));
                        }
                    }
                },
            )
            .response;

        let rows_output = ScrollArea::vertical().show_rows(
            ui,
            self.row_height,
            visible_rows.len(),
            |ui, range| {
                for visible_index in range {
                    let row_index = visible_rows[visible_index];
                    let row = &rows[row_index];
                    let selected = self.state.selection.row_id.as_deref() == Some(row.id.as_str());
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), self.row_height),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            for (column_index, column) in &visible_columns {
                                let value = row
                                    .cell(*column_index)
                                    .map(|cell| cell.value.as_str())
                                    .unwrap_or("");
                                let text = if selected {
                                    RichText::new(value).strong().color(Color32::WHITE)
                                } else {
                                    RichText::new(value)
                                };
                                let button =
                                    egui::Button::new(text).selected(selected).frame(false);
                                if ui
                                    .add_sized([column.width, self.row_height], button)
                                    .clicked()
                                {
                                    self.state.select_row(row.id.clone());
                                }
                            }
                        },
                    );
                }
            },
        );
        let rows_response = ui.interact(
            rows_output.inner_rect,
            ui.id().with("data_table_rows"),
            egui::Sense::hover(),
        );

        header_response.union(rows_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::data::{
        DataCell, DataColumn, DataGridModel, DataRow, DataSortDirection, DataSortState,
    };

    #[test]
    fn data_table_prefers_model_filtered_sorted_rows() {
        let model = DataGridModel::new(
            vec![DataColumn::new("name", "Name")],
            vec![
                DataRow::new("alpha", vec![DataCell::new("Alpha")]),
                DataRow::new("beta", vec![DataCell::new("Beta")]),
            ],
        );
        let state = DataGridState {
            sort: Some(DataSortState::new(
                Some("name".into()),
                DataSortDirection::Desc,
            )),
            ..Default::default()
        };
        assert_eq!(model.filtered_sorted_row_indices(&state), vec![1, 0]);
    }

    #[test]
    fn data_table_reports_view_status_messages() {
        let model = DataGridModel::new(vec![DataColumn::new("name", "Name")], vec![]);
        let mut loading = DataGridState::loading();
        let table = DataTable::new(&model, &mut loading);
        assert_eq!(table.status_text().as_deref(), Some("Loading…"));
    }
}
