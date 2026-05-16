use super::*;

fn find_matching_close(s: &str, tag: &str) -> Option<usize> {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);

    let mut depth = 1;
    let mut pos = 0;

    while depth > 0 && pos < s.len() {
        if s[pos..].starts_with(&open) && !s[pos..].starts_with(&format!("{}/", open)) {
            depth += 1;
            pos += open.len();
        } else if s[pos..].starts_with(&close) {
            depth -= 1;
            if depth == 0 {
                return Some(pos);
            }
            pos += close.len();
        } else {
            pos += 1;
        }
    }

    None
}

fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let patterns = [format!("{}=", attr), format!("{} =", attr)];

    for pattern in &patterns {
        if let Some(idx) = tag.find(pattern) {
            let idx = idx + pattern.len();
            if idx < tag.len() && (tag[idx..].starts_with('"') || tag[idx..].starts_with('\'')) {
                let quote = tag[idx..].chars().next()?;
                let rest = &tag[idx + 1..];
                if let Some(end_idx) = rest.find(quote) {
                    return Some(rest[..end_idx].to_string());
                }
            }
        }
    }

    None
}

fn extract_float_attr(tag: &str, attr: &str) -> Option<f32> {
    extract_attr(tag, attr)?.parse().ok()
}

fn extract_fill_from_tag(tag: &str) -> Option<Color32> {
    // Try fill attribute
    if let Some(fill) = extract_attr(tag, "fill") {
        if fill != "none" {
            if let Some(c) = crate::svg::parse_svg_color(&fill) {
                return Some(c);
            }
        }
    }

    // Try style attribute
    if let Some(style) = extract_attr(tag, "style") {
        // Look for fill: in style
        if let Some(f_start) = style.find("fill:") {
            let after_fill = &style[f_start + 5..];
            // Get the value until ; or end
            let value = after_fill.trim_start_matches(' ').trim_start_matches(':');
            let end = value.find(';').unwrap_or(value.len());
            let fill_value = value[..end].trim();
            if fill_value != "none" {
                if let Some(c) = crate::svg::parse_svg_color(fill_value) {
                    return Some(c);
                }
            }
        }
    }

    None
}

fn extract_stroke_from_tag(tag: &str) -> Option<(f32, Color32)> {
    let stroke_color = extract_attr(tag, "stroke");
    let stroke_width = extract_float_attr(tag, "stroke-width");

    if let Some(color_str) = stroke_color {
        if color_str != "none" {
            if let Some(c) = crate::svg::parse_svg_color(&color_str) {
                return Some((stroke_width.unwrap_or(1.0), c));
            }
        }
    }

    None
}

fn extract_transform_xy(tag: &str) -> (Option<f32>, Option<f32>) {
    if let Some(transform) = extract_attr(tag, "transform") {
        // Parse translate(x, y) or translate(x y)
        if let Some(inner) = transform.strip_prefix("translate(") {
            if let Some(end) = inner.find(')') {
                let coords = &inner[..end];
                let parts: Vec<&str> = coords
                    .split(|c: char| c == ',' || c.is_whitespace())
                    .filter(|s| !s.is_empty())
                    .collect();

                if parts.len() >= 2 {
                    let x = parts[0].parse().ok();
                    let y = parts[1].parse().ok();
                    return (x, y);
                } else if parts.len() == 1 {
                    let x = parts[0].parse().ok();
                    return (x, None);
                }
            }
        }
    }

    (None, None)
}

fn estimate_path_bounds(d: &str) -> (f32, f32) {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    let mut current_x = 0.0f32;
    let mut current_y = 0.0f32;

    let tokens: Vec<&str> = d.split(|c: char| c.is_whitespace() || c == ',').collect();
    let mut i = 0;

    while i < tokens.len() {
        let token = tokens[i];

        match token {
            "M" | "L" | "m" | "l" => {
                if i + 2 < tokens.len() {
                    let x: f32 = tokens[i + 1].parse().unwrap_or(0.0);
                    let y: f32 = tokens[i + 2].parse().unwrap_or(0.0);

                    if token == "m" || token == "l" {
                        current_x += x;
                        current_y += y;
                    } else {
                        current_x = x;
                        current_y = y;
                    }

                    min_x = min_x.min(current_x);
                    min_y = min_y.min(current_y);
                    max_x = max_x.max(current_x);
                    max_y = max_y.max(current_y);

                    i += 3;
                } else {
                    i += 1;
                }
            }
            "H" | "h" | "V" | "v" => {
                if i + 1 < tokens.len() {
                    let val: f32 = tokens[i + 1].parse().unwrap_or(0.0);
                    if token == "h" {
                        current_x += val;
                    } else if token == "v" {
                        current_y += val;
                    } else if token == "H" {
                        current_x = val;
                    } else {
                        current_y = val;
                    }

                    min_x = min_x.min(current_x);
                    min_y = min_y.min(current_y);
                    max_x = max_x.max(current_x);
                    max_y = max_y.max(current_y);

                    i += 2;
                } else {
                    i += 1;
                }
            }
            "C" | "c" => {
                if i + 6 < tokens.len() {
                    let x: f32 = tokens[i + 5].parse().unwrap_or(0.0);
                    let y: f32 = tokens[i + 6].parse().unwrap_or(0.0);

                    if token == "c" {
                        current_x += x;
                        current_y += y;
                    } else {
                        current_x = x;
                        current_y = y;
                    }

                    min_x = min_x.min(current_x);
                    min_y = min_y.min(current_y);
                    max_x = max_x.max(current_x);
                    max_y = max_y.max(current_y);

                    i += 7;
                } else {
                    i += 1;
                }
            }
            "Q" | "q" => {
                if i + 4 < tokens.len() {
                    let x: f32 = tokens[i + 3].parse().unwrap_or(0.0);
                    let y: f32 = tokens[i + 4].parse().unwrap_or(0.0);

                    if token == "q" {
                        current_x += x;
                        current_y += y;
                    } else {
                        current_x = x;
                        current_y = y;
                    }

                    min_x = min_x.min(current_x);
                    min_y = min_y.min(current_y);
                    max_x = max_x.max(current_x);
                    max_y = max_y.max(current_y);

                    i += 5;
                } else {
                    i += 1;
                }
            }
            "Z" | "z" => {
                i += 1;
            }
            _ => {
                // Try to parse as a number
                if let Ok(val) = token.parse::<f32>() {
                    if i + 1 < tokens.len() {
                        if let Ok(y_val) = tokens[i + 1].parse::<f32>() {
                            current_x += val;
                            current_y += y_val;

                            min_x = min_x.min(current_x);
                            min_y = min_y.min(current_y);
                            max_x = max_x.max(current_x);
                            max_y = max_y.max(current_y);

                            i += 2;
                        } else {
                            i += 1;
                        }
                    } else {
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            }
        }
    }

    if min_x == f32::INFINITY {
        return (100.0, 100.0);
    }

    ((max_x - min_x).max(1.0), (max_y - min_y).max(1.0))
}
