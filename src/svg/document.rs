use super::*;

/// Parse a minimal SVG string (no external deps, no full XML parser).
/// Extracts `<path>` elements and returns shapes + their bounding rects.
/// Handles `fill="..."`, `stroke="..."`, and `stroke-width="..."` attributes.
pub fn svg_to_shapes(svg: &str) -> Vec<(Shape, Rect)> {
    let mut shapes = Vec::new();

    let mut path_start = 0;
    while let Some(idx) = svg[path_start..].find("<path") {
        let idx = idx + path_start;
        if let Some(tag_end) = svg[idx..].find('>') {
            let tag_end = idx + tag_end;
            let tag = &svg[idx..tag_end];

            // Extract d attribute
            if let Some(d_start) = tag.find("d=\"") {
                let d_start = d_start + 3;
                if let Some(d_end) = tag[d_start..].find('"') {
                    let d = &tag[d_start..d_start + d_end];

                    // Extract fill
                    let fill = extract_color_attr(tag, "fill").unwrap_or(Color32::BLACK);

                    // Extract stroke
                    let stroke_color =
                        extract_color_attr(tag, "stroke").unwrap_or(Color32::TRANSPARENT);

                    // Extract stroke-width
                    let stroke_width = extract_float_attr(tag, "stroke-width").unwrap_or(1.0);

                    let stroke = Stroke::new(stroke_width, stroke_color);

                    let points = svg_path_to_points(d);
                    if !points.is_empty() {
                        let closed = d.contains('Z') || d.contains('z');

                        // Calculate bounding rect
                        let min_x = points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
                        let min_y = points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
                        let max_x = points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
                        let max_y = points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);

                        let rect =
                            Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y));

                        let shape = Shape::Path(egui::epaint::PathShape {
                            points,
                            closed,
                            fill,
                            stroke: PathStroke::new(stroke.width, stroke.color),
                        });

                        shapes.push((shape, rect));
                    }
                }
            }

            path_start = tag_end + 1;
        } else {
            path_start = idx + 1;
        }
    }

    shapes
}

pub(crate) fn extract_color_attr(tag: &str, attr_name: &str) -> Option<Color32> {
    let pattern = format!("{}=\"", attr_name);
    if let Some(idx) = tag.find(&pattern) {
        let idx = idx + pattern.len();
        if let Some(end) = tag[idx..].find('"') {
            let value = &tag[idx..idx + end];
            return parse_svg_color(value);
        }
    }
    None
}

pub(crate) fn extract_float_attr(tag: &str, attr_name: &str) -> Option<f32> {
    let pattern = format!("{}=\"", attr_name);
    if let Some(idx) = tag.find(&pattern) {
        let idx = idx + pattern.len();
        if let Some(end) = tag[idx..].find('"') {
            let value = &tag[idx..idx + end];
            return value.parse().ok();
        }
    }
    None
}

/// Parse a CSS/SVG color string: #rgb, #rrggbb, #rrggbbaa, rgb(r,g,b), rgba(r,g,b,a)
pub fn parse_svg_color(s: &str) -> Option<Color32> {
    let s = s.trim();

    // Handle hex colors
    if let Some(hex) = s.strip_prefix('#') {
        match hex.len() {
            3 => {
                // #rgb
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                return Some(Color32::from_rgb(r, g, b));
            }
            6 => {
                // #rrggbb
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                return Some(Color32::from_rgb(r, g, b));
            }
            8 => {
                // #rrggbbaa
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                return Some(Color32::from_rgba_unmultiplied(r, g, b, a));
            }
            _ => return None,
        }
    }

    // Handle rgb/rgba
    if s.starts_with("rgb(") || s.starts_with("rgba(") {
        let inner = if s.starts_with("rgba(") {
            &s[5..s.len() - 1]
        } else {
            &s[4..s.len() - 1]
        };

        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();

        if parts.len() >= 3 {
            let r: f32 = parts[0].parse().ok()?;
            let g: f32 = parts[1].parse().ok()?;
            let b: f32 = parts[2].parse().ok()?;

            let a: f32 = if parts.len() >= 4 {
                parts[3].parse().ok().unwrap_or(1.0)
            } else {
                1.0
            };

            // Handle percentage values
            let r = if parts[0].ends_with('%') {
                let v: f32 = parts[0][..parts[0].len() - 1].parse().ok()?;
                (v * 255.0 / 100.0) as u8
            } else {
                r as u8
            };

            let g = if parts[1].ends_with('%') {
                let v: f32 = parts[1][..parts[1].len() - 1].parse().ok()?;
                (v * 255.0 / 100.0) as u8
            } else {
                g as u8
            };

            let b = if parts[2].ends_with('%') {
                let v: f32 = parts[2][..parts[2].len() - 1].parse().ok()?;
                (v * 255.0 / 100.0) as u8
            } else {
                b as u8
            };

            let a = (a * 255.0) as u8;

            return Some(Color32::from_rgba_unmultiplied(r, g, b, a));
        }

        return None;
    }

    // Handle named colors (common ones)
    match s.to_lowercase().as_str() {
        "none" => Some(Color32::TRANSPARENT),
        "black" => Some(Color32::BLACK),
        "white" => Some(Color32::WHITE),
        "red" => Some(Color32::from_rgb(255, 0, 0)),
        "green" => Some(Color32::from_rgb(0, 128, 0)),
        "blue" => Some(Color32::from_rgb(0, 0, 255)),
        "yellow" => Some(Color32::from_rgb(255, 255, 0)),
        "cyan" => Some(Color32::from_rgb(0, 255, 255)),
        "magenta" => Some(Color32::from_rgb(255, 0, 255)),
        "gray" | "grey" => Some(Color32::from_gray(128)),
        "orange" => Some(Color32::from_rgb(255, 165, 0)),
        "purple" => Some(Color32::from_rgb(128, 0, 128)),
        "pink" => Some(Color32::from_rgb(255, 192, 203)),
        "brown" => Some(Color32::from_rgb(165, 42, 42)),
        _ => None,
    }
}

// ============================================================================
// ASE (Adobe Swatch Exchange) Parser
// ============================================================================
