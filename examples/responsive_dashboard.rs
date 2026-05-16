use eframe::egui;
use egui_expressive::{
    AppShellLayoutState, AppShellPanelState, BreadcrumbItem, Breadcrumbs, DockPanel, DockPlacement,
    DockZone, ResizableSplit, SidebarItem, SidebarNav, StatusBar, StatusBarItem, TabBar,
    TabSetState, Tw,
};

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Responsive Dashboard",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(ResponsiveDashboard::default()))),
    )
}

struct ResponsiveDashboard {
    selected_nav: String,
    tab_state: TabSetState,
    split_fraction: f32,
    layout: AppShellLayoutState,
    sidebar_items: Vec<SidebarItem>,
    breadcrumbs: Vec<BreadcrumbItem>,
}

impl Default for ResponsiveDashboard {
    fn default() -> Self {
        let main_panel = AppShellPanelState::new(DockPanel::new(
            "main",
            "Main",
            DockPlacement::docked(DockZone::Center),
        ))
        .with_tab_set("dashboard-tabs");
        Self {
            selected_nav: "overview".into(),
            tab_state: TabSetState::new(0),
            split_fraction: 0.26,
            layout: AppShellLayoutState::with_panels(vec![main_panel]),
            sidebar_items: vec![
                SidebarItem::new("overview", "Overview").icon("⌂"),
                SidebarItem::new("reports", "Reports").icon("▦"),
                SidebarItem::new("settings", "Settings").icon("⚙"),
            ],
            breadcrumbs: vec![
                BreadcrumbItem::new("dashboard", "Dashboard"),
                BreadcrumbItem::new("overview", "Overview"),
            ],
        }
    }
}

impl eframe::App for ResponsiveDashboard {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let tabs = vec![
            "Summary".to_owned(),
            "Activity".to_owned(),
            "Settings".to_owned(),
        ];
        let selected_nav = self.selected_nav.clone();
        let split_fraction = self.split_fraction;
        self.tab_state.recover(tabs.len());
        let visible_panel_count = self.layout.visible_panels().count();
        let closable_panel_count = self
            .layout
            .visible_panels()
            .filter(|panel| panel.panel.closable())
            .count();
        let primary_panel_title = self
            .layout
            .visible_panels()
            .next()
            .map(|panel| panel.panel.title())
            .unwrap_or("none")
            .to_owned();
        let status = vec![
            StatusBarItem::new("Layout").value("recovered"),
            StatusBarItem::new("Panels").value(visible_panel_count.to_string()),
            StatusBarItem::new("Closable").value(closable_panel_count.to_string()),
        ];

        ResizableSplit::new(
            "dashboard_split",
            &mut self.split_fraction,
            egui_expressive::SplitAxis::Horizontal,
        )
        .show(
            ui,
            |ui| {
                Tw::new().p(8.0).show(ui, |ui| {
                    ui.heading("App Shell");
                    ui.checkbox(&mut self.layout.sidebar_collapsed, "Collapse sidebar");
                    ui.add(
                        SidebarNav::new(&mut self.selected_nav, &self.sidebar_items)
                            .collapsed(self.layout.sidebar_collapsed),
                    );
                });
            },
            |ui| {
                ui.add(Breadcrumbs::new(&self.breadcrumbs));
                ui.separator();
                ui.add(TabBar::new(&mut self.tab_state.selected, tabs));
                ui.add_space(8.0);
                Tw::new().p(16.0).rounded_lg().show(ui, |ui| {
                    ui.heading("Responsive dashboard proof");
                    ui.label("Stage 2 composes split panes, sidebar navigation, breadcrumbs, tabs, status, and persistent layout state.");
                    ui.label(format!("Selected nav: {selected_nav}"));
                    ui.label(format!("Selected tab index: {}", self.tab_state.selected));
                    ui.label(format!("Primary panel: {primary_panel_title}"));
                    ui.label(format!("Split fraction: {split_fraction:.2}"));
                });
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.separator();
                    ui.add(StatusBar::new(&status));
                });
            },
        );
        self.layout.split_fraction = self.split_fraction;
    }
}
