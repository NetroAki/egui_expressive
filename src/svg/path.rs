use egui::{epaint::PathStroke, Color32, Pos2, Shape, Stroke};

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

    let closed = d.trim().ends_with('Z') || d.trim().ends_with('z');

    Shape::Path(egui::epaint::PathShape {
        points,
        closed,
        fill,
        stroke: PathStroke::new(stroke.width, stroke.color),
    })
}

/// Parse an SVG path and return the raw `Vec<Pos2>` points.
pub fn svg_path_to_points(d: &str) -> Vec<Pos2> {
    let mut points = Vec::new();
    let mut current_pos = Pos2::ZERO;
    let mut start_pos = Pos2::ZERO;
    let mut last_cubic_control: Option<Pos2> = None;
    let mut last_quad_control: Option<Pos2> = None;
    let tokens = tokenize_svg_path(d);
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
pub(crate) fn approximate_arc(
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

    let mut rx = rx.abs();
    let mut ry = ry.abs();

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
        rx *= sqrt_lambda;
        ry *= sqrt_lambda;
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

pub(crate) fn tokenize_svg_path(d: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut prev_was_command = true;
    let mut has_dot = false;

    for ch in d.chars() {
        if ch.is_ascii_alphabetic() && !current.is_empty() {
            tokens.push(current.clone());
            current.clear();
            has_dot = false;
            tokens.push(ch.to_string());
            prev_was_command = true;
        } else if ch == '-' && !prev_was_command && !current.is_empty() {
            // Negative number after a value
            tokens.push(current.clone());
            current.clear();
            has_dot = false;
            current.push(ch);
            prev_was_command = false;
        } else if ch.is_ascii_whitespace() || ch == ',' {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
                has_dot = false;
            }
            prev_was_command = false;
        } else if ch == '.' && current == "-" {
            current.push('.');
            has_dot = true;
        } else if ch == '.' && has_dot {
            // Second dot in a number, split here
            tokens.push(current.clone());
            current.clear();
            current.push('.');
            has_dot = true;
            prev_was_command = false;
        } else if ch.is_ascii_digit() || ch == '.' || ch == '-' {
            if ch == '.' {
                has_dot = true;
            }
            current.push(ch);
            prev_was_command = false;
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

pub(crate) fn is_command(s: &str) -> bool {
    s.len() == 1 && s.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
}
