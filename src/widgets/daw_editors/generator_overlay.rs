use egui::{Response, Ui};

#[derive(Clone, Debug, PartialEq)]
pub struct GeneratorSlot {
    pub name: String,
    pub enabled: bool,
    pub macro_value: f32,
}

pub struct GeneratorOverlay<'a> {
    pub title: &'a str,
    pub slots: &'a mut [GeneratorSlot],
}

impl<'a> egui::Widget for GeneratorOverlay<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.heading(self.title);
            for slot in self.slots.iter_mut() {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut slot.enabled, &slot.name);
                    ui.add(egui::Slider::new(&mut slot.macro_value, 0.0..=1.0).text("macro"));
                });
            }
        })
        .response
    }
}
