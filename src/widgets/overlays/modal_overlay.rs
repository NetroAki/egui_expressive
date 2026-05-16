use egui::{Color32, Id, Response, Sense, Ui, Vec2};

pub struct ModalOverlay<'a> {
    id: Id,
    title: String,
    close_requested: Option<&'a mut bool>,
    click_outside_to_close: bool,
    escape_to_close: bool,
}

impl<'a> ModalOverlay<'a> {
    pub fn new() -> Self {
        Self {
            id: Id::new("egui_expressive_modal"),
            title: "Modal".to_owned(),
            close_requested: None,
            click_outside_to_close: true,
            escape_to_close: true,
        }
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    pub fn close_requested(mut self, flag: &'a mut bool) -> Self {
        self.close_requested = Some(flag);
        self
    }
    pub fn click_outside_to_close(mut self, enabled: bool) -> Self {
        self.click_outside_to_close = enabled;
        self
    }
    pub fn escape_to_close(mut self, enabled: bool) -> Self {
        self.escape_to_close = enabled;
        self
    }
    pub fn show(self, ui: &mut Ui, add: impl FnOnce(&mut Ui)) -> Response {
        let rect = ui.ctx().input(|input| input.content_rect());
        let resp = ui.allocate_rect(rect, Sense::click());
        ui.painter().rect_filled(
            resp.rect,
            0.0,
            Color32::from_rgba_unmultiplied(0, 0, 0, 150),
        );
        let mut close = self.escape_to_close && ui.input(|i| i.key_pressed(egui::Key::Escape));
        close |= self.click_outside_to_close && resp.clicked();
        egui::Area::new(self.id)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .order(egui::Order::Tooltip)
            .show(ui.ctx(), |ui| {
                egui::Frame::window(ui.style()).show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.heading(&self.title);
                        ui.separator();
                        add(ui);
                    });
                });
            });
        if let Some(flag) = self.close_requested {
            *flag |= close;
        }
        resp
    }
}

impl<'a> Default for ModalOverlay<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> egui::Widget for ModalOverlay<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui, |_| {})
    }
}
