use egui::{Response, Ui};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginManagerItem {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub category: String,
    pub enabled: bool,
    pub favorite: bool,
}

pub struct PluginManager<'a> {
    pub query: &'a mut String,
    pub plugins: &'a mut [PluginManagerItem],
}

impl<'a> egui::Widget for PluginManager<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.text_edit_singleline(self.query);
        let query = self.query.to_lowercase();
        let mut hidden_matches = 0usize;
        for plugin in self
            .plugins
            .iter_mut()
            .filter(|p| p.name.to_lowercase().contains(&query))
        {
            if hidden_matches >= 8 {
                hidden_matches += 1;
                continue;
            }

            ui.horizontal(|ui| {
                ui.checkbox(&mut plugin.enabled, "");
                ui.toggle_value(&mut plugin.favorite, "★");
                ui.label(format!(
                    "{} — {} / {}",
                    plugin.name, plugin.vendor, plugin.category
                ));
            });
            hidden_matches += 1;
        }
        if hidden_matches > 8 {
            ui.label(format!("{} more plugins match", hidden_matches - 8));
        }
        ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
    }
}
