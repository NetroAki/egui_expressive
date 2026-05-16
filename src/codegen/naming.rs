use super::*;

pub fn parse_naming(name: &str) -> NamingHint {
    let lower = name.to_lowercase();

    // Check for gap-N first (might be standalone)
    if let Some(gap_idx) = lower.find("gap-") {
        let after = &lower[gap_idx + 4..];
        if let Some(end_idx) = after.find(char::is_whitespace).or(Some(after.len())) {
            let num_str = &after[..end_idx];
            if let Ok(gap) = num_str.parse::<f32>() {
                return NamingHint::Gap(gap);
            }
        }
        // Also try just parsing the first number
        let mut num_str = String::new();
        for ch in after.chars() {
            if ch.is_ascii_digit() || ch == '.' {
                num_str.push(ch);
            } else {
                break;
            }
        }
        if let Ok(gap) = num_str.parse::<f32>() {
            return NamingHint::Gap(gap);
        }
    }

    // Row patterns: row-*, hstack-*
    if lower.starts_with("row-") || lower.starts_with("hstack-") {
        let label = extract_label(name);
        return NamingHint::Row(label);
    }

    // Column patterns: col-*, vstack-*
    if lower.starts_with("col-") || lower.starts_with("vstack-") {
        let label = extract_label(name);
        return NamingHint::Column(label);
    }

    // Panel patterns: panel-left, panel-right, panel-top, panel-bottom, panel-center
    if lower.starts_with("panel-") {
        let side = if lower.contains("left") {
            PanelSide::Left
        } else if lower.contains("right") {
            PanelSide::Right
        } else if lower.contains("top") {
            PanelSide::Top
        } else if lower.contains("bottom") {
            PanelSide::Bottom
        } else {
            PanelSide::Center
        };
        return NamingHint::Panel(side);
    }

    // Grid pattern: grid-*
    if lower.starts_with("grid-") {
        let label = extract_label(name);
        return NamingHint::Grid(label);
    }

    // Scroll pattern: scroll-*
    if lower.starts_with("scroll-") {
        let label = extract_label(name);
        return NamingHint::Scroll(label);
    }

    // Button patterns: btn-*, button-*
    if lower.starts_with("btn-") || lower.starts_with("button-") {
        let label = extract_label(name);
        return NamingHint::Button(label);
    }

    // TextEdit patterns: input-*, field-*, textedit-*
    if lower.starts_with("input-") || lower.starts_with("field-") || lower.starts_with("textedit-")
    {
        let label = extract_label(name);
        return NamingHint::TextEdit(label);
    }

    // Label patterns: label-*, text-*
    if lower.starts_with("label-") || lower.starts_with("text-") {
        let label = extract_label(name);
        return NamingHint::Label(label);
    }

    // Image patterns: img-*, image-*
    if lower.starts_with("img-") || lower.starts_with("image-") {
        let label = extract_label(name);
        return NamingHint::Image(label);
    }

    // Icon pattern: icon-*
    if lower.starts_with("icon-") {
        let label = extract_label(name);
        return NamingHint::Icon(label);
    }

    // Card pattern: card-*
    if lower.starts_with("card-") {
        let label = extract_label(name);
        return NamingHint::Card(label);
    }

    // Divider: divider, divider-*
    if lower.starts_with("divider") {
        return NamingHint::Divider;
    }

    // Spacer: spacer, spacer-*
    if lower.starts_with("spacer") {
        return NamingHint::Spacer;
    }

    // Badge pattern: badge-*
    if lower.starts_with("badge-") {
        let label = extract_label(name);
        return NamingHint::Badge(label);
    }

    // Chip pattern: chip-*
    if lower.starts_with("chip-") {
        let label = extract_label(name);
        return NamingHint::Chip(label);
    }

    // Toggle pattern: toggle-*, switch-*
    if lower.starts_with("toggle-") || lower.starts_with("switch-") {
        let label = extract_label(name);
        return NamingHint::Toggle(label);
    }

    // Slider pattern: slider-*, knob-*
    if lower.starts_with("slider-") || lower.starts_with("knob-") {
        let label = extract_label(name);
        return NamingHint::Slider(label);
    }

    NamingHint::None
}

pub(crate) fn extract_label(name: &str) -> String {
    // Extract the part after the prefix, trimmed
    let lower = name.to_lowercase();
    let prefixes = [
        "row-",
        "hstack-",
        "col-",
        "vstack-",
        "panel-",
        "grid-",
        "scroll-",
        "btn-",
        "button-",
        "input-",
        "field-",
        "textedit-",
        "label-",
        "text-",
        "img-",
        "image-",
        "icon-",
        "card-",
        "badge-",
        "chip-",
        "toggle-",
        "switch-",
        "slider-",
        "knob-",
    ];

    for prefix in &prefixes {
        if lower.starts_with(prefix) {
            let label = &name[prefix.len()..];
            return label.trim().to_string();
        }
    }

    name.trim().to_string()
}

#[derive(Clone, Debug, PartialEq)]
pub enum NamingHint {
    Row(String),
    Column(String),
    Panel(PanelSide),
    Grid(String),
    Scroll(String),
    Button(String),
    TextEdit(String),
    Label(String),
    Image(String),
    Icon(String),
    Card(String),
    Divider,
    Spacer,
    Badge(String),
    Chip(String),
    Toggle(String),
    Slider(String),
    Gap(f32),
    None,
}

// ============================================================================
// Gap Inference
// ============================================================================
