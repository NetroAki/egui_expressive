use super::*;

/// Artboard state definition for code generation.
pub struct ArtboardState {
    pub name: String,
    pub text_fields: Vec<String>,
    pub button_labels: Vec<String>,
}

/// Component definition for code generation.
#[derive(Clone, Debug)]
pub struct ComponentDef {
    pub name: String,
    pub fill: Color32,
    pub rounding: f32,
    pub text_size: f32,
    pub text_color: Color32,
}

/// Artboard output containing all data needed for code generation.
#[derive(Clone, Debug)]
pub struct ArtboardOutput {
    pub name: String,
    pub nodes: Vec<LayoutNode>,
    pub bg_color: Option<Color32>,
    pub artboard_w: f32,
    pub artboard_h: f32,
    pub text_fields: Vec<String>,
    pub button_labels: Vec<String>,
}

/// Multi-file output structure containing all generated files.
#[derive(Clone, Debug)]
pub struct MultiFileOutput {
    pub mod_rs: String,
    pub tokens_rs: String,
    pub state_rs: String,
    pub components_rs: String,
    pub artboard_files: Vec<(String, String)>,
}

/// Generate a tokens.rs file from a color map.
pub fn generate_tokens_file(color_map: &HashMap<String, Color32>, spacing: &[f32]) -> String {
    let mut output = String::new();

    output.push_str("// Auto-generated design tokens\n");
    output.push_str("use egui::Color32;\n\n");

    // Generate color tokens
    let mut color_tokens: Vec<_> = color_map.iter().collect();
    color_tokens.sort_by(|a, b| a.0.cmp(b.0));

    for (name, color) in color_tokens {
        let token_name = name.to_uppercase();
        let [r, g, b, a] = color.to_srgba_unmultiplied();
        if a < 255 {
            output.push_str(&format!(
                "pub const {}: Color32 = Color32::from_rgba_unmultiplied({}, {}, {}, {});\n",
                token_name, r, g, b, a
            ));
        } else {
            output.push_str(&format!(
                "pub const {}: Color32 = Color32::from_rgb({}, {}, {});\n",
                token_name, r, g, b
            ));
        }
    }

    // Add default tokens if not present
    if !color_map.contains_key("surface") {
        output.push_str("\npub const SURFACE: Color32 = Color32::from_rgb(28, 27, 31);\n");
    }
    if !color_map.contains_key("on_surface") {
        output.push_str("pub const ON_SURFACE: Color32 = Color32::from_rgb(228, 226, 230);\n");
    }
    if !color_map.contains_key("primary") {
        output.push_str("pub const PRIMARY: Color32 = Color32::from_rgb(103, 80, 164);\n");
    }
    if !color_map.contains_key("on_primary") {
        output.push_str("pub const ON_PRIMARY: Color32 = Color32::from_rgb(255, 255, 255);\n");
    }
    if !color_map.contains_key("secondary") {
        output.push_str("pub const SECONDARY: Color32 = Color32::from_rgb(69, 69, 69);\n");
    }
    if !color_map.contains_key("on_secondary") {
        output.push_str("pub const ON_SECONDARY: Color32 = Color32::from_rgb(255, 255, 255);\n");
    }

    // Generate spacing tokens
    output.push('\n');
    let spacing_tokens = [
        ("SPACING_SM", 8.0),
        ("SPACING_MD", 16.0),
        ("SPACING_LG", 24.0),
        ("SPACING_XL", 32.0),
    ];
    for (name, value) in spacing_tokens {
        output.push_str(&format!("pub const {}: f32 = {:.1};\n", name, value));
    }

    // Add custom spacing from the spacing array
    for (i, &sp) in spacing.iter().enumerate() {
        output.push_str(&format!("pub const SPACING_{}: f32 = {:.1};\n", i, sp));
    }

    output
}

/// Generate a state.rs file from artboard states.
pub fn generate_state_file(artboards: &[ArtboardState]) -> String {
    let mut output = String::new();

    output.push_str("// Auto-generated state\n\n");

    for artboard in artboards {
        let struct_name = to_pascal_case(&artboard.name);

        // Generate struct with text fields
        output.push_str(&format!(
            "#[derive(Default, Clone)]\npub struct {}State {{\n",
            struct_name
        ));
        for field in &artboard.text_fields {
            let field_name = sanitize_field_name(field);
            output.push_str(&format!("    pub {}: String,\n", field_name));
        }
        output.push_str("}\n\n");

        // Generate Action enum
        output.push_str(&format!("pub enum {}Action {{\n", struct_name));
        for label in &artboard.button_labels {
            let action_name = to_pascal_case(label);
            output.push_str(&format!("    {},\n", action_name));
        }
        output.push_str("}\n\n");
    }

    output
}

/// Generate a mod.rs file listing all artboard modules.
pub fn generate_mod_file(artboard_names: &[&str]) -> String {
    let mut output = String::new();

    output.push_str("// Auto-generated module declarations\n");
    output.push_str("pub mod tokens;\n");
    output.push_str("pub mod state;\n");
    output.push_str("pub mod components;\n");

    for name in artboard_names {
        let safe_name = sanitize_module_name(name);
        output.push_str(&format!("pub mod {};\n", safe_name));
    }

    output
}

/// Generate a components.rs file with reusable component functions.
pub fn generate_components_file(components: &[ComponentDef]) -> String {
    let mut output = String::new();

    let _ = components;
    output.push_str("// Auto-generated component hook.\n");
    output.push_str("// Local wrapper primitives are intentionally not emitted here.\n");
    output.push_str(
        "// Reusable design primitives live in egui_expressive (scene, typography, image slots).\n",
    );

    output
}

/// Generate all files for multiple artboards.
pub fn generate_multi_file_output(artboards: &[ArtboardOutput]) -> MultiFileOutput {
    let mut artboard_files = Vec::new();

    // Collect all unique colors and spacing from artboards
    let mut all_colors: HashMap<String, Color32> = HashMap::new();
    let mut all_spacing: Vec<f32> = vec![8.0, 16.0, 24.0, 32.0];

    // Collect text fields and button labels per artboard
    let artboard_states: Vec<ArtboardState> = artboards
        .iter()
        .map(|a| ArtboardState {
            name: a.name.clone(),
            text_fields: a.text_fields.clone(),
            button_labels: a.button_labels.clone(),
        })
        .collect();

    // Collect colors from artboards
    for artboard in artboards {
        if let Some(bg) = artboard.bg_color {
            let name = format!("{}_bg", artboard.name);
            all_colors.insert(name, bg);
        }

        // Extract colors from nodes
        collect_colors_from_nodes(&artboard.nodes, &mut all_colors);

        // Add spacing from nodes
        collect_spacing_from_nodes(&artboard.nodes, &mut all_spacing);
    }

    // Generate artboard files
    let artboard_names: Vec<&str> = artboards.iter().map(|a| a.name.as_str()).collect();

    for artboard in artboards {
        let state_struct_name = format!("{}State", to_pascal_case(&artboard.name));
        let token_map: HashMap<String, Color32> = all_colors.clone();

        let content = generate_rust(
            &artboard.name,
            artboard.artboard_w,
            artboard.artboard_h,
            &artboard.nodes,
            artboard.bg_color,
            Some(&state_struct_name),
            Some(&token_map),
        );

        let filename = format!("{}.rs", sanitize_module_name(&artboard.name));
        artboard_files.push((filename, content));
    }

    // Generate common tokens
    let tokens_rs = generate_tokens_file(&all_colors, &all_spacing);

    // Generate state file
    let state_rs = generate_state_file(&artboard_states);

    // Generate components file (empty for now, can be extended)
    let components = vec![];
    let components_rs = generate_components_file(&components);

    // Generate mod.rs
    let mod_rs = generate_mod_file(&artboard_names);

    MultiFileOutput {
        mod_rs,
        tokens_rs,
        state_rs,
        components_rs,
        artboard_files,
    }
}

/// Collect colors from layout nodes into the color map.
pub(crate) fn collect_colors_from_nodes(
    nodes: &[LayoutNode],
    color_map: &mut HashMap<String, Color32>,
) {
    for node in nodes {
        match node {
            LayoutNode::Shape { fill, id, .. } => {
                let name = id.to_string();
                color_map.entry(name).or_insert(*fill);
            }
            LayoutNode::Card { bg, id, .. } => {
                let name = format!("{}_bg", id);
                color_map.entry(name).or_insert(*bg);
            }
            LayoutNode::Row { children, .. } => {
                collect_colors_from_nodes(children, color_map);
            }
            LayoutNode::Column { children, .. } => {
                collect_colors_from_nodes(children, color_map);
            }
            LayoutNode::ScrollArea { children, .. } => {
                collect_colors_from_nodes(children, color_map);
            }
            LayoutNode::Panel { children, .. } => {
                collect_colors_from_nodes(children, color_map);
            }
            _ => {}
        }
    }
}

/// Collect spacing values from layout nodes.
pub(crate) fn collect_spacing_from_nodes(nodes: &[LayoutNode], spacing: &mut Vec<f32>) {
    for node in nodes {
        match node {
            LayoutNode::Row { gap, children, .. } => {
                if !spacing.contains(gap) {
                    spacing.push(*gap);
                }
                collect_spacing_from_nodes(children, spacing);
            }
            LayoutNode::Column { gap, children, .. } => {
                if !spacing.contains(gap) {
                    spacing.push(*gap);
                }
                collect_spacing_from_nodes(children, spacing);
            }
            _ => {}
        }
    }
}

/// Convert a string to PascalCase.
pub(crate) fn to_pascal_case(s: &str) -> String {
    // Strip non-ASCII and non-alphanumeric chars (except separators)
    let cleaned: String = s
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ' ' {
                c
            } else {
                ' '
            }
        })
        .collect();
    let result: String = cleaned
        .split(|c: char| c == '_' || c == '-' || c.is_whitespace())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect();
    // Handle empty result
    let result = if result.is_empty() {
        "Component".to_string()
    } else {
        result
    };
    // Handle leading digit
    if result.starts_with(|c: char| c.is_ascii_digit()) {
        format!("S{}", result)
    } else {
        result
    }
}

/// Sanitize a field name for use in Rust code.
pub(crate) fn sanitize_field_name(name: &str) -> String {
    const RUST_KEYWORDS: &[&str] = &[
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
        "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
        "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe",
        "use", "where", "while", "async", "await", "dyn", "abstract", "become", "box", "do",
        "final", "macro", "override", "priv", "typeof", "unsized", "virtual", "yield",
    ];
    let sanitized: String = name
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    // Remove leading/trailing underscores, collapse multiple underscores
    let sanitized = sanitized.trim_matches('_').to_string();
    let sanitized = {
        let mut s = String::new();
        let mut prev_underscore = false;
        for c in sanitized.chars() {
            if c == '_' {
                if !prev_underscore {
                    s.push(c);
                }
                prev_underscore = true;
            } else {
                s.push(c);
                prev_underscore = false;
            }
        }
        s
    };
    // Handle empty result
    let sanitized = if sanitized.is_empty() {
        "field".to_string()
    } else {
        sanitized
    };
    // Handle leading digit
    let sanitized = if sanitized.starts_with(|c: char| c.is_ascii_digit()) {
        format!("f_{}", sanitized)
    } else {
        sanitized
    };
    // Handle Rust keywords
    if RUST_KEYWORDS.contains(&sanitized.as_str()) {
        format!("{}_", sanitized)
    } else {
        sanitized
    }
}
