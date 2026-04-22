mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated/mod.rs"));
}

include!(concat!(env!("OUT_DIR"), "/dispatch.rs"));

use eframe::egui;
use std::fs;
use std::path::Path;

enum AppMode {
    Launcher,
    Preview,
    Building { message: String },
}

struct PreviewApp {
    mode: AppMode,
    selected: Option<String>,
    status: String,
}

impl Default for PreviewApp {
    fn default() -> Self {
        let names = artboard_names();
        let mode = if names.is_empty() {
            AppMode::Launcher
        } else {
            AppMode::Preview
        };
        Self {
            mode,
            selected: names.first().map(|s| s.to_string()),
            status: String::new(),
        }
    }
}

impl eframe::App for PreviewApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        match &mut self.mode {
            AppMode::Launcher => self.show_launcher(ui),
            AppMode::Preview => self.show_preview(ui),
            AppMode::Building { message } => {
                ui.centered_and_justified(|ui| {
                    ui.heading("Building preview...");
                    ui.add_space(8.0);
                    ui.label(message.as_str());
                    ui.add_space(16.0);
                    ui.spinner();
                });
            }
        }
    }
}

impl PreviewApp {
    fn show_launcher(&mut self, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.group(|ui| {
                ui.set_min_width(420.0);
                ui.heading("egui_expressive Preview");
                ui.add_space(12.0);
                ui.label("Select the folder where you exported your artboard code.");
                ui.add_space(16.0);

                if ui.button("[ Select Export Folder ]").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.load_from_folder(path);
                    }
                }

                if !self.status.is_empty() {
                    ui.add_space(12.0);
                    ui.label(&self.status);
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);
                ui.small("Tip: In the Illustrator panel, click 'Save to Folder' and choose a location.");
            });
        });
    }

    fn load_from_folder(&mut self, src: std::path::PathBuf) {
        self.status = format!("Loading from: {}", src.display());

        let generated_dir = Path::new("generated");
        let mut copied = 0;
        // Find artboard files (.rs that aren't the known ones)
        let mut artboard_files = Vec::new();
        match fs::read_dir(&src) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "rs") {
                        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                            if is_valid_module_name(name)
                                && !["mod", "tokens", "state", "components"].contains(&name)
                            {
                                artboard_files.push(name.to_string());
                            }
                        }
                        // Copy all .rs files with valid module names
                        if let Some(fname) = path.file_name().and_then(|n| n.to_str()) {
                            if let Some(stem) = fname.strip_suffix(".rs") {
                                if is_valid_module_name(stem) {
                                    let dest = generated_dir.join(fname);
                                    if let Err(e) = fs::copy(&path, &dest) {
                                        self.status = format!("Failed to copy {}: {}", path.display(), e);
                                        return;
                                    }
                                    copied += 1;
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                self.status = format!("Cannot read folder: {}", e);
                return;
            }
        }

        if copied == 0 {
            self.status = "No .rs files found in that folder.".to_string();
            return;
        }

        // Update mod.rs to include artboard modules
        let mut mod_content = String::from(
            "// Auto-generated module declarations.\n// Artboard modules auto-discovered at build time.\n\npub mod tokens;\npub mod state;\npub mod components;\n"
        );
        for name in &artboard_files {
            mod_content.push_str(&format!("pub mod {};\n", name));
        }
        if let Err(e) = fs::write(generated_dir.join("mod.rs"), mod_content) {
            self.status = format!("Failed to write mod.rs: {}", e);
            return;
        }

        self.status = format!(
            "Copied {} file(s). Rebuilding preview...",
            copied
        );
        self.mode = AppMode::Building {
            message: self.status.clone(),
        };

        // Trigger rebuild in subprocess
        let cwd = match std::env::current_dir() {
            Ok(d) => d,
            Err(e) => {
                self.status = format!("Cannot detect working directory: {}", e);
                self.mode = AppMode::Launcher;
                return;
            }
        };
        match std::process::Command::new("cargo")
            .arg("run")
            .current_dir(cwd)
            .spawn()
        {
            Ok(_) => {
                // Exit launcher so the rebuilt instance takes over
                std::process::exit(0);
            }
            Err(e) => {
                self.status = format!("Failed to start rebuild: {}", e);
                self.mode = AppMode::Launcher;
            }
        }
    }

    fn show_preview(&mut self, ui: &mut egui::Ui) {
        let names = artboard_names();

        // Top bar
        ui.horizontal(|ui| {
            ui.heading("egui_expressive Preview");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("[ Load Different Folder ]").clicked() {
                    // Clear generated files and go back to launcher
                    if let Err(e) = self.clear_generated() {
                        self.status = format!("Warning: failed to clear generated files: {}", e);
                    }
                    self.mode = AppMode::Launcher;
                    self.selected = None;
                    return;
                }
                ui.label(format!("{} artboard(s)", names.len()));
            });
        });
        ui.separator();

        // Sidebar + main area
        ui.horizontal(|ui| {
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

    fn clear_generated(&self,
    ) -> Result<(), std::io::Error> {
        let generated_dir = Path::new("generated");
        // Restore placeholder files
        let placeholders = [
            ("mod.rs", include_str!("../generated/mod.rs")),
            ("tokens.rs", include_str!("../generated/tokens.rs")),
            ("state.rs", include_str!("../generated/state.rs")),
            ("components.rs", include_str!("../generated/components.rs")),
        ];
        for (name, content) in &placeholders {
            fs::write(generated_dir.join(name), content)?;
        }
        // Remove any artboard files
        if let Ok(entries) = fs::read_dir(generated_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "rs") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if !["mod", "tokens", "state", "components"].contains(&stem) {
                            let _ = fs::remove_file(&path);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

fn is_valid_module_name(s: &str) -> bool {
    if s.is_empty() || s.starts_with(|c: char| c.is_ascii_digit()) {
        return false;
    }
    s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
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
