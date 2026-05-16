use egui::{Response, Sense, Ui, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipKind {
    Audio,
    Midi,
    Automation,
}

pub struct TimelineClip<'a> {
    pub start: &'a mut f32,
    pub length: &'a mut f32,
    pub kind: ClipKind,
}

impl<'a> egui::Widget for TimelineClip<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.allocate_response(
            Vec2::new(ui.available_width(), 24.0),
            Sense::click_and_drag(),
        )
    }
}
