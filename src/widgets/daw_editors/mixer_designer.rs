use egui::{Response, Ui};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MixerStripSection {
    pub id: String,
    pub label: String,
    pub visible: bool,
}

pub struct MixerStripDesigner<'a> {
    pub sections: &'a mut [MixerStripSection],
}

impl<'a> egui::Widget for MixerStripDesigner<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.heading("Mixer strip designer");
            for section in self.sections.iter_mut() {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut section.visible, &section.label);
                    ui.label(&section.id);
                });
            }
        })
        .response
    }
}
