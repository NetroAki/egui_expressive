use eframe::egui;
use egui_expressive::widgets::SearchField;
use egui_expressive::{
    AppShellLayoutState, AppShellPanelState, BreadcrumbItem, Breadcrumbs, DataCell, DataColumn,
    DataGridModel, DataGridState, DataRow, DataSortDirection, DataSortState, DataTable,
    DataViewStatus, PropertyGrid, PropertyGridEntry, PropertyGridModel, ResizableSplit,
    SidebarItem, SidebarNav, StatusBar, StatusBarItem, TabBar, TabSetState, TreeTable,
    TreeTableModel, TreeTableNode, TreeTableState, Tw,
};

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Data Explorer Dashboard",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(DataExplorerDashboard::default()))),
    )
}

struct DataExplorerDashboard {
    nav: String,
    tabs: TabSetState,
    split_fraction: f32,
    layout: AppShellLayoutState,
    search: String,
    data_state: DataGridState,
    data_model: DataGridModel,
    tree_state: TreeTableState,
    tree_model: TreeTableModel,
    property_model: PropertyGridModel,
    sidebar_items: Vec<SidebarItem>,
    breadcrumbs: Vec<BreadcrumbItem>,
}

impl Default for DataExplorerDashboard {
    fn default() -> Self {
        let data_model = DataGridModel::new(
            vec![
                DataColumn::new("name", "Name").width(180.0),
                DataColumn::new("type", "Type").width(110.0),
                DataColumn::new("status", "Status").width(120.0),
            ],
            vec![
                DataRow::new(
                    "alpha",
                    vec![
                        DataCell::new("Alpha"),
                        DataCell::new("File"),
                        DataCell::new("Ready"),
                    ],
                ),
                DataRow::new(
                    "beta",
                    vec![
                        DataCell::new("Beta"),
                        DataCell::new("Folder"),
                        DataCell::new("Paused"),
                    ],
                ),
                DataRow::new(
                    "gamma",
                    vec![
                        DataCell::new("Gamma"),
                        DataCell::new("File"),
                        DataCell::new("Ready"),
                    ],
                ),
            ],
        );

        let tree_model = TreeTableModel::new(
            vec![DataColumn::new("label", "Label")],
            vec![TreeTableNode::new("project", "Project").with_children(vec![
                TreeTableNode::new("assets", "Assets").with_children(vec![
                    TreeTableNode::new("images", "Images"),
                    TreeTableNode::new("docs", "Docs"),
                ]),
                TreeTableNode::new("library", "Library"),
            ])],
        );

        let property_model = PropertyGridModel::new(vec![
            PropertyGridEntry::new("Name", "Data Explorer", "General").group("Identity"),
            PropertyGridEntry::new("Rows", "3", "General").group("Metrics"),
            PropertyGridEntry::new("Status", "Ready", "Metrics").group("Metrics"),
        ]);

        let main_panel = AppShellPanelState::new(egui_expressive::DockPanel::new(
            "main",
            "Main",
            egui_expressive::DockPlacement::docked(egui_expressive::DockZone::Center),
        ));

        Self {
            nav: "table".into(),
            tabs: TabSetState::new(0),
            split_fraction: 0.24,
            layout: AppShellLayoutState::with_panels(vec![main_panel]),
            search: String::new(),
            data_state: DataGridState {
                sort: Some(DataSortState::new(
                    Some("name".into()),
                    DataSortDirection::Asc,
                )),
                ..DataGridState::default()
            },
            data_model,
            tree_state: TreeTableState::default(),
            tree_model,
            property_model,
            sidebar_items: vec![
                SidebarItem::new("table", "Table").icon("▦"),
                SidebarItem::new("tree", "Tree").icon("⮞"),
                SidebarItem::new("properties", "Properties").icon("◎"),
                SidebarItem::new("states", "States").icon("◌"),
            ],
            breadcrumbs: vec![
                BreadcrumbItem::new("dash", "Dashboard"),
                BreadcrumbItem::new("data", "Data Explorer"),
            ],
        }
    }
}

impl eframe::App for DataExplorerDashboard {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.tabs.recover(4);
        let mut nav = self.nav.clone();
        let original_nav = nav.clone();
        let mut selected_tab = self.tabs.selected.min(3);
        let status = vec![
            StatusBarItem::new("Rows").value(self.data_model.rows().len().to_string()),
            StatusBarItem::new("Layout").value(if self.layout.sidebar_collapsed {
                "compact"
            } else {
                "wide"
            }),
            StatusBarItem::new("Mode").value(match &self.data_state.view_status {
                DataViewStatus::Ready => "ready",
                DataViewStatus::Empty => "empty",
                DataViewStatus::Loading => "loading",
                DataViewStatus::Error(_) => "error",
            }),
        ];

        ResizableSplit::new(
            "data_explorer_split",
            &mut self.split_fraction,
            egui_expressive::SplitAxis::Horizontal,
        )
        .show(
            ui,
            |ui| {
                Tw::new().p(8.0).show(ui, |ui| {
                    ui.heading("Data Explorer");
                    ui.checkbox(&mut self.layout.sidebar_collapsed, "Collapse sidebar");
                    ui.add(
                        SidebarNav::new(&mut nav, &self.sidebar_items)
                            .collapsed(self.layout.sidebar_collapsed),
                    );
                    ui.separator();
                });
            },
            |ui| {
                ui.add(Breadcrumbs::new(&self.breadcrumbs));
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Filter rows");
                    ui.add(SearchField::new(&mut self.search));
                });
                ui.separator();
                self.data_state.filter.query = self.search.clone();
                ui.horizontal(|ui| {
                    if ui.button("Ready").clicked() {
                        self.data_state.view_status = DataViewStatus::Ready;
                    }
                    if ui.button("Loading").clicked() {
                        self.data_state.view_status = DataViewStatus::Loading;
                    }
                    if ui.button("Empty").clicked() {
                        self.data_state.view_status = DataViewStatus::Empty;
                    }
                    if ui.button("Error").clicked() {
                        self.data_state.view_status =
                            DataViewStatus::Error("Network timeout".into());
                    }
                });
                let mut tab = selected_tab;
                ui.add(TabBar::new(
                    &mut tab,
                    vec![
                        "Table".into(),
                        "Tree".into(),
                        "Properties".into(),
                        "States".into(),
                    ],
                ));
                selected_tab = tab.min(3);
                ui.add_space(8.0);
                match selected_tab {
                    0 => {
                        ui.add(
                            DataTable::new(&self.data_model, &mut self.data_state).row_height(26.0),
                        );
                    }
                    1 => {
                        ui.add(TreeTable::new(&self.tree_model, &mut self.tree_state));
                    }
                    2 => {
                        ui.add(PropertyGrid::new(&self.property_model));
                    }
                    _ => {
                        ui.label("Read-only state samples");
                        ui.label(format!(
                            "Selected row: {:?}",
                            self.data_state.selection.row_id
                        ));
                        ui.label(format!(
                            "Selected column: {:?}",
                            self.data_state.selection.column_id
                        ));
                        ui.label(format!(
                            "Visible columns: {:?}",
                            self.data_state
                                .visible_column_ids(self.data_model.columns())
                        ));
                    }
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.separator();
                    ui.add(StatusBar::new(&status));
                });
            },
        );
        if nav != original_nav {
            self.tabs.selected = tab_for_nav(&nav);
            self.nav = nav;
        } else {
            self.tabs.selected = selected_tab;
            self.nav = nav_for_tab(selected_tab).into();
        }
    }
}

fn tab_for_nav(nav: &str) -> usize {
    match nav {
        "tree" => 1,
        "properties" => 2,
        "states" => 3,
        _ => 0,
    }
}

fn nav_for_tab(tab: usize) -> &'static str {
    match tab {
        1 => "tree",
        2 => "properties",
        3 => "states",
        _ => "table",
    }
}
