use super::*;

fn color_to_token_or_literal(
    color: &Color32,
    token_map: Option<&HashMap<String, Color32>>,
) -> String {
    if let Some(map) = token_map {
        // Look up the color in the token map — sort keys for deterministic output
        let mut entries: Vec<(&String, &Color32)> = map.iter().collect();
        entries.sort_by_key(|(name, _)| name.as_str());
        for (name, c) in entries {
            if *c == *color {
                return format!("tokens::{}", name.to_uppercase());
            }
        }
    }
    // Fall back to literal — use to_srgba_unmultiplied() to get straight-alpha bytes
    // (Color32 stores premultiplied; feeding .r()/.g()/.b() to from_rgba_unmultiplied would double-premultiply)
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    if a < 255 {
        format!(
            "egui::Color32::from_rgba_unmultiplied({}, {}, {}, {})",
            r, g, b, a
        )
    } else {
        format!("egui::Color32::from_rgb({}, {}, {})", r, g, b)
    }
}

fn get_node_width(node: &LayoutNode) -> f32 {
    match node {
        LayoutNode::Shape { w, .. } => *w,
        LayoutNode::Image { w, .. } => *w,
        LayoutNode::Card { children, .. } => {
            children.iter().map(get_node_width).fold(0.0f32, f32::max)
        }
        LayoutNode::Row { children, .. } => children.iter().map(get_node_width).sum(),
        LayoutNode::Column { children, .. } => {
            children.iter().map(get_node_width).fold(0.0f32, f32::max)
        }
        LayoutNode::Panel { children, .. } => {
            children.iter().map(get_node_width).fold(0.0f32, f32::max)
        }
        LayoutNode::ScrollArea { children, .. } => {
            children.iter().map(get_node_width).fold(0.0f32, f32::max)
        }
        LayoutNode::Spacer { size, .. } => *size,
        LayoutNode::RichScene(scene_node) => scene_node.geometry.bounds().width(),
        _ => 100.0,
    }
}

fn get_node_height(node: &LayoutNode) -> f32 {
    match node {
        LayoutNode::Shape { h, .. } => *h,
        LayoutNode::Image { h, .. } => *h,
        LayoutNode::Card { children, .. } => children.iter().map(get_node_height).sum(),
        LayoutNode::Row { children, .. } => {
            children.iter().map(get_node_height).fold(0.0f32, f32::max)
        }
        LayoutNode::Column { children, .. } => children.iter().map(get_node_height).sum(),
        LayoutNode::Panel { children, .. } => {
            children.iter().map(get_node_height).fold(0.0f32, f32::max)
        }
        LayoutNode::ScrollArea { children, .. } => {
            children.iter().map(get_node_height).fold(0.0f32, f32::max)
        }
        LayoutNode::Spacer { size, .. } => *size,
        LayoutNode::RichScene(scene_node) => scene_node.geometry.bounds().height(),
        _ => 24.0,
    }
}

fn calculate_panel_dimensions(children: &[LayoutNode], side: PanelSide) -> (f32, f32) {
    let w = children.iter().map(get_node_width).fold(0.0f32, f32::max);
    let h = children.iter().map(get_node_height).fold(0.0f32, f32::max);

    match side {
        PanelSide::Left | PanelSide::Right => (w.max(200.0), 800.0),
        PanelSide::Top | PanelSide::Bottom => (375.0, h.max(100.0)),
        PanelSide::Center => (w.max(300.0), h.max(200.0)),
    }
}

fn calculate_card_dimensions(children: &[LayoutNode]) -> (f32, f32) {
    let w = children
        .iter()
        .map(get_node_width)
        .fold(0.0f32, f32::max)
        .max(100.0);
    let h = children.iter().map(get_node_height).sum::<f32>().max(60.0);
    (w + 16.0, h + 16.0) // Add padding
}
