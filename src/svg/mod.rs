#![allow(dead_code)]

use egui::{epaint::PathStroke, Color32, Pos2, Rect, Shape, Stroke};

/// Parse an SVG path `d` string into an egui Shape.
/// Supports: M/m, L/l, H/h, V/v, C/c, S/s, Q/q, T/t, A/a, Z/z commands.
/// Cubic bezier (C) and smooth cubic (S) are approximated with 16 line segments.
/// Quadratic bezier (Q) and smooth quadratic (T) are approximated with 8 line segments.
/// Elliptical arc (A) is approximated with line segments.
pub fn svg_path_to_shape(d: &str, fill: Color32, stroke: Stroke) -> Shape {
    let points = svg_path_to_points(d);
    if points.is_empty() {
        return Shape::Noop;
    }

    let closed = !d.trim().ends_with('Z') && !d.trim().ends_with('z');

    Shape::Path(egui::epaint::PathShape {
        points,
        closed,
        fill,
        stroke: PathStroke::new(stroke.width, stroke.color),
    })
}

/// Parse an SVG path and return the raw Vec<Pos2> points.
pub fn svg_path_to_points(d: &str) -> Vec<Pos2> {
    let mut points = Vec::new();
    let mut current_pos = Pos2::ZERO;
    let mut start_pos = Pos2::ZERO;
    let mut last_cubic_control: Option<Pos2> = None;
    let mut last_quad_control: Option<Pos2> = None;
    let mut tokens = tokenize_svg_path(d);
    let mut i = 0;

    while i < tokens.len() {
        let cmd = &tokens[i];
        i += 1;

        match cmd.as_str() {
            "M" | "m" => {
                // Reset smooth control points on moveto
                last_cubic_control = None;
                last_quad_control = None;

                while i < tokens.len() && !is_command(&tokens[i]) {
                    let x: f32 = tokens[i].parse().unwrap_or(0.0);
                    let y: f32 = tokens[i + 1].parse().unwrap_or(0.0);
                    i += 2;

                    if cmd == "m" && !points.is_empty() {
                        current_pos = Pos2::new(current_pos.x + x, current_pos.y + y);
                    } else {
                        current_pos = Pos2::new(x, y);
                    }

                    if points.is_empty() {
                        start_pos = current_pos;
                    }
                    points.push(current_pos);

                    // Implicit lineto after moveto
                    while i < tokens.len() && !is_command(&tokens[i]) {
                        let x: f32 = tokens[i].parse().unwrap_or(0.0);
                        let y: f32 = tokens[i + 1].parse().unwrap_or(0.0);
                        i += 2;

                        if cmd == "m" {
                            current_pos = Pos2::new(current_pos.x + x, current_pos.y + y);
                        } else {
                            current_pos = Pos2::new(x, y);
                        }
                        points.push(current_pos);
                    }
                }
            }
            "L" | "l" => {
                last_cubic_control = None;
                last_quad_control = None;

                while i < tokens.len() && !is_command(&tokens[i]) {
                    let x: f32 = tokens[i].parse().unwrap_or(0.0);
                    let y: f32 = tokens[i + 1].parse().unwrap_or(0.0);
                    i += 2;

                    if cmd == "l" {
                        current_pos = Pos2::new(current_pos.x + x, current_pos.y + y);
                    } else {
                        current_pos = Pos2::new(x, y);
                    }
                    points.push(current_pos);
                }
            }
            "H" | "h" => {
                last_cubic_control = None;
                last_quad_control = None;

                while i < tokens.len() && !is_command(&tokens[i]) {
                    let x: f32 = tokens[i].parse().unwrap_or(0.0);
                    i += 1;

                    if cmd == "h" {
                        current_pos.x += x;
                    } else {
                        current_pos.x = x;
                    }
                    points.push(current_pos);
                }
            }
            "V" | "v" => {
                last_cubic_control = None;
                last_quad_control = None;

                while i < tokens.len() && !is_command(&tokens[i]) {
                    let y: f32 = tokens[i].parse().unwrap_or(0.0);
                    i += 1;

                    if cmd == "v" {
                        current_pos.y += y;
                    } else {
                        current_pos.y = y;
                    }
                    points.push(current_pos);
                }
            }
            "C" | "c" => {
                last_quad_control = None;

                while i < tokens.len() && !is_command(&tokens[i]) {
                    let x1: f32 = tokens[i].parse().unwrap_or(0.0);
                    let y1: f32 = tokens[i + 1].parse().unwrap_or(0.0);
                    let x2: f32 = tokens[i + 2].parse().unwrap_or(0.0);
                    let y2: f32 = tokens[i + 3].parse().unwrap_or(0.0);
                    let x: f32 = tokens[i + 4].parse().unwrap_or(0.0);
                    let y: f32 = tokens[i + 5].parse().unwrap_or(0.0);
                    i += 6;

                    let p0 = current_pos;
                    let p1 = if cmd == "c" {
                        Pos2::new(p0.x + x1, p0.y + y1)
                    } else {
                        Pos2::new(x1, y1)
                    };
                    let p2 = if cmd == "c" {
                        Pos2::new(p0.x + x2, p0.y + y2)
                    } else {
                        Pos2::new(x2, y2)
                    };
                    let p3 = if cmd == "c" {
                        Pos2::new(p0.x + x, p0.y + y)
                    } else {
                        Pos2::new(x, y)
                    };

                    last_cubic_control = Some(p2);

                    // Sample 16 points along the cubic bezier
                    for k in 1..=16 {
                        let t = k as f32 / 16.0;
                        let t2 = t * t;
                        let t3 = t2 * t;
                        let mt = 1.0 - t;
                        let mt2 = mt * mt;
                        let mt3 = mt2 * mt;

                        let x_cubic =
                            mt3 * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t3 * p3.x;
                        let y_cubic =
                            mt3 * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t3 * p3.y;
                        points.push(Pos2::new(x_cubic, y_cubic));
                    }

                    current_pos = p3;
                }
            }
            "S" | "s" => {
                // Smooth cubic bezier: control1 is reflection of last cubic control
                last_quad_control = None;

                while i < tokens.len() && !is_command(&tokens[i]) {
                    let x2: f32 = tokens[i].parse().unwrap_or(0.0);
                    let y2: f32 = tokens[i + 1].parse().unwrap_or(0.0);
                    let x: f32 = tokens[i + 2].parse().unwrap_or(0.0);
                    let y: f32 = tokens[i + 3].parse().unwrap_or(0.0);
                    i += 4;

                    let p0 = current_pos;

                    // First control point is reflection of last cubic control around current pos
                    let p1 = if let Some(last_ctrl) = last_cubic_control {
                        Pos2::new(2.0 * p0.x - last_ctrl.x, 2.0 * p0.y - last_ctrl.y)
                    } else {
                        p0 // If no previous control, use current pos
                    };

                    let p2 = if cmd == "s" {
                        Pos2::new(p0.x + x2, p0.y + y2)
                    } else {
                        Pos2::new(x2, y2)
                    };
                    let p3 = if cmd == "s" {
                        Pos2::new(p0.x + x, p0.y + y)
                    } else {
                        Pos2::new(x, y)
                    };

                    last_cubic_control = Some(p2);

                    // Sample 16 points along the cubic bezier
                    for k in 1..=16 {
                        let t = k as f32 / 16.0;
                        let t2 = t * t;
                        let t3 = t2 * t;
                        let mt = 1.0 - t;
                        let mt2 = mt * mt;
                        let mt3 = mt2 * mt;

                        let x_cubic =
                            mt3 * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t3 * p3.x;
                        let y_cubic =
                            mt3 * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t3 * p3.y;
                        points.push(Pos2::new(x_cubic, y_cubic));
                    }

                    current_pos = p3;
                }
            }
            "Q" | "q" => {
                last_cubic_control = None;

                while i < tokens.len() && !is_command(&tokens[i]) {
                    let x1: f32 = tokens[i].parse().unwrap_or(0.0);
                    let y1: f32 = tokens[i + 1].parse().unwrap_or(0.0);
                    let x: f32 = tokens[i + 2].parse().unwrap_or(0.0);
                    let y: f32 = tokens[i + 3].parse().unwrap_or(0.0);
                    i += 4;

                    let p0 = current_pos;
                    let p1 = if cmd == "q" {
                        Pos2::new(p0.x + x1, p0.y + y1)
                    } else {
                        Pos2::new(x1, y1)
                    };
                    let p2 = if cmd == "q" {
                        Pos2::new(p0.x + x, p0.y + y)
                    } else {
                        Pos2::new(x, y)
                    };

                    last_quad_control = Some(p1);

                    // Sample 8 points along the quadratic bezier
                    for k in 1..=8 {
                        let t = k as f32 / 8.0;
                        let mt = 1.0 - t;
                        let mt2 = mt * mt;
                        let t2 = t * t;

                        let x_quad = mt2 * p0.x + 2.0 * mt * t * p1.x + t2 * p2.x;
                        let y_quad = mt2 * p0.y + 2.0 * mt * t * p1.y + t2 * p2.y;
                        points.push(Pos2::new(x_quad, y_quad));
                    }

                    current_pos = p2;
                }
            }
            "T" | "t" => {
                // Smooth quadratic bezier: control is reflection of last quadratic control
                last_cubic_control = None;

                while i < tokens.len() && !is_command(&tokens[i]) {
                    let x: f32 = tokens[i].parse().unwrap_or(0.0);
                    let y: f32 = tokens[i + 1].parse().unwrap_or(0.0);
                    i += 2;

                    let p0 = current_pos;

                    // Control point is reflection of last quad control around current pos
                    let p1 = if let Some(last_ctrl) = last_quad_control {
                        Pos2::new(2.0 * p0.x - last_ctrl.x, 2.0 * p0.y - last_ctrl.y)
                    } else {
                        p0 // If no previous control, use current pos
                    };

                    let p2 = if cmd == "t" {
                        Pos2::new(p0.x + x, p0.y + y)
                    } else {
                        Pos2::new(x, y)
                    };

                    last_quad_control = Some(p1);

                    // Sample 8 points along the quadratic bezier
                    for k in 1..=8 {
                        let t = k as f32 / 8.0;
                        let mt = 1.0 - t;
                        let mt2 = mt * mt;
                        let t2 = t * t;

                        let x_quad = mt2 * p0.x + 2.0 * mt * t * p1.x + t2 * p2.x;
                        let y_quad = mt2 * p0.y + 2.0 * mt * t * p1.y + t2 * p2.y;
                        points.push(Pos2::new(x_quad, y_quad));
                    }

                    current_pos = p2;
                }
            }
            "A" | "a" => {
                last_cubic_control = None;
                last_quad_control = None;

                while i < tokens.len() && !is_command(&tokens[i]) {
                    let rx: f32 = tokens[i].parse().unwrap_or(0.0);
                    let ry: f32 = tokens[i + 1].parse().unwrap_or(0.0);
                    let x_rot: f32 = tokens[i + 2].parse().unwrap_or(0.0);
                    let large_arc: i32 = tokens[i + 3].parse().unwrap_or(0);
                    let sweep: i32 = tokens[i + 4].parse().unwrap_or(0);
                    let x: f32 = tokens[i + 5].parse().unwrap_or(0.0);
                    let y: f32 = tokens[i + 6].parse().unwrap_or(0.0);
                    i += 7;

                    let end_point = if cmd == "a" {
                        Pos2::new(current_pos.x + x, current_pos.y + y)
                    } else {
                        Pos2::new(x, y)
                    };

                    // Approximate arc with line segments using SVG arc-to-center conversion
                    let arc_points = approximate_arc(
                        current_pos,
                        end_point,
                        rx,
                        ry,
                        x_rot,
                        large_arc != 0,
                        sweep != 0,
                    );
                    for pt in arc_points {
                        points.push(pt);
                    }

                    current_pos = end_point;
                }
            }
            "Z" | "z" => {
                last_cubic_control = None;
                last_quad_control = None;
                points.push(start_pos);
                current_pos = start_pos;
            }
            _ => {
                // Unknown command, skip
            }
        }
    }

    points
}

/// Approximate an elliptical arc with line segments.
/// Uses the SVG arc-to-center-parameterization algorithm.
fn approximate_arc(
    start: Pos2,
    end: Pos2,
    rx: f32,
    ry: f32,
    x_rotation: f32,
    large_arc: bool,
    sweep: bool,
) -> Vec<Pos2> {
    let mut points = Vec::new();

    // Handle degenerate cases
    if rx == 0.0 || ry == 0.0 {
        points.push(end);
        return points;
    }

    let rx = rx.abs();
    let ry = ry.abs();

    // Convert rotation angle to radians
    let r = x_rotation.to_radians();
    let (s, c) = r.sin_cos();

    // Step 1: Compute (x1', y1') - the start point rotated and translated
    let dx = (start.x - end.x) / 2.0;
    let dy = (start.y - end.y) / 2.0;

    let x1p = c * dx + s * dy;
    let y1p = -s * dx + c * dy;

    // Step 2: Correct radii if necessary
    let lambda = (x1p * x1p) / (rx * rx) + (y1p * y1p) / (ry * ry);
    if lambda > 1.0 {
        let sqrt_lambda = lambda.sqrt();
        let rx = sqrt_lambda * rx;
        let ry = sqrt_lambda * ry;
    }

    // Compute (cx', cy') - the center in transformed coordinates
    let rx_sq = rx * rx;
    let ry_sq = ry * ry;
    let x1p_sq = x1p * x1p;
    let y1p_sq = y1p * y1p;

    let mut radicand =
        (rx_sq * ry_sq - rx_sq * y1p_sq - ry_sq * x1p_sq) / (rx_sq * y1p_sq + ry_sq * x1p_sq);
    if radicand < 0.0 {
        radicand = 0.0;
    }
    let sqrt_radicand = radicand.sqrt();

    // Adjust sign based on large-arc-flag
    let sign = if large_arc == sweep { -1.0 } else { 1.0 };

    let cxp = sign * sqrt_radicand * rx * y1p / ry;
    let cyp = sign * sqrt_radicand * -ry * x1p / rx;

    // Step 3: Compute center point (cx, cy)
    let cx = c * cxp - s * cyp + (start.x + end.x) / 2.0;
    let cy = s * cxp + c * cyp + (start.y + end.y) / 2.0;

    // Step 4: Compute theta1 and dtheta
    let ux = (x1p - cxp) / rx;
    let uy = (y1p - cyp) / ry;
    let vx = (-x1p - cxp) / rx;
    let vy = (-y1p - cyp) / ry;

    let mut theta1 = (ux / ux.hypot(uy)).acos().to_degrees();
    if uy < 0.0 {
        theta1 = 360.0 - theta1;
    }

    let dtheta = ((ux * vx + uy * vy) / ((ux.hypot(uy)) * (vx.hypot(vy))))
        .acos()
        .to_degrees();
    let mut dtheta = if sweep { dtheta } else { -dtheta };
    if !large_arc && dtheta < 0.0 {
        dtheta += 360.0;
    } else if large_arc && dtheta > 0.0 {
        dtheta -= 360.0;
    }

    // Step 5: Sample the ellipse
    let num_segments = ((dtheta.abs() / 90.0).ceil() as i32).max(1) * 4;
    let dtheta_rad = dtheta.to_radians() / num_segments as f32;
    let theta1_rad = theta1.to_radians();

    for i in 1..=num_segments {
        let theta = theta1_rad + dtheta_rad * i as f32;
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        let xp = rx * cos_theta;
        let yp = ry * sin_theta;

        // Rotate back
        let px = c * xp - s * yp + cx;
        let py = s * xp + c * yp + cy;

        points.push(Pos2::new(px, py));
    }

    points
}

fn tokenize_svg_path(d: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut prev_was_command = true;

    for ch in d.chars() {
        if ch.is_ascii_alphabetic() && !current.is_empty() {
            tokens.push(current.clone());
            current.clear();
            tokens.push(ch.to_string());
            prev_was_command = true;
        } else if ch == '-' && !prev_was_command && !current.is_empty() {
            // Negative number after a value
            tokens.push(current.clone());
            current.clear();
            current.push(ch);
            prev_was_command = false;
        } else if ch.is_ascii_whitespace() || ch == ',' {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            prev_was_command = false;
        } else if ch == '.' && current == "-" {
            current.push('.');
        } else if ch.is_ascii_digit() || ch == '.' || ch == '-' {
            current.push(ch);
            prev_was_command = false;
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn is_command(s: &str) -> bool {
    s.len() == 1 && s.chars().next().map_or(false, |c| c.is_ascii_alphabetic())
}

/// Parse a minimal SVG string (no external deps, no full XML parser).
/// Extracts <path> elements and returns shapes + their bounding rects.
/// Handles fill="..." stroke="..." stroke-width="..." attributes.
/// Ignores unsupported elements.
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
                        let closed = !d.contains('Z') && !d.contains('z');

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

fn extract_color_attr(tag: &str, attr_name: &str) -> Option<Color32> {
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

fn extract_float_attr(tag: &str, attr_name: &str) -> Option<f32> {
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
    if s.starts_with('#') {
        let hex = &s[1..];

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
        "none" => return Some(Color32::TRANSPARENT),
        "black" => return Some(Color32::BLACK),
        "white" => return Some(Color32::WHITE),
        "red" => return Some(Color32::from_rgb(255, 0, 0)),
        "green" => return Some(Color32::from_rgb(0, 128, 0)),
        "blue" => return Some(Color32::from_rgb(0, 0, 255)),
        "yellow" => return Some(Color32::from_rgb(255, 255, 0)),
        "cyan" => return Some(Color32::from_rgb(0, 255, 255)),
        "magenta" => return Some(Color32::from_rgb(255, 0, 255)),
        "gray" | "grey" => return Some(Color32::from_gray(128)),
        "orange" => return Some(Color32::from_rgb(255, 165, 0)),
        "purple" => return Some(Color32::from_rgb(128, 0, 128)),
        "pink" => return Some(Color32::from_rgb(255, 192, 203)),
        "brown" => return Some(Color32::from_rgb(165, 42, 42)),
        _ => return None,
    }
}

// ============================================================================
// ASE (Adobe Swatch Exchange) Parser
// ============================================================================

#[derive(Debug)]
pub enum AseError {
    InvalidMagic,
    InvalidVersion,
    UnexpectedEof,
    Utf16Error,
}

impl std::fmt::Display for AseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AseError::InvalidMagic => write!(f, "Invalid ASE magic bytes"),
            AseError::InvalidVersion => write!(f, "Invalid or unsupported ASE version"),
            AseError::UnexpectedEof => write!(f, "Unexpected end of file"),
            AseError::Utf16Error => write!(f, "Invalid UTF-16 encoding in ASE file"),
        }
    }
}

impl std::error::Error for AseError {}

/// Parse an Adobe Swatch Exchange (.ase) binary file.
/// Returns a list of (name, Color32) pairs.
pub fn parse_ase(bytes: &[u8]) -> Result<Vec<(String, Color32)>, AseError> {
    if bytes.len() < 12 {
        return Err(AseError::UnexpectedEof);
    }

    // Check magic bytes "ASEF"
    if &bytes[0..4] != b"ASEF" {
        return Err(AseError::InvalidMagic);
    }

    // Read version (big-endian)
    let major_version = u16::from_be_bytes([bytes[4], bytes[5]]);
    let minor_version = u16::from_be_bytes([bytes[6], bytes[7]]);

    if major_version != 1 || minor_version != 0 {
        return Err(AseError::InvalidVersion);
    }

    // Read block count (big-endian u32)
    let block_count = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);

    let mut colors = Vec::new();
    let mut offset = 12;

    for _ in 0..block_count {
        if offset + 6 > bytes.len() {
            return Err(AseError::UnexpectedEof);
        }

        // Read block type (big-endian u16)
        let block_type = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]);
        offset += 2;

        // Read block length (big-endian u32)
        let block_length = u32::from_be_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;

        match block_type {
            0x0001 => {
                // Color block
                if offset + 2 > bytes.len() {
                    return Err(AseError::UnexpectedEof);
                }

                // Read name length (big-endian u16)
                let name_len = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]);
                offset += 2;

                let name_bytes_len = (name_len as usize) * 2;
                if offset + name_bytes_len > bytes.len() {
                    return Err(AseError::UnexpectedEof);
                }

                // Read name as UTF-16BE
                let mut name_chars = Vec::with_capacity(name_len as usize);
                for i in 0..name_len {
                    let char_bytes = [
                        bytes[offset + i as usize * 2],
                        bytes[offset + i as usize * 2 + 1],
                    ];
                    let c = char::from_u32(u16::from_be_bytes(char_bytes) as u32)
                        .ok_or(AseError::Utf16Error)?;
                    name_chars.push(c);
                }
                let name: String = name_chars.into_iter().collect();
                offset += name_bytes_len;

                if offset + 4 > bytes.len() {
                    return Err(AseError::UnexpectedEof);
                }

                // Read color model (4 bytes)
                let color_model = &bytes[offset..offset + 4];
                offset += 4;

                let color = match color_model {
                    b"RGB " => {
                        if offset + 12 > bytes.len() {
                            return Err(AseError::UnexpectedEof);
                        }

                        let r = f32::from_be_bytes([
                            bytes[offset],
                            bytes[offset + 1],
                            bytes[offset + 2],
                            bytes[offset + 3],
                        ]);
                        let g = f32::from_be_bytes([
                            bytes[offset + 4],
                            bytes[offset + 5],
                            bytes[offset + 6],
                            bytes[offset + 7],
                        ]);
                        let b = f32::from_be_bytes([
                            bytes[offset + 8],
                            bytes[offset + 9],
                            bytes[offset + 10],
                            bytes[offset + 11],
                        ]);

                        offset += 12;

                        let r = (r.max(0.0).min(1.0) * 255.0) as u8;
                        let g = (g.max(0.0).min(1.0) * 255.0) as u8;
                        let b = (b.max(0.0).min(1.0) * 255.0) as u8;

                        Color32::from_rgb(r, g, b)
                    }
                    b"CMYK" => {
                        if offset + 16 > bytes.len() {
                            return Err(AseError::UnexpectedEof);
                        }

                        let c = f32::from_be_bytes([
                            bytes[offset],
                            bytes[offset + 1],
                            bytes[offset + 2],
                            bytes[offset + 3],
                        ]);
                        let m = f32::from_be_bytes([
                            bytes[offset + 4],
                            bytes[offset + 5],
                            bytes[offset + 6],
                            bytes[offset + 7],
                        ]);
                        let y = f32::from_be_bytes([
                            bytes[offset + 8],
                            bytes[offset + 9],
                            bytes[offset + 10],
                            bytes[offset + 11],
                        ]);
                        let k = f32::from_be_bytes([
                            bytes[offset + 12],
                            bytes[offset + 13],
                            bytes[offset + 14],
                            bytes[offset + 15],
                        ]);

                        offset += 16;

                        // CMYK to RGB conversion
                        let c = c.max(0.0).min(1.0);
                        let m = m.max(0.0).min(1.0);
                        let y = y.max(0.0).min(1.0);
                        let k = k.max(0.0).min(1.0);

                        let r = (255.0 * (1.0 - c) * (1.0 - k)) as u8;
                        let g = (255.0 * (1.0 - m) * (1.0 - k)) as u8;
                        let b = (255.0 * (1.0 - y) * (1.0 - k)) as u8;

                        Color32::from_rgb(r, g, b)
                    }
                    b"Gray" => {
                        if offset + 4 > bytes.len() {
                            return Err(AseError::UnexpectedEof);
                        }

                        let gray = f32::from_be_bytes([
                            bytes[offset],
                            bytes[offset + 1],
                            bytes[offset + 2],
                            bytes[offset + 3],
                        ]);
                        offset += 4;

                        let gray = (gray.max(0.0).min(1.0) * 255.0) as u8;
                        Color32::from_gray(gray)
                    }
                    b"LAB " => {
                        // LAB is not supported, skip the color data (3 floats = 12 bytes)
                        if offset + 12 > bytes.len() {
                            return Err(AseError::UnexpectedEof);
                        }
                        offset += 12;
                        continue; // Don't add this color
                    }
                    _ => {
                        // Unknown color model, skip the block content
                        offset += block_length as usize - 6 - name_bytes_len - 4;
                        continue;
                    }
                };

                colors.push((name, color));
            }
            0xC001 => {
                // Group start - skip content
                offset += block_length as usize;
            }
            0xC002 => {
                // Group end - skip content
                offset += block_length as usize;
            }
            _ => {
                // Unknown block type, skip
                offset += block_length as usize;
            }
        }

        // Align to even byte boundary (ASE spec requires this)
        if offset % 2 != 0 {
            offset += 1;
        }
    }

    Ok(colors)
}

/// Convert ASE parse result to a flat Vec of Color32 values (names discarded).
pub fn ase_to_colors(bytes: &[u8]) -> Result<Vec<Color32>, AseError> {
    let colors = parse_ase(bytes)?;
    Ok(colors.into_iter().map(|(_, color)| color).collect())
}
