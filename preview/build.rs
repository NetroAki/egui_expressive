use std::fs;
use std::path::{Path, PathBuf};

const RUST_KEYWORDS: [&str; 52] = [
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
    "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut",
    "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true",
    "type", "unsafe", "use", "where", "while", "async", "await", "dyn",     "abstract", "become",
    "box", "do", "final", "macro", "override", "priv", "typeof", "union", "unsized", "virtual", "yield",
    "try",
];

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR set by Cargo"));
    let generated_src = Path::new("generated");
    let generated_out = out_dir.join("generated");

    fs::create_dir_all(&generated_out).expect("create OUT_DIR/generated");

    // Clear stale files from previous builds
    if let Ok(entries) = fs::read_dir(&generated_out) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "rs") {
                if let Err(e) = fs::remove_file(&path) {
                    eprintln!("cargo:warning=failed to remove stale file {}: {}", path.display(), e);
                }
            }
        }
    }

    // Copy all .rs files from generated/ to OUT_DIR/generated/
    let mut has_files = false;
    if generated_src.exists() {
        if let Ok(entries) = fs::read_dir(generated_src) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "rs") {
                    if let Some(fname) = path.file_name().and_then(|n| n.to_str()) {
                        let dest = generated_out.join(fname);
                        match fs::copy(&path, &dest) {
                            Ok(_) => has_files = true,
                            Err(e) => eprintln!("cargo:warning=failed to copy {}: {}", path.display(), e),
                        }
                    }
                }
            }
        }
    }

    // Ensure minimal placeholder files exist so the crate always compiles
    let ensure_placeholder = |name: &str, content: &str| {
        let p = generated_out.join(name);
        if !p.exists() {
            if let Err(e) = fs::write(&p, content) {
                eprintln!("cargo:warning=failed to write placeholder {}: {}", name, e);
            }
        }
    };
    ensure_placeholder("mod.rs", include_str!("generated/mod.rs"));
    ensure_placeholder("tokens.rs", include_str!("generated/tokens.rs"));
    ensure_placeholder("state.rs", include_str!("generated/state.rs"));
    ensure_placeholder("components.rs", include_str!("generated/components.rs"));

    // Find artboard files (anything that's not mod/tokens/state/components)
    let mut artboards = Vec::new();
    if has_files {
        if let Ok(entries) = fs::read_dir(&generated_out) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "rs") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        if is_valid_module_name(name) && !["mod", "tokens", "state", "components"].contains(&name) {
                            artboards.push(name.to_string());
                        }
                    }
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

    if let Err(e) = fs::write(out_dir.join("dispatch.rs"), dispatch) {
        eprintln!("cargo:warning=failed to write dispatch.rs: {}", e);
    }

    println!("cargo:rerun-if-changed=generated");
}

fn is_valid_module_name(s: &str) -> bool {
    if s.is_empty() || s.starts_with(|c: char| c.is_ascii_digit()) {
        return false;
    }
    if !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return false;
    }
    !RUST_KEYWORDS.contains(&s)
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
