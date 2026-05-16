use egui::{Color32, Stroke, Ui};

/// Dense control-group/card primitive for pro-audio panels.
pub struct ControlGroup<'a> {
    title: Option<String>,
    frame: egui::Frame,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> ControlGroup<'a> {
    pub fn new() -> Self {
        Self {
            title: None,
            frame: egui::Frame::group(&egui::Style::default())
                .fill(Color32::from_rgb(22, 22, 27))
                .stroke(Stroke::new(1.0, Color32::from_rgb(40, 40, 47))),
            _marker: std::marker::PhantomData,
        }
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
    pub fn frame(mut self, frame: egui::Frame) -> Self {
        self.frame = frame;
        self
    }
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        self.frame.show(ui, |ui| {
            if let Some(title) = self.title {
                ui.label(egui::RichText::new(title.to_uppercase()).size(9.0).weak());
            }
            add_contents(ui)
        })
    }
}

impl<'a> Default for ControlGroup<'a> {
    fn default() -> Self {
        Self::new()
    }
}
