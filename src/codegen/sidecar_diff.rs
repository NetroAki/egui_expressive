use super::*;

/// Represents a change between two sidecar JSON files.
#[derive(Clone, Debug)]
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
