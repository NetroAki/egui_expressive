use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let generated_src = Path::new("generated");
    let generated_out = out_dir.join("generated");

    fs::create_dir_all(&generated_out).unwrap();

    // Clear stale files from previous builds
    if let Ok(entries) = fs::read_dir(&generated_out) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "rs") {
                fs::remove_file(&path).unwrap_or(());
            }
        }
    }

    // Copy all .rs files from generated/ to OUT_DIR/generated/
    let mut has_files = false;
    if generated_src.exists() {
        for entry in fs::read_dir(generated_src).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "rs") {
                let dest = generated_out.join(path.file_name().unwrap());
                fs::copy(&path, &dest).unwrap();
                has_files = true;
            }
        }
    }

    // Ensure minimal mod.rs exists so the crate always compiles
    let mod_rs = generated_out.join("mod.rs");
    if !mod_rs.exists() {
        fs::write(&mod_rs, include_str!("generated/mod.rs")).unwrap();
    }

    // Ensure tokens.rs exists
    let tokens_rs = generated_out.join("tokens.rs");
    if !tokens_rs.exists() {
        fs::write(&tokens_rs, include_str!("generated/tokens.rs")).unwrap();
    }

    // Ensure state.rs exists
    let state_rs = generated_out.join("state.rs");
    if !state_rs.exists() {
        fs::write(&state_rs, include_str!("generated/state.rs")).unwrap();
    }

    // Ensure components.rs exists
    let components_rs = generated_out.join("components.rs");
    if !components_rs.exists() {
        fs::write(&components_rs, include_str!("generated/components.rs")).unwrap();
    }

    // Find artboard files (anything that's not mod/tokens/state/components)
    let mut artboards = Vec::new();
    if has_files {
        for entry in fs::read_dir(&generated_out).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "rs") {
                let name = path.file_stem().unwrap().to_str().unwrap();
                if !["mod", "tokens", "state", "components"].contains(&name) {
                    artboards.push(name.to_string());
                }
            }
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

    fs::write(out_dir.join("dispatch.rs"), dispatch).unwrap();

    println!("cargo:rerun-if-changed=generated");
}

fn snake_to_pascal(s: &str) -> String {
    s.split('_')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}
