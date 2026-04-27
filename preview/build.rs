use std::fs;
use std::path::{Path, PathBuf};

include!("src/shared.rs");
include!("src/naming.rs");

fn read_declared_artboards(dir: &Path) -> Option<Vec<String>> {
    let mod_path = dir.join("mod.rs");
    let content = fs::read_to_string(&mod_path).ok()?;
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
    Some(artboards)
}

fn generate_dispatch_code(artboards: &[String]) -> String {
    let mut dispatch = String::new();
    dispatch.push_str("use egui::Ui;\n\n");

    if artboards.is_empty() {
        dispatch.push_str("pub fn artboard_names() -> &'static [&'static str] { &[] }\n");
        dispatch.push_str("pub fn render_artboard(_name: &str, _ui: &mut Ui) -> bool { false }\n");
    } else {
        dispatch.push_str("pub fn artboard_names() -> &'static [&'static str] {\n");
        dispatch.push_str("    &[\n");
        for name in artboards {
            dispatch.push_str(&format!("        \"{}\",\n", name));
        }
        dispatch.push_str("    ]\n");
        dispatch.push_str("}\n\n");

        dispatch.push_str("pub fn render_artboard(name: &str, ui: &mut Ui) -> bool {\n");
        dispatch.push_str("    trait HandleAction { fn handle(self); }\n");
        dispatch.push_str(
            "    impl<T> HandleAction for Option<T> { fn handle(self) { let _ = self; } }\n",
        );
        dispatch.push_str("    impl HandleAction for () { fn handle(self) {} }\n");
        dispatch.push_str("    match name {\n");
        for name in artboards {
            let pascal = snake_to_pascal(name);
            dispatch.push_str(&format!("        \"{}\" => {{\n", name));
            dispatch.push_str(&format!(
                "            let id = egui::Id::new(\"{}_state\");\n",
                name
            ));
            dispatch.push_str(&format!(
                "            let mut state = ui.data_mut(|d| d.remove_temp::<crate::generated::state::{}State>(id)).unwrap_or_default();\n",
                pascal
            ));
            dispatch.push_str(&format!(
                "            crate::generated::{}::draw_{}(ui, &mut state).handle();\n",
                name, name
            ));
            dispatch.push_str("            ui.data_mut(|d| d.insert_temp(id, state));\n");
            dispatch.push_str("            true\n");
            dispatch.push_str("        }\n");
        }
        dispatch.push_str("        _ => false,\n");
        dispatch.push_str("    }\n");
        dispatch.push_str("}\n");
    }
    dispatch
}

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR set by Cargo"));
    let generated_src = Path::new("generated");
    let generated_out = out_dir.join("generated");

    fs::create_dir_all(&generated_out).expect("create OUT_DIR/generated");
    let declared_artboards = read_declared_artboards(generated_src);

    // Clear stale files from previous builds
    match fs::read_dir(&generated_out) {
        Ok(entries) => {
            for entry_res in entries {
                let entry = match entry_res {
                    Ok(e) => e,
                    Err(e) => {
                        println!(
                            "cargo:warning=failed to read directory entry in generated_out: {}",
                            e
                        );
                        continue;
                    }
                };
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "rs") {
                    if let Err(e) = fs::remove_file(&path) {
                        println!(
                            "cargo:warning=failed to remove stale file {}: {}",
                            path.display(),
                            e
                        );
                    }
                }
            }
        }
        Err(e) => println!(
            "cargo:warning=failed to read generated_out directory: {}",
            e
        ),
    }

    // Copy all .rs files from generated/ to OUT_DIR/generated/
    let mut has_files = false;
    if generated_src.exists() {
        // Also copy assets folder if it exists
        let assets_src = generated_src.join("assets");
        let assets_out = generated_out.join("assets");
        if assets_src.exists() && assets_src.is_dir() {
            if let Err(e) = fs::create_dir_all(&assets_out) {
                println!(
                    "cargo:warning=failed to create assets output directory: {}",
                    e
                );
            } else if let Ok(entries) = fs::read_dir(&assets_src) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let dest = assets_out.join(path.file_name().unwrap());
                        match fs::copy(&path, &dest) {
                            Ok(_) => println!("cargo:rerun-if-changed={}", path.display()),
                            Err(e) => println!(
                                "cargo:warning=failed to copy asset {}: {}",
                                path.display(),
                                e
                            ),
                        }
                    }
                }
            }
        }

        match fs::read_dir(generated_src) {
            Ok(entries) => {
                for entry_res in entries {
                    let entry = match entry_res {
                        Ok(e) => e,
                        Err(e) => {
                            println!(
                                "cargo:warning=failed to read directory entry in generated_src: {}",
                                e
                            );
                            continue;
                        }
                    };
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "rs") {
                        if let (Some(fname), Some(stem)) = (
                            path.file_name().and_then(|n| n.to_str()),
                            path.file_stem().and_then(|s| s.to_str()),
                        ) {
                            let is_core = ["mod", "tokens", "state", "components"].contains(&stem);
                            let is_declared = declared_artboards
                                .as_ref()
                                .map(|names| names.iter().any(|name| name == stem))
                                .unwrap_or_else(|| is_valid_module_name(stem));
                            if !is_core && !is_declared {
                                continue;
                            }
                            let dest = generated_out.join(fname);
                            match fs::copy(&path, &dest) {
                                Ok(_) => {
                                    has_files = true;
                                    println!("cargo:rerun-if-changed={}", path.display());
                                }
                                Err(e) => println!(
                                    "cargo:warning=failed to copy {}: {}",
                                    path.display(),
                                    e
                                ),
                            }
                        }
                    }
                }
            }
            Err(e) => println!(
                "cargo:warning=failed to read generated source directory: {}",
                e
            ),
        }
    }

    // Ensure the generated module root exists so the crate can report missing exports clearly.
    let ensure_generated_root = |name: &str, content: &str| {
        let p = generated_out.join(name);
        if !p.exists() {
            if let Err(e) = fs::write(&p, content) {
                println!(
                    "cargo:warning=failed to write generated root {}: {}",
                    name, e
                );
            }
        }
    };
    ensure_generated_root("mod.rs", "// Auto-generated module declarations.\n// Artboard modules will be auto-discovered at build time.\n");

    // Find artboard files (anything that's not mod/tokens/state/components)
    let mut artboards = declared_artboards.unwrap_or_default();
    if artboards.is_empty() && has_files {
        match fs::read_dir(&generated_out) {
            Ok(entries) => {
                for entry_res in entries {
                    let entry = match entry_res {
                        Ok(e) => e,
                        Err(e) => {
                            println!(
                                "cargo:warning=failed to read directory entry for artboards: {}",
                                e
                            );
                            continue;
                        }
                    };
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "rs") {
                        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                            if is_valid_module_name(name)
                                && !["tokens", "state", "components"].contains(&name)
                            {
                                artboards.push(name.to_string());
                            }
                        }
                    }
                }
            }
            Err(e) => println!(
                "cargo:warning=failed to read generated output directory for artboards: {}",
                e
            ),
        }
    }

    let dispatch = generate_dispatch_code(&artboards);

    if let Err(e) = fs::write(out_dir.join("dispatch.rs"), dispatch) {
        println!("cargo:warning=failed to write dispatch.rs: {}", e);
    }

    println!("cargo:rerun-if-changed=generated");
}

#[cfg(test)]
mod build_tests {
    use super::*;

    #[test]
    fn test_generate_dispatch_code_empty() {
        let code = generate_dispatch_code(&[]);
        assert!(code.contains("pub fn artboard_names() -> &'static [&'static str] { &[] }"));
        assert!(
            code.contains("pub fn render_artboard(_name: &str, _ui: &mut Ui) -> bool { false }")
        );
    }

    #[test]
    fn test_generate_dispatch_code_with_artboards() {
        let code = generate_dispatch_code(&["login_screen".to_string(), "home".to_string()]);
        assert!(code.contains("\"login_screen\","));
        assert!(code.contains("\"home\","));
        assert!(code.contains("trait HandleAction { fn handle(self); }"));
        assert!(!code.contains("println!"));
        assert!(code.contains("let mut state = ui.data_mut(|d| d.remove_temp::<crate::generated::state::LoginScreenState>(id)).unwrap_or_default();"));
        assert!(code.contains(
            "crate::generated::login_screen::draw_login_screen(ui, &mut state).handle();"
        ));
        assert!(code.contains("ui.data_mut(|d| d.insert_temp(id, state));"));
    }
}
