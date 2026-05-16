use super::*;

pub(crate) fn generate_row_node(
    gap: f32,
    children: &[LayoutNode],
    bg: Option<Color32>,
    id: &str,
    indent: usize,
    token_map: Option<&HashMap<String, Color32>>,
) -> String {
    let indent_str = " ".repeat(indent);
    let mut output = String::new();
    if let Some(bg_color) = bg {
        output.push_str(&format!(
            "{}// Row: {}
{}hstack!(ui, gap: {:.1}, {{
",
            indent_str, id, indent_str, gap
        ));
        output.push_str(&format!(
            "{}let row_rect = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(
",
            indent_str
        ));
        let row_w: f32 = children.iter().map(get_node_width).sum();
        let row_h: f32 = children.iter().map(get_node_height).fold(0.0f32, f32::max);
        output.push_str(&format!(
            "{}{:.1}, {:.1})),
",
            indent_str, row_w, row_h
        ));
        output.push_str(&format!(
            "{});
{}painter.rect_filled(row_rect, 0.0, {});
",
            indent_str,
            indent_str,
            color_to_token_or_literal(&bg_color, token_map)
        ));
    } else {
        output.push_str(&format!(
            "{}// Row: {}
{}hstack!(ui, gap: {:.1}, {{
",
            indent_str, id, indent_str, gap
        ));
    }
    for child in children {
        output.push_str(&generate_node(child, indent + 4, token_map));
    }
    output.push_str(&format!(
        "{}}});
",
        indent_str
    ));
    output
}

pub(crate) fn generate_column_node(
    gap: f32,
    children: &[LayoutNode],
    bg: Option<Color32>,
    id: &str,
    indent: usize,
    token_map: Option<&HashMap<String, Color32>>,
) -> String {
    let indent_str = " ".repeat(indent);
    let mut output = String::new();
    if let Some(bg_color) = bg {
        output.push_str(&format!(
            "{}// Column: {}
{}vstack!(ui, gap: {:.1}, {{
{}",
            indent_str, id, indent_str, gap, indent_str
        ));
        output.push_str(&format!(
            "{}let col_rect = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(
",
            indent_str
        ));
        let col_w: f32 = children.iter().map(get_node_width).fold(0.0f32, f32::max);
        let col_h: f32 = children.iter().map(get_node_height).sum();
        output.push_str(&format!(
            "{}{:.1}, {:.1})),
",
            indent_str, col_w, col_h
        ));
        output.push_str(&format!(
            "{});
{}painter.rect_filled(col_rect, 0.0, {});
",
            indent_str,
            indent_str,
            color_to_token_or_literal(&bg_color, token_map)
        ));
    } else {
        output.push_str(&format!(
            "{}// Column: {}
{}vstack!(ui, gap: {:.1}, {{
",
            indent_str, id, indent_str, gap
        ));
    }
    for child in children {
        output.push_str(&generate_node(child, indent + 4, token_map));
    }
    output.push_str(&format!(
        "{}}});
",
        indent_str
    ));
    output
}
