use egui::{Color32, Response, Sense, Stroke, Ui, Vec2};

pub struct ColorSwatch<'a> {
    color: &'a mut Color32,
    size: Vec2,
    label: Option<String>,
}

impl<'a> ColorSwatch<'a> {
    pub fn new(color: &'a mut Color32) -> Self {
        Self {
            color,
            size: Vec2::splat(24.0),
            label: None,
        }
    }
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl<'a> egui::Widget for ColorSwatch<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::click());
        ui.painter().rect_filled(rect, 4.0, *self.color);
        ui.painter().rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, ui.visuals().widgets.inactive.bg_stroke.color),
            egui::StrokeKind::Inside,
        );
        if let Some(label) = self.label {
            response.on_hover_text(label)
        } else {
            response
        }
    }
}
