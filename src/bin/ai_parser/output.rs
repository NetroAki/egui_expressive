use super::*;

pub fn generate_per_artboard_output(result: &AiParseResult) -> Vec<serde_json::Value> {
    let artboards = if result.artboards.is_empty() {
        vec![("default".to_string(), 0.0f64, 0.0f64, f64::MAX, f64::MAX)]
    } else {
        result
            .artboards
            .iter()
            .map(|a| (a.name.clone(), a.x, a.y, a.x + a.width, a.y + a.height))
            .collect::<Vec<_>>()
    };

    let mut entries: Vec<serde_json::Value> = Vec::new();
    for (artboard_idx, (name, _x1, _y1, _x2, _y2)) in artboards.iter().enumerate() {
        let sanitized = name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c.to_ascii_lowercase()
                } else {
                    '_'
                }
            })
            .collect::<String>();
        let sanitized = if sanitized.starts_with(|c: char| c.is_ascii_digit()) {
            format!("ab_{}", sanitized)
        } else if sanitized.is_empty() {
            "artboard".to_string()
        } else {
            sanitized
        };
        let filename = format!("{}.rs", sanitized);
        let selected_elements: Vec<&Element> = result
            .elements
            .iter()
            .filter(|e| !e.is_pseudo_element || !e.path_points.is_empty())
            .filter(|e| {
                element_belongs_to_artboard(
                    e,
                    name,
                    (*_x1, *_y1, _x2 - _x1, _y2 - _y1),
                    &artboards,
                    artboard_idx == 0,
                )
            })
            .collect();
        let element_count = selected_elements.len();
        let artboard_info = artboards.iter().find(|(n, _, _, _, _)| n == name);
        let (ab_w, ab_h) = artboard_info
            .map(|(_, x1, y1, x2, y2)| ((x2 - x1).abs(), (y2 - y1).abs()))
            .unwrap_or((375.0, 812.0));
        let layout_elements: Vec<LayoutElement> = selected_elements
            .iter()
            .enumerate()
            .map(|(i, e)| element_to_layout(e, i))
            .collect();
        let code = generate_artboard_file(
            name,
            ab_w as f32,
            ab_h as f32,
            &layout_elements,
            &std::collections::HashMap::new(),
        );
        entries.push(serde_json::json!({
            "artboard": name,
            "filename": filename,
            "width": ab_w,
            "height": ab_h,
            "element_count": element_count,
            "code": code,
            "elements": result.elements.iter()
                .filter(|e| {
                    element_belongs_to_artboard(
                        e,
                        name,
                        (*_x1, *_y1, _x2 - _x1, _y2 - _y1),
                        &artboards,
                        artboard_idx == 0,
                    )
                })
                .collect::<Vec<_>>(),
        }));
    }
    entries
}

pub fn generate_canvas_output(result: &AiParseResult) -> Vec<serde_json::Value> {
    let mut max_x = 0.0f64;
    let mut max_y = 0.0f64;

    for artboard in &result.artboards {
        max_x = max_x.max(artboard.x + artboard.width);
        max_y = max_y.max(artboard.y + artboard.height);
    }
    for element in &result.elements {
        let (x, y, w, h) = element_bounds(element);
        max_x = max_x.max(x + w);
        max_y = max_y.max(y + h);
    }

    let width = max_x.ceil().max(1.0);
    let height = max_y.ceil().max(1.0);
    let mut layout_elements: Vec<LayoutElement> = Vec::new();
    let mut background = LayoutElement::new(
        "pdf_page_background".to_string(),
        ElementType::Shape,
        0.0,
        0.0,
        width as f32,
        height as f32,
    );
    background.fill = Some(egui::Color32::WHITE);
    background.is_opaque = true;
    layout_elements.push(background);

    layout_elements.extend(
        result
            .elements
            .iter()
            .filter(|e| !e.is_pseudo_element || !e.path_points.is_empty())
            .enumerate()
            .map(|(i, e)| element_to_layout(e, i)),
    );
    let code = generate_artboard_file(
        "Full Canvas",
        width as f32,
        height as f32,
        &layout_elements,
        &std::collections::HashMap::new(),
    );

    vec![serde_json::json!({
        "artboard": "Full Canvas",
        "filename": "full_canvas.rs",
        "width": width,
        "height": height,
        "element_count": layout_elements.len(),
        "code": code,
        "elements": result.elements,
    })]
}
