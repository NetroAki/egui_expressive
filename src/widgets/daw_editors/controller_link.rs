use egui::{Response, Ui};

#[derive(Clone, Debug, PartialEq)]
pub struct ControllerLinkState {
    pub target: String,
    pub source: String,
    pub automation_enabled: bool,
    pub learn_mode: bool,
}

pub struct ControllerLinkOverlay<'a> {
    pub state: &'a mut ControllerLinkState,
}

impl<'a> egui::Widget for ControllerLinkOverlay<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.heading("Controller link");
            ui.label(format!("Target: {}", self.state.target));
            ui.text_edit_singleline(&mut self.state.source);
            ui.checkbox(&mut self.state.automation_enabled, "Automation link");
            ui.toggle_value(&mut self.state.learn_mode, "MIDI learn");
        })
        .response
    }
}
