use super::*;

/// Represents a change between two sidecar JSON files.
pub enum SidecarChange {
    /// A new element was added.
    Added(Box<LayoutElement>),
    /// An element was removed.
    Removed(String),
    /// An element moved positions.
    Moved {
        id: String,
        old_pos: (f32, f32),
        new_pos: (f32, f32),
    },
    /// An element was resized.
    Resized {
        id: String,
        old_size: (f32, f32),
        new_size: (f32, f32),
    },
    /// An element's color changed.
    ColorChanged {
        id: String,
        old: Color32,
        new: Color32,
    },
    /// An element's text content changed.
    TextChanged {
        id: String,
        old: String,
        new: String,
    },
}

/// Diff two sidecar JSON strings and return a list of changes.
pub fn diff_sidecars(old_json: &str, new_json: &str) -> Vec<SidecarChange> {
    let old_els = parse_json_sidecar(old_json)
        .map(|(_, els)| els)
        .unwrap_or_default();
    let new_els = parse_json_sidecar(new_json)
        .map(|(_, els)| els)
        .unwrap_or_default();

    let old_map: std::collections::HashMap<String, &LayoutElement> =
        old_els.iter().map(|e| (e.id.clone(), e)).collect();
    let new_map: std::collections::HashMap<String, &LayoutElement> =
        new_els.iter().map(|e| (e.id.clone(), e)).collect();

    let mut changes = Vec::new();

    // Added
    for (id, el) in &new_map {
        if !old_map.contains_key(id) {
            changes.push(SidecarChange::Added(Box::new((*el).clone())));
        }
    }

    // Removed
    for id in old_map.keys() {
        if !new_map.contains_key(id) {
            changes.push(SidecarChange::Removed(id.clone()));
        }
    }

    // Changed
    for (id, new_el) in &new_map {
        if let Some(old_el) = old_map.get(id) {
            if (old_el.x - new_el.x).abs() > 0.5 || (old_el.y - new_el.y).abs() > 0.5 {
                changes.push(SidecarChange::Moved {
                    id: id.clone(),
                    old_pos: (old_el.x, old_el.y),
                    new_pos: (new_el.x, new_el.y),
                });
            }
            if (old_el.w - new_el.w).abs() > 0.5 || (old_el.h - new_el.h).abs() > 0.5 {
                changes.push(SidecarChange::Resized {
                    id: id.clone(),
                    old_size: (old_el.w, old_el.h),
                    new_size: (new_el.w, new_el.h),
                });
            }
            if old_el.fill != new_el.fill {
                changes.push(SidecarChange::ColorChanged {
                    id: id.clone(),
                    old: old_el.fill.unwrap_or(Color32::BLACK),
                    new: new_el.fill.unwrap_or(Color32::BLACK),
                });
            }
            if old_el.text != new_el.text {
                changes.push(SidecarChange::TextChanged {
                    id: id.clone(),
                    old: old_el.text.clone().unwrap_or_default(),
                    new: new_el.text.clone().unwrap_or_default(),
                });
            }
        }
    }

    changes
}

// ============================================================================
// Public API Entry Point
// ============================================================================

/// Full pipeline: SVG string → Rust scaffold code.
/// This is the main entry point for the Illustrator export workflow.
pub fn svg_to_rust_scaffold(svg: &str, fn_name: &str, options: &InferenceOptions) -> String {
    // Parse SVG into elements
    let elements = parse_svg_elements(svg);

    // Infer layout
    let nodes = infer_layout(&elements, options);

    // Generate Rust code
    let artboard_w = elements.iter().map(|e| e.x + e.w).fold(375.0f32, f32::max);

    let artboard_h = elements.iter().map(|e| e.y + e.h).fold(812.0f32, f32::max);

    let bg_color = elements
        .iter()
        .find(|e| e.id.to_lowercase().contains("background") || e.id.to_lowercase().contains("bg"))
        .and_then(|e| e.fill);

    generate_rust(
        fn_name, artboard_w, artboard_h, &nodes, bg_color, None, None,
    )
}

/// Generate a complete Rust source file for a single artboard.
///
/// Returns a string containing a valid `.rs` file with a `pub fn draw_<name>(ui: &mut egui::Ui, state: &mut <Name>State) -> Option<<Name>Action>`
/// function that renders all elements belonging to this artboard.
pub fn generate_artboard_file(
    artboard_name: &str,
    artboard_w: f32,
    artboard_h: f32,
    elements: &[LayoutElement],
    token_map: &HashMap<String, Color32>,
) -> String {
    let fn_name = sanitize_fn_name(artboard_name);
    let fn_name = if fn_name.is_empty() {
        "artboard".to_string()
    } else {
        fn_name
    };
    let state_struct_name = format!("{}State", to_pascal_case(artboard_name));
    let options = InferenceOptions::default();
    let layout = infer_layout(elements, &options);
    // generate_rust already produces a complete file (imports + pub fn draw_X)
    generate_rust(
        &fn_name,
        artboard_w,
        artboard_h,
        &layout,
        None,
        Some(&state_struct_name),
        Some(token_map),
    )
}

/// Artboard tuple adapter used by [`generate_all_artboards`].
pub trait ArtboardDef {
    fn artboard_name(&self) -> &str;
    fn artboard_x(&self) -> f32;
    fn artboard_y(&self) -> f32;
    fn artboard_w(&self) -> f32;
    fn artboard_h(&self) -> f32;
}

impl ArtboardDef for (&str, f32, f32) {
    fn artboard_name(&self) -> &str {
        self.0
    }
    fn artboard_x(&self) -> f32 {
        0.0
    }
    fn artboard_y(&self) -> f32 {
        0.0
    }
    fn artboard_w(&self) -> f32 {
        self.1
    }
    fn artboard_h(&self) -> f32 {
        self.2
    }
}

impl ArtboardDef for (&str, f32, f32, f32, f32) {
    fn artboard_name(&self) -> &str {
        self.0
    }
    fn artboard_x(&self) -> f32 {
        self.1
    }
    fn artboard_y(&self) -> f32 {
        self.2
    }
    fn artboard_w(&self) -> f32 {
        self.3
    }
    fn artboard_h(&self) -> f32 {
        self.4
    }
}

pub(crate) fn element_intersects_artboard(
    element: &LayoutElement,
    artboard: &impl ArtboardDef,
) -> bool {
    let ax0 = artboard.artboard_x();
    let ay0 = artboard.artboard_y();
    let ax1 = ax0 + artboard.artboard_w();
    let ay1 = ay0 + artboard.artboard_h();
    let ex0 = element.x;
    let ey0 = element.y;
    let ex1 = element.x + element.w;
    let ey1 = element.y + element.h;
    ex0 < ax1 && ex1 > ax0 && ey0 < ay1 && ey1 > ay0
}

/// Generate one Rust file per artboard.
///
/// Returns a `Vec<(filename, file_content)>` — one entry per artboard.
/// Elements are assigned to artboards by their `artboard_name` field if set.
/// Elements with no `artboard_name` are assigned by bounding-box intersection,
/// with a first-artboard fallback only for elements that intersect no artboard.
pub fn generate_all_artboards<T: ArtboardDef>(
    all_elements: &[LayoutElement],
    artboards: &[T],
    token_map: &HashMap<String, Color32>,
) -> Vec<(String, String)> {
    artboards
        .iter()
        .enumerate()
        .map(|(artboard_idx, artboard)| {
            let name = artboard.artboard_name();
            let w = artboard.artboard_w();
            let h = artboard.artboard_h();
            let sanitized = {
                let s = sanitize_fn_name(name);
                if s.is_empty() {
                    "artboard".to_string()
                } else {
                    s
                }
            };
            let filename = format!("{}.rs", sanitized);
            let artboard_elements: Vec<LayoutElement> = all_elements
                .iter()
                .filter(|e| {
                    e.artboard_name.as_deref() == Some(name)
                        || (e.artboard_name.is_none() && element_intersects_artboard(e, artboard))
                        || (e.artboard_name.is_none()
                            && artboard_idx == 0
                            && !artboards.iter().any(|a| element_intersects_artboard(e, a)))
                })
                .cloned()
                .collect();
            let code = generate_artboard_file(name, w, h, &artboard_elements, token_map);
            (filename, code)
        })
        .collect()
}
