mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated/mod.rs"));
}

include!(concat!(env!("OUT_DIR"), "/dispatch.rs"));

use eframe::egui;

struct PreviewApp {
    selected: Option<String>,
}

impl Default for PreviewApp {
    fn default() -> Self {
        let names = artboard_names();
        Self {
            selected: names.first().map(|s| s.to_string()),
        }
    }
}

impl eframe::App for PreviewApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let names = artboard_names();

        // Top bar
        ui.horizontal(|ui| {
            ui.heading("egui_expressive Preview");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if names.is_empty() {
                    ui.label("No artboards loaded");
                } else {
                    ui.label(format!("{} artboard(s)", names.len()));
                }
            });
        });
        ui.separator();

        if names.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.group(|ui| {
                    ui.set_min_width(400.0);
                    ui.heading("No generated files found");
                    ui.add_space(8.0);
                    ui.label("Copy your exported .rs files to preview/generated/ and run cargo run again.");
                    ui.add_space(4.0);
                    ui.label("Files needed: mod.rs, tokens.rs, state.rs, components.rs, <artboard>.rs");
                });
            });
            return;
        }

        // Sidebar + main area
        ui.horizontal(|ui| {
            // Sidebar
            ui.vertical(|ui| {
                ui.set_min_width(180.0);
                ui.set_max_width(240.0);
                ui.heading("Artboards");
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for name in names {
                        let selected = self.selected.as_deref() == Some(*name);
                        if ui.selectable_label(selected, *name).clicked() {
                            self.selected = Some(name.to_string());
                        }
                    }
                });
            });

            ui.separator();

            // Main area
            ui.vertical(|ui| {
                ui.set_min_width(600.0);
                if let Some(ref name) = self.selected {
                    egui::ScrollArea::both().show(ui, |ui| {
                        render_artboard(name, ui);
                    });
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Select an artboard from the sidebar");
                    });
                }
            });
        });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("egui_expressive Preview"),
        ..Default::default()
    };

    eframe::run_native(
        "egui_expressive Preview",
        options,
        Box::new(|_cc| Ok(Box::new(PreviewApp::default()))),
    )
}
