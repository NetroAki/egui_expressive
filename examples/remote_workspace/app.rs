use eframe::egui;

use super::tokens::WorkspaceTokens;
use super::widgets::StatusKind;

pub struct RemoteWorkspaceShowcase {
    pub(super) tokens: WorkspaceTokens,
    pub(super) selected_nav: usize,
    pub(super) selected_tab: usize,
    pub(super) selected_host: usize,
    pub(super) selected_state: StatusKind,
    pub(super) address: String,
    pub(super) port: String,
    pub(super) username: String,
    pub(super) vault: usize,
    pub(super) connection_message: String,
}

impl RemoteWorkspaceShowcase {
    pub fn new(ctx: &egui::Context) -> Self {
        super::fonts::install(ctx);
        let tokens = WorkspaceTokens::default();
        tokens.apply(ctx);
        Self {
            tokens,
            selected_nav: 0,
            selected_tab: 0,
            selected_host: 0,
            selected_state: StatusKind::Focus,
            address: "10.0.0.15".to_owned(),
            port: "22".to_owned(),
            username: "admin".to_owned(),
            vault: 0,
            connection_message: "Ready for a local connection".to_owned(),
        }
    }
}

impl eframe::App for RemoteWorkspaceShowcase {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(250));
        self.tokens.apply(ui.ctx());

        egui::Panel::left("workspace_nav")
            .exact_size(76.0)
            .resizable(false)
            .frame(
                egui::Frame::NONE
                    .fill(self.tokens.sidebar)
                    .inner_margin(egui::Margin::same(10)),
            )
            .show_inside(ui, |ui| self.show_navigation(ui));

        egui::Panel::top("workspace_tabs")
            .exact_size(76.0)
            .frame(
                egui::Frame::NONE
                    .fill(self.tokens.panel)
                    .stroke(egui::Stroke::new(1.0, self.tokens.border))
                    .inner_margin(egui::Margin::symmetric(20, 14)),
            )
            .show_inside(ui, |ui| self.show_tabs(ui));

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(self.tokens.bg_deep)
                    .inner_margin(egui::Margin::symmetric(40, 32)),
            )
            .show_inside(ui, |ui| {
                let content_width = ui.available_width();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.set_width(content_width);
                    ui.set_max_width(content_width);
                    self.show_intro(ui);
                    ui.add_space(24.0);
                    self.show_tokens(ui);
                    ui.add_space(24.0);
                    self.show_components(ui);
                    ui.add_space(20.0);
                });
            });
    }
}
