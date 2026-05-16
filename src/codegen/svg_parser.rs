use super::*;

pub(crate) fn sanitize_module_name(name: &str) -> String {
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
        "module".to_string()
    } else {
        sanitized
    };
    // Handle leading digit
    let sanitized = if sanitized.starts_with(|c: char| c.is_ascii_digit()) {
        format!("m_{}", sanitized)
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

// ============================================================================
// SVG-to-LayoutElement Parser
// ============================================================================

/// Parse an SVG string into a flat list of LayoutElements.
/// Uses simple string scanning (no XML parser dependency).
pub fn parse_svg_elements(svg: &str) -> Vec<LayoutElement> {
    let mut elements: Vec<LayoutElement> = Vec::new();

    // Find all groups
    let mut search_start = 0;
    while let Some(g_start) = svg[search_start..].find("<g") {
        let g_start = search_start + g_start;
        if let Some(g_tag_end) = svg[g_start..].find('>') {
            let g_tag_end = g_start + g_tag_end;
            let g_tag = &svg[g_start..g_tag_end + 1];

            // Extract group id
            let id =
                extract_attr(g_tag, "id").unwrap_or_else(|| format!("group_{}", elements.len()));

            // Check for transform attribute (might contain x, y)
            let (x, y) = extract_transform_xy(g_tag);

            // Find the group's direct children (rect, text, path, image)
            let group_content_start = g_tag_end + 1;
            if let Some(g_end) = find_matching_close(&svg[group_content_start..], "g") {
                let group_content = &svg[group_content_start..group_content_start + g_end];

                let children = parse_group_children(group_content);

                // If children exist, create a group element
                if !children.is_empty() {
                    // Calculate bounding box from children
                    let min_x = children.iter().map(|c| c.x).fold(f32::INFINITY, f32::min);
                    let min_y = children.iter().map(|c| c.y).fold(f32::INFINITY, f32::min);
                    let max_x = children.iter().map(|c| c.x + c.w).fold(0.0f32, f32::max);
                    let max_y = children.iter().map(|c| c.y + c.h).fold(0.0f32, f32::max);

                    elements.push(LayoutElement {
                        id,
                        el_type: ElementType::Group,
                        x: x.unwrap_or(min_x),
                        y: y.unwrap_or(min_y),
                        w: max_x - min_x,
                        h: max_y - min_y,
                        fill: extract_fill_from_tag(g_tag),
                        stroke: extract_stroke_from_tag(g_tag),
                        text: None,
                        text_size: None,
                        children,
                        opacity: 1.0,
                        rotation_deg: 0.0,
                        corner_radius: 0.0,
                        gradient: None,
                        blend_mode: BlendMode::Normal,
                        effects: vec![],
                        stroke_dash: None,
                        clip_children: false,
                        text_align: None,
                        letter_spacing: None,
                        line_height: None,
                        stroke_cap: None,
                        stroke_join: None,
                        stroke_miter_limit: None,
                        text_decoration: None,
                        text_transform: None,
                        text_runs: vec![],
                        symbol_name: None,
                        is_compound_path: false,
                        is_gradient_mesh: false,
                        is_chart: false,
                        is_opaque: false,
                        third_party_effects: vec![],
                        notes: vec![],
                        appearance_fills: vec![],
                        appearance_strokes: vec![],
                        appearance_stack: crate::scene::AppearanceStack::default(),
                        path_points: vec![],
                        path_closed: false,
                        artboard_name: None,
                        image_path: None,
                    });
                }
            }

            search_start = g_tag_end + 1;
        } else {
            search_start = g_start + 1;
        }
    }

    // Also look for top-level elements not in groups
    elements.extend(parse_top_level_elements(svg));

    elements
}

pub(crate) fn parse_group_children(content: &str) -> Vec<LayoutElement> {
    let mut elements = Vec::new();

    // Parse rects
    let mut rect_start = 0;
    while let Some(idx) = content[rect_start..].find("<rect") {
        let idx = rect_start + idx;
        if let Some(tag_end) = content[idx..].find('>') {
            let tag_end = idx + tag_end;
            let tag = &content[idx..tag_end + 1];

            let id = extract_attr(tag, "id").unwrap_or_else(|| format!("rect_{}", elements.len()));
            let x: f32 = extract_float_attr(tag, "x").unwrap_or(0.0);
            let y: f32 = extract_float_attr(tag, "y").unwrap_or(0.0);
            let w: f32 = extract_float_attr(tag, "width").unwrap_or(0.0);
            let h: f32 = extract_float_attr(tag, "height").unwrap_or(0.0);

            elements.push(LayoutElement {
                id,
                el_type: ElementType::Shape,
                x,
                y,
                w,
                h,
                fill: extract_fill_from_tag(tag),
                stroke: extract_stroke_from_tag(tag),
                text: None,
                text_size: None,
                children: vec![],
                opacity: 1.0,
                rotation_deg: 0.0,
                corner_radius: 0.0,
                gradient: None,
                blend_mode: BlendMode::Normal,
                effects: vec![],
                stroke_dash: None,
                clip_children: false,
                text_align: None,
                letter_spacing: None,
                line_height: None,
                stroke_cap: None,
                stroke_join: None,
                stroke_miter_limit: None,
                text_decoration: None,
                text_transform: None,
                text_runs: vec![],
                symbol_name: None,
                is_compound_path: false,
                is_gradient_mesh: false,
                is_chart: false,
                is_opaque: false,
                third_party_effects: vec![],
                notes: vec![],
                appearance_fills: vec![],
                appearance_strokes: vec![],
                appearance_stack: crate::scene::AppearanceStack::default(),
                path_points: vec![],
                path_closed: false,
                artboard_name: None,
                image_path: None,
            });

            rect_start = tag_end + 1;
        } else {
            rect_start = idx + 1;
        }
    }

    // Parse text elements
    let mut text_start = 0;
    while let Some(idx) = content[text_start..].find("<text") {
        let idx = text_start + idx;
        if let Some(tag_end) = content[idx..].find('>') {
            let tag_end = idx + tag_end;
            let tag = &content[idx..tag_end + 1];

            // Find text content (between > and </text>)
            let text_content_start = tag_end + 1;
            if let Some(text_end) = content[text_content_start..].find("</text>") {
                let text_content = &content[text_content_start..text_content_start + text_end];
                let text = text_content.trim().to_string();

                let id =
                    extract_attr(tag, "id").unwrap_or_else(|| format!("text_{}", elements.len()));
                let x: f32 = extract_float_attr(tag, "x").unwrap_or(0.0);
                let y: f32 = extract_float_attr(tag, "y").unwrap_or(0.0);
                let font_size: f32 = extract_float_attr(tag, "font-size")
                    .or_else(|| extract_float_attr(tag, "fontsize"))
                    .unwrap_or(14.0);

                // Try to get fill color from style attribute
                let fill = extract_fill_from_tag(tag);

                elements.push(LayoutElement {
                    id,
                    el_type: ElementType::Text,
                    x,
                    y,
                    w: text.len() as f32 * font_size * 0.6,
                    h: font_size * 1.2,
                    fill,
                    stroke: None,
                    text: Some(text),
                    text_size: Some(font_size),
                    children: vec![],
                    opacity: 1.0,
                    rotation_deg: 0.0,
                    corner_radius: 0.0,
                    gradient: None,
                    blend_mode: BlendMode::Normal,
                    effects: vec![],
                    stroke_dash: None,
                    clip_children: false,
                    text_align: None,
                    letter_spacing: None,
                    line_height: None,
                    stroke_cap: None,
                    stroke_join: None,
                    stroke_miter_limit: None,
                    text_decoration: None,
                    text_transform: None,
                    text_runs: vec![],
                    symbol_name: None,
                    is_compound_path: false,
                    is_gradient_mesh: false,
                    is_chart: false,
                    is_opaque: false,
                    third_party_effects: vec![],
                    notes: vec![],
                    appearance_fills: vec![],
                    appearance_strokes: vec![],
                    appearance_stack: crate::scene::AppearanceStack::default(),
                    path_points: vec![],
                    path_closed: false,
                    artboard_name: None,
                    image_path: None,
                });
            }

            text_start = tag_end + 1;
        } else {
            text_start = idx + 1;
        }
    }

    // Parse path elements
    let mut path_start = 0;
    while let Some(idx) = content[path_start..].find("<path") {
        let idx = path_start + idx;
        if let Some(tag_end) = content[idx..].find('>') {
            let tag_end = idx + tag_end;
            let tag = &content[idx..tag_end + 1];

            let id = extract_attr(tag, "id").unwrap_or_else(|| format!("path_{}", elements.len()));

            // Try to extract approximate bounds from path data
            let (w, h) = if let Some(d_start) = tag.find("d=\"") {
                let d_start = d_start + 3;
                if let Some(d_end) = tag[d_start..].find('"') {
                    let d = &tag[d_start..d_start + d_end];
                    estimate_path_bounds(d)
                } else {
                    (100.0, 100.0)
                }
            } else {
                (100.0, 100.0)
            };

            elements.push(LayoutElement {
                id,
                el_type: ElementType::Path,
                x: 0.0,
                y: 0.0,
                w,
                h,
                fill: extract_fill_from_tag(tag),
                stroke: extract_stroke_from_tag(tag),
                text: None,
                text_size: None,
                children: vec![],
                opacity: 1.0,
                rotation_deg: 0.0,
                corner_radius: 0.0,
                gradient: None,
                blend_mode: BlendMode::Normal,
                effects: vec![],
                stroke_dash: None,
                clip_children: false,
                text_align: None,
                letter_spacing: None,
                line_height: None,
                stroke_cap: None,
                stroke_join: None,
                stroke_miter_limit: None,
                text_decoration: None,
                text_transform: None,
                text_runs: vec![],
                symbol_name: None,
                is_compound_path: false,
                is_gradient_mesh: false,
                is_chart: false,
                is_opaque: false,
                third_party_effects: vec![],
                notes: vec![],
                appearance_fills: vec![],
                appearance_strokes: vec![],
                appearance_stack: crate::scene::AppearanceStack::default(),
                path_points: vec![],
                path_closed: false,
                artboard_name: None,
                image_path: None,
            });

            path_start = tag_end + 1;
        } else {
            path_start = idx + 1;
        }
    }

    // Parse image elements
    let mut img_start = 0;
    while let Some(idx) = content[img_start..].find("<image") {
        let idx = img_start + idx;
        if let Some(tag_end) = content[idx..].find('>') {
            let tag_end = idx + tag_end;
            let tag = &content[idx..tag_end + 1];

            let id = extract_attr(tag, "id").unwrap_or_else(|| format!("image_{}", elements.len()));
            let x: f32 = extract_float_attr(tag, "x").unwrap_or(0.0);
            let y: f32 = extract_float_attr(tag, "y").unwrap_or(0.0);
            let w: f32 = extract_float_attr(tag, "width").unwrap_or(100.0);
            let h: f32 = extract_float_attr(tag, "height").unwrap_or(100.0);
            let image_path = extract_attr(tag, "href").or_else(|| extract_attr(tag, "xlink:href"));

            elements.push(LayoutElement {
                id,
                el_type: ElementType::Image,
                x,
                y,
                w,
                h,
                fill: None,
                stroke: None,
                text: None,
                text_size: None,
                children: vec![],
                opacity: 1.0,
                rotation_deg: 0.0,
                corner_radius: 0.0,
                gradient: None,
                blend_mode: BlendMode::Normal,
                effects: vec![],
                stroke_dash: None,
                clip_children: false,
                text_align: None,
                letter_spacing: None,
                line_height: None,
                stroke_cap: None,
                stroke_join: None,
                stroke_miter_limit: None,
                text_decoration: None,
                text_transform: None,
                text_runs: vec![],
                symbol_name: None,
                is_compound_path: false,
                is_gradient_mesh: false,
                is_chart: false,
                is_opaque: false,
                third_party_effects: vec![],
                notes: vec![],
                appearance_fills: vec![],
                appearance_strokes: vec![],
                appearance_stack: crate::scene::AppearanceStack::default(),
                path_points: vec![],
                path_closed: false,
                artboard_name: None,
                image_path,
            });

            img_start = tag_end + 1;
        } else {
            img_start = idx + 1;
        }
    }

    elements
}
