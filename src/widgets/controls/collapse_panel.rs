/// A collapsible panel with smooth easing animation.
pub struct CollapsePanel<'a> {
    title: &'a str,
    default_open: bool,
}

impl<'a> CollapsePanel<'a> {
    pub fn new(id: impl std::hash::Hash, title: &'a str) -> Self {
        let _ = id;
        Self {
            title,
            default_open: true,
        }
    }
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }
    pub fn show(self, ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) -> bool {
        let _ = egui::CollapsingHeader::new(self.title)
            .default_open(self.default_open)
            .show(ui, add_contents);
        self.default_open
    }
}
