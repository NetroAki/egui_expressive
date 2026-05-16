use egui::{Response, Sense, Ui, Vec2};

pub struct Ruler<'a> {
    pub beats: &'a mut f32,
}

impl<'a> egui::Widget for Ruler<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.allocate_response(Vec2::new(ui.available_width(), 28.0), Sense::hover())
    }
}
