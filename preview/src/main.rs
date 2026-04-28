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

fn is_supported_asset(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| matches!(e.to_ascii_lowercase().as_str(), "png" | "jpg" | "jpeg"))
        .unwrap_or(false)
}

fn same_directory(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

fn read_declared_artboards(src: &Path) -> Result<Option<Vec<String>>, String> {
    let mod_path = src.join("mod.rs");
    if !mod_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&mod_path)
        .map_err(|e| format!("Failed to read {}: {}", mod_path.display(), e))?;
    let mut artboards = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("pub mod ") else {
            continue;
        };
        let name = rest.trim_end_matches(';').trim();
        if is_valid_module_name(name) && !["tokens", "state", "components"].contains(&name) {
            artboards.push(name.to_string());
        }
    }

    Ok(Some(artboards))
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
                ui.small(
                    "Tip: In the Illustrator panel, click 'Save to Folder' and choose a location.",
                );
            });
        });
    }

    fn load_from_folder(&mut self, src: PathBuf) {
        self.status = format!("Loading from: {}", src.display());

        let generated_dir = generated_dir();
        if let Err(e) = fs::create_dir_all(&generated_dir) {
            self.status = format!("Failed to create generated directory: {}", e);
            return;
        }
        let loading_generated_dir = same_directory(&src, &generated_dir);
        // Clear any stale files from previous loads unless the user selected preview/generated itself.
        // In that case clearing first would delete the files we are trying to load.
        if !loading_generated_dir {
            let _ = self.clear_generated();
        }
        let declared_artboards = match read_declared_artboards(&src) {
            Ok(artboards) => artboards,
            Err(e) => {
                self.status = e;
                return;
            }
        };
        let mut copied = 0;
        let mut copied_assets = 0;

        // Copy assets folder if it exists
        let src_assets = src.join("assets");
        let dest_assets = generated_dir.join("assets");
        if src_assets.exists() && src_assets.is_dir() {
            if !loading_generated_dir {
                let _ = fs::create_dir_all(&dest_assets);
                if let Ok(entries) = fs::read_dir(&src_assets) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if is_supported_asset(&path) {
                            let dest = dest_assets.join(path.file_name().unwrap());
                            if let Err(e) = fs::copy(&path, &dest) {
                                self.status =
                                    format!("Failed to copy asset {}: {}", path.display(), e);
                                return;
                            }
                            copied_assets += 1;
                        }
                    }
                }
            } else if let Ok(entries) = fs::read_dir(&src_assets) {
                for entry in entries.flatten() {
                    if is_supported_asset(&entry.path()) {
                        copied_assets += 1;
                    }
                }
            }
        }

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
                                let is_core = ["tokens", "state", "components"].contains(&stem);
                                let is_declared = declared_artboards
                                    .as_ref()
                                    .map(|names| names.iter().any(|name| name == stem))
                                    .unwrap_or(!is_core);
                                if !is_core && is_declared {
                                    artboard_files.push(stem.to_string());
                                }
                                if is_core || is_declared {
                                    if !loading_generated_dir {
                                        let dest = generated_dir.join(path.file_name().unwrap());
                                        if let Err(e) = fs::copy(&path, &dest) {
                                            self.status =
                                                format!("Failed to copy {}: {}", path.display(), e);
                                            return;
                                        }
                                    }
                                    copied += 1;
                                }
                            }
                        }
                    } else if is_supported_asset(&path) {
                        if !loading_generated_dir {
                            let dest = generated_dir.join(path.file_name().unwrap());
                            if let Err(e) = fs::copy(&path, &dest) {
                                self.status =
                                    format!("Failed to copy asset {}: {}", path.display(), e);
                                return;
                            }
                        }
                        copied_assets += 1;
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
            "// Auto-generated module declarations.\n// Artboard modules auto-discovered at build time.\n\n"
        );
        for core in ["tokens", "state", "components"] {
            if generated_dir.join(format!("{}.rs", core)).exists() {
                mod_content.push_str(&format!("pub mod {};\n", core));
            }
        }
        for name in &artboard_files {
            mod_content.push_str(&format!("pub mod {};\n", name));
        }
        if let Err(e) = fs::write(generated_dir.join("mod.rs"), mod_content) {
            self.status = format!("Failed to write mod.rs: {}", e);
            return;
        }

        self.status = format!(
            "Copied {} Rust file(s) and {} asset(s). Rebuilding preview...",
            copied, copied_assets
        );

        let cargo_path = std::env::var("CARGO").unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            let cargo_home =
                std::env::var("CARGO_HOME").unwrap_or_else(|_| format!("{}/.cargo", home));
            let cargo_bin = format!("{}/bin/cargo", cargo_home);
            if Path::new(&cargo_bin).exists() {
                cargo_bin
            } else {
                "cargo".to_string()
            }
        });

        match std::process::Command::new(cargo_path)
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
        let preview_height = ui.available_height();
        ui.horizontal(|ui| {
            ui.set_min_height(preview_height);
            ui.vertical(|ui| {
                ui.set_min_width(180.0);
                ui.set_max_width(240.0);
                ui.set_min_height(preview_height);
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
                ui.set_min_height(preview_height);
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
        // Restore the generated module root and remove exported artboard files.
        let generated_roots = [
            ("mod.rs", "// Auto-generated module declarations.\n// Artboard modules will be auto-discovered at build time.\n"),
        ];
        for (name, content) in &generated_roots {
            fs::write(generated_dir.join(name), content)?;
        }
        // Remove any artboard files
        let entries = fs::read_dir(&generated_dir)?;
        for entry_res in entries {
            let entry = entry_res?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "rs") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if stem != "mod" {
                        fs::remove_file(&path)?;
                    }
                }
            } else if is_supported_asset(&path) {
                fs::remove_file(&path)?;
            }
        }

        // Also remove assets folder
        let assets_dir = generated_dir.join("assets");
        if assets_dir.exists() {
            fs::remove_dir_all(&assets_dir)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_supported_preview_assets() {
        assert!(is_supported_asset(Path::new("linked_asset.png")));
        assert!(is_supported_asset(Path::new("photo.JPG")));
        assert!(is_supported_asset(Path::new("photo.jpeg")));
        assert!(!is_supported_asset(Path::new("vector.svg")));
        assert!(!is_supported_asset(Path::new("artboard.rs")));
    }

    #[test]
    fn test_read_declared_artboards() {
        let temp_dir = std::env::temp_dir().join("egui_expressive_test_read_declared");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        let mod_path = temp_dir.join("mod.rs");

        // Test missing mod.rs
        assert_eq!(read_declared_artboards(&temp_dir).unwrap(), None);

        // Test valid mod.rs
        let content = "pub mod tokens;\npub mod state;\npub mod components;\npub mod my_artboard;\npub mod another_artboard;";
        fs::write(&mod_path, content).unwrap();

        let artboards = read_declared_artboards(&temp_dir).unwrap().unwrap();
        assert_eq!(artboards, vec!["my_artboard", "another_artboard"]);
    }

    #[test]
    fn test_clear_generated() {
        let app = PreviewApp::default();
        let generated_dir = generated_dir();

        // Create some dummy files
        fs::create_dir_all(&generated_dir).unwrap();
        fs::write(generated_dir.join("dummy.rs"), "fn dummy() {}").unwrap();
        fs::write(generated_dir.join("dummy.png"), "dummy").unwrap();

        let assets_dir = generated_dir.join("assets");
        fs::create_dir_all(&assets_dir).unwrap();
        fs::write(assets_dir.join("dummy_asset.png"), "dummy").unwrap();

        app.clear_generated().unwrap();

        assert!(!generated_dir.join("dummy.rs").exists());
        assert!(!generated_dir.join("dummy.png").exists());
        assert!(!assets_dir.exists());
        assert!(generated_dir.join("mod.rs").exists());
        assert!(!generated_dir.join("tokens.rs").exists());
    }
}
