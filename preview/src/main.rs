mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated/mod.rs"));
}

mod shared;
use shared::is_valid_module_name;

include!(concat!(env!("OUT_DIR"), "/dispatch.rs"));

use eframe::egui;
use std::fs;
use std::path::{Path, PathBuf};

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

fn generated_dir() -> PathBuf {
    Path::new(MANIFEST_DIR).join("generated")
}

enum AppMode {
    Launcher,
    Preview,
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

    fn load_from_folder(&mut self, src: PathBuf) {
        self.status = format!("Loading from: {}", src.display());

        let generated_dir = generated_dir();
        // Clear any stale files from previous loads
        let _ = self.clear_generated();
        if let Err(e) = fs::create_dir_all(&generated_dir) {
            self.status = format!("Failed to create generated directory: {}", e);
            return;
        }
        let mut copied = 0;
        // Find artboard files (.rs that aren't the known ones)
        let mut artboard_files = Vec::new();
        match fs::read_dir(&src) {
            Ok(entries) => {
                for entry_res in entries {
                    let entry = match entry_res {
                        Ok(e) => e,
                        Err(e) => {
                            self.status = format!("Failed to read directory entry: {}", e);
                            return;
                        }
                    };
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "rs") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            if is_valid_module_name(stem) {
                                if !["tokens", "state", "components"].contains(&stem) {
                                    artboard_files.push(stem.to_string());
                                }
                                let dest = generated_dir.join(path.file_name().unwrap());
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

        match std::process::Command::new("cargo")
            .arg("run")
            .current_dir(MANIFEST_DIR)
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
                egui::ScrollArea::vertical()
                    .id_salt("preview_artboard_list")
                    .show(ui, |ui| {
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
                    egui::ScrollArea::both()
                        .id_salt(("preview_artboard_canvas", name))
                        .show(ui, |ui| {
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

    fn clear_generated(&self) -> Result<(), std::io::Error> {
        let generated_dir = generated_dir();
        fs::create_dir_all(&generated_dir)?;
        // Restore placeholder files
        let placeholders = [
            ("mod.rs", "// Auto-generated module declarations.\n// Artboard modules will be auto-discovered at build time.\n\npub mod tokens;\npub mod state;\npub mod components;\n"),
            ("tokens.rs", ""),
            ("state.rs", ""),
            ("components.rs", ""),
        ];
        for (name, content) in &placeholders {
            fs::write(generated_dir.join(name), content)?;
        }
        // Remove any artboard files
        let entries = fs::read_dir(generated_dir)?;
        for entry_res in entries {
            let entry = match entry_res {
                Ok(e) => e,
                Err(e) => return Err(e),
            };
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "rs") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if !["mod", "tokens", "state", "components"].contains(&stem) {
                        fs::remove_file(&path)?;
                    }
                }
            }
        }
        Ok(())
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
