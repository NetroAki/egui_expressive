use std::fs;
use std::path::{Path, PathBuf};

include!("src/shared.rs");

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR set by Cargo"));
    let generated_src = Path::new("generated");
    let generated_out = out_dir.join("generated");

    fs::create_dir_all(&generated_out).expect("create OUT_DIR/generated");

    // Clear stale files from previous builds
    match fs::read_dir(&generated_out) {
        Ok(entries) => {
            for entry_res in entries {
                let entry = match entry_res {
                    Ok(e) => e,
                    Err(e) => {
                        println!("cargo:warning=failed to read directory entry in generated_out: {}", e);
                        continue;
                    }
                };
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "rs") {
                    if let Err(e) = fs::remove_file(&path) {
                        println!("cargo:warning=failed to remove stale file {}: {}", path.display(), e);
                    }
                }
            }
        }
        Err(e) => println!("cargo:warning=failed to read generated_out directory: {}", e),
    }

    // Copy all .rs files from generated/ to OUT_DIR/generated/
    let mut has_files = false;
    if generated_src.exists() {
        match fs::read_dir(generated_src) {
            Ok(entries) => {
                for entry_res in entries {
                    let entry = match entry_res {
                        Ok(e) => e,
                        Err(e) => {
                            println!("cargo:warning=failed to read directory entry in generated_src: {}", e);
                            continue;
                        }
                    };
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "rs") {
                        if let Some(fname) = path.file_name().and_then(|n| n.to_str()) {
                            let dest = generated_out.join(fname);
                            match fs::copy(&path, &dest) {
                                Ok(_) => has_files = true,
                                Err(e) => println!("cargo:warning=failed to copy {}: {}", path.display(), e),
                            }
                        }
                    }
                }
            }
            Err(e) => println!("cargo:warning=failed to read generated source directory: {}", e),
        }
    }

    // Ensure minimal placeholder files exist so the crate always compiles
    let ensure_placeholder = |name: &str, content: &str| {
        let p = generated_out.join(name);
        if !p.exists() {
            if let Err(e) = fs::write(&p, content) {
                println!("cargo:warning=failed to write placeholder {}: {}", name, e);
            }
        }
    };
    ensure_placeholder("mod.rs", "// Auto-generated module declarations.\n// Artboard modules will be auto-discovered at build time.\n\npub mod tokens;\npub mod state;\npub mod components;\n");
    ensure_placeholder("tokens.rs", "");
    ensure_placeholder("state.rs", "");
    ensure_placeholder("components.rs", "");

    // Find artboard files (anything that's not mod/tokens/state/components)
    let mut artboards = Vec::new();
    if has_files {
        match fs::read_dir(&generated_out) {
            Ok(entries) => {
                for entry_res in entries {
                    let entry = match entry_res {
                        Ok(e) => e,
                        Err(e) => {
                            println!("cargo:warning=failed to read directory entry for artboards: {}", e);
                            continue;
                        }
                    };
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "rs") {
                        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                            if is_valid_module_name(name) && !["tokens", "state", "components"].contains(&name) {
                                artboards.push(name.to_string());
                            }
                        }
                    }
                }
            }
            Err(e) => println!("cargo:warning=failed to read generated output directory for artboards: {}", e),
        }
    }

    // Generate dispatch.rs
    let mut dispatch = String::new();
    dispatch.push_str("use egui::Ui;\n\n");

    if artboards.is_empty() {
        dispatch.push_str("pub fn artboard_names() -> &'static [&'static str] { &[] }\n");
        dispatch.push_str("pub fn render_artboard(_name: &str, _ui: &mut Ui) -> bool { false }\n");
    } else {
        dispatch.push_str("pub fn artboard_names() -> &'static [&'static str] {\n");
        dispatch.push_str("    &[\n");
        for name in &artboards {
            dispatch.push_str(&format!("        \"{}\",\n", name));
        }
        dispatch.push_str("    ]\n");
        dispatch.push_str("}\n\n");

        dispatch.push_str("pub fn render_artboard(name: &str, ui: &mut Ui) -> bool {\n");
        dispatch.push_str("    match name {\n");
        for name in &artboards {
            let pascal = snake_to_pascal(name);
            dispatch.push_str(&format!("        \"{}\" => {{\n", name));
            dispatch.push_str(&format!(
                "            let mut state = crate::generated::state::{}State::default();\n",
                pascal
            ));
            dispatch.push_str(&format!(
                "            crate::generated::{}::draw_{}(ui, &mut state);\n",
                name, name
            ));
            dispatch.push_str("            true\n");
            dispatch.push_str("        }\n");
        }
        dispatch.push_str("        _ => false,\n");
        dispatch.push_str("    }\n");
        dispatch.push_str("}\n");
    }

    if let Err(e) = fs::write(out_dir.join("dispatch.rs"), dispatch) {
        println!("cargo:warning=failed to write dispatch.rs: {}", e);
    }

    println!("cargo:rerun-if-changed=generated");
}
