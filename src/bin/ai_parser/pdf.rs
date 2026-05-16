use super::*;

#[derive(Clone)]
pub(crate) struct PdfGraphicsState {
    ctm: [f64; 6],
    fill: Color,
    stroke: Stroke,
}

impl Default for PdfGraphicsState {
    fn default() -> Self {
        Self {
            ctm: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            fill: Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
                opacity: Some(1.0),
                blend_mode: "normal".to_string(),
            },
            stroke: Stroke {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
                width: 1.0,
                opacity: Some(1.0),
                blend_mode: "normal".to_string(),
                cap: None,
                join: None,
                dash: None,
                miter_limit: None,
                gradient: None,
            },
        }
    }
}

pub(crate) fn concat_ctm(current: [f64; 6], next: [f64; 6]) -> [f64; 6] {
    let [a, b, c, d, e, f] = current;
    let [g, h, i, j, k, l] = next;
    [
        a * g + c * h,
        b * g + d * h,
        a * i + c * j,
        b * i + d * j,
        a * k + c * l + e,
        b * k + d * l + f,
    ]
}

pub(crate) fn transform_pdf_point(ctm: [f64; 6], x: f64, y: f64) -> [f64; 2] {
    [
        ctm[0] * x + ctm[2] * y + ctm[4],
        ctm[1] * x + ctm[3] * y + ctm[5],
    ]
}

pub(crate) fn pdf_color_from_components(values: &[f64], blend_mode: &str) -> Color {
    let (r, g, b) = match values.len() {
        0 => (0.0, 0.0, 0.0),
        1 => {
            let gray = values[0].clamp(0.0, 1.0);
            (gray, gray, gray)
        }
        2 | 3 => (
            values[0].clamp(0.0, 1.0),
            values.get(1).copied().unwrap_or(0.0).clamp(0.0, 1.0),
            values.get(2).copied().unwrap_or(0.0).clamp(0.0, 1.0),
        ),
        _ => {
            let c = values[0].clamp(0.0, 1.0);
            let m = values[1].clamp(0.0, 1.0);
            let y = values[2].clamp(0.0, 1.0);
            let k = values[3].clamp(0.0, 1.0);
            (
                (1.0 - c) * (1.0 - k),
                (1.0 - m) * (1.0 - k),
                (1.0 - y) * (1.0 - k),
            )
        }
    };

    Color {
        r: (r * 255.0).round().clamp(0.0, 255.0) as u8,
        g: (g * 255.0).round().clamp(0.0, 255.0) as u8,
        b: (b * 255.0).round().clamp(0.0, 255.0) as u8,
        a: 255,
        opacity: Some(1.0),
        blend_mode: blend_mode.to_string(),
    }
}

pub(crate) fn path_bounds(points: &[PathPoint]) -> Option<[f64; 4]> {
    let first = points.first()?;
    let mut min_x = first.anchor[0];
    let mut min_y = first.anchor[1];
    let mut max_x = first.anchor[0];
    let mut max_y = first.anchor[1];
    for point in points {
        for [x, y] in [point.anchor, point.left_ctrl, point.right_ctrl] {
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
    }
    Some([
        min_x,
        min_y,
        (max_x - min_x).max(1.0),
        (max_y - min_y).max(1.0),
    ])
}

pub(crate) fn painted_path_element(
    stream_idx: usize,
    element_idx: usize,
    points: &[PathPoint],
    closed: bool,
    state: &PdfGraphicsState,
    fill: bool,
    stroke: bool,
) -> Option<Element> {
    if points.is_empty() || (!fill && !stroke) {
        return None;
    }
    let mut element = Element {
        id: format!("pdf_path_{}_{}", stream_idx, element_idx),
        element_type: Some("shape".to_string()),
        path_points: points.to_vec(),
        path_closed: closed,
        corner_radius: detect_corner_radius(points),
        is_pseudo_element: true,
        ..Default::default()
    };
    if fill {
        element.appearance_fills.push(state.fill.clone());
    }
    if stroke && state.stroke.width > 0.0 {
        element.appearance_strokes.push(state.stroke.clone());
    }
    element.bounds = path_bounds(points);
    Some(element)
}

/// Parse painted PDF path objects from a content stream.
///
/// This keeps the Illustrator/PDF reference path vector-only: it converts PDF path paint commands
/// into the same codegen/scene primitives used by hand-authored egui_expressive code rather than
/// embedding the rendered PDF/PNG as an image.
pub(crate) fn parse_pdf_painted_path_elements(content: &str, stream_idx: usize) -> Vec<Element> {
    let token_re = match Regex::new(
        r"/[A-Za-z0-9_.#-]+|-?\d*\.?\d+(?:[eE][+-]?\d+)?|f\*|B\*|b\*|[A-Za-z]{1,3}|\S",
    ) {
        Ok(re) => re,
        Err(_) => return Vec::new(),
    };
    let mut state = PdfGraphicsState::default();
    let mut stack: Vec<f64> = Vec::new();
    let mut saved_states: Vec<PdfGraphicsState> = Vec::new();
    let mut path: Vec<PathPoint> = Vec::new();
    let mut closed = false;
    let mut elements = Vec::new();

    for token in token_re.find_iter(content).map(|m| m.as_str()) {
        if let Ok(value) = token.parse::<f64>() {
            stack.push(value);
            continue;
        }
        if token.starts_with('/') {
            continue;
        }

        match token {
            "q" => saved_states.push(state.clone()),
            "Q" => {
                if let Some(saved) = saved_states.pop() {
                    state = saved;
                }
                path.clear();
                closed = false;
            }
            "cm" if stack.len() >= 6 => {
                let m = [
                    stack[stack.len() - 6],
                    stack[stack.len() - 5],
                    stack[stack.len() - 4],
                    stack[stack.len() - 3],
                    stack[stack.len() - 2],
                    stack[stack.len() - 1],
                ];
                state.ctm = concat_ctm(state.ctm, m);
            }
            "w" if !stack.is_empty() => {
                state.stroke.width = stack[stack.len() - 1].max(0.0);
            }
            "J" if !stack.is_empty() => {
                state.stroke.cap = match stack[stack.len() - 1].round() as i32 {
                    0 => Some("butt".to_string()),
                    1 => Some("round".to_string()),
                    2 => Some("square".to_string()),
                    _ => state.stroke.cap.clone(),
                };
            }
            "j" if !stack.is_empty() => {
                state.stroke.join = match stack[stack.len() - 1].round() as i32 {
                    0 => Some("miter".to_string()),
                    1 => Some("round".to_string()),
                    2 => Some("bevel".to_string()),
                    _ => state.stroke.join.clone(),
                };
            }
            "M" if !stack.is_empty() => {
                state.stroke.miter_limit = Some(stack[stack.len() - 1] as f32);
            }
            "rg" if stack.len() >= 3 => {
                state.fill =
                    pdf_color_from_components(&stack[stack.len() - 3..], &state.fill.blend_mode);
            }
            "RG" if stack.len() >= 3 => {
                let mut stroke = state.stroke.clone();
                let color =
                    pdf_color_from_components(&stack[stack.len() - 3..], &stroke.blend_mode);
                stroke.r = color.r;
                stroke.g = color.g;
                stroke.b = color.b;
                stroke.a = color.a;
                state.stroke = stroke;
            }
            "g" if !stack.is_empty() => {
                state.fill =
                    pdf_color_from_components(&stack[stack.len() - 1..], &state.fill.blend_mode);
            }
            "G" if !stack.is_empty() => {
                let mut stroke = state.stroke.clone();
                let color =
                    pdf_color_from_components(&stack[stack.len() - 1..], &stroke.blend_mode);
                stroke.r = color.r;
                stroke.g = color.g;
                stroke.b = color.b;
                stroke.a = color.a;
                state.stroke = stroke;
            }
            "k" | "sc" | "scn" if !stack.is_empty() => {
                state.fill = pdf_color_from_components(&stack, &state.fill.blend_mode);
            }
            "K" | "SC" | "SCN" if !stack.is_empty() => {
                let mut stroke = state.stroke.clone();
                let color = pdf_color_from_components(&stack, &stroke.blend_mode);
                stroke.r = color.r;
                stroke.g = color.g;
                stroke.b = color.b;
                stroke.a = color.a;
                state.stroke = stroke;
            }
            "m" if stack.len() >= 2 => {
                let [x, y] =
                    transform_pdf_point(state.ctm, stack[stack.len() - 2], stack[stack.len() - 1]);
                path.clear();
                closed = false;
                path.push(PathPoint {
                    anchor: [x, y],
                    left_ctrl: [x, y],
                    right_ctrl: [x, y],
                });
            }
            "l" if stack.len() >= 2 => {
                let [x, y] =
                    transform_pdf_point(state.ctm, stack[stack.len() - 2], stack[stack.len() - 1]);
                path.push(PathPoint {
                    anchor: [x, y],
                    left_ctrl: [x, y],
                    right_ctrl: [x, y],
                });
            }
            "c" if stack.len() >= 6 => {
                let [x1, y1] =
                    transform_pdf_point(state.ctm, stack[stack.len() - 6], stack[stack.len() - 5]);
                let [x2, y2] =
                    transform_pdf_point(state.ctm, stack[stack.len() - 4], stack[stack.len() - 3]);
                let [x3, y3] =
                    transform_pdf_point(state.ctm, stack[stack.len() - 2], stack[stack.len() - 1]);
                if let Some(prev) = path.last_mut() {
                    prev.right_ctrl = [x1, y1];
                }
                path.push(PathPoint {
                    anchor: [x3, y3],
                    left_ctrl: [x2, y2],
                    right_ctrl: [x3, y3],
                });
            }
            "re" if stack.len() >= 4 => {
                let x = stack[stack.len() - 4];
                let y = stack[stack.len() - 3];
                let w = stack[stack.len() - 2];
                let h = stack[stack.len() - 1];
                let p1 = transform_pdf_point(state.ctm, x, y);
                let p2 = transform_pdf_point(state.ctm, x + w, y);
                let p3 = transform_pdf_point(state.ctm, x + w, y + h);
                let p4 = transform_pdf_point(state.ctm, x, y + h);
                path = [p1, p2, p3, p4]
                    .into_iter()
                    .map(|p| PathPoint {
                        anchor: p,
                        left_ctrl: p,
                        right_ctrl: p,
                    })
                    .collect();
                closed = true;
            }
            "h" => closed = true,
            "n" => {
                path.clear();
                closed = false;
            }
            "f" | "F" | "f*" => {
                if let Some(element) = painted_path_element(
                    stream_idx,
                    elements.len(),
                    &path,
                    true,
                    &state,
                    true,
                    false,
                ) {
                    elements.push(element);
                }
                path.clear();
                closed = false;
            }
            "S" => {
                if let Some(element) = painted_path_element(
                    stream_idx,
                    elements.len(),
                    &path,
                    closed,
                    &state,
                    false,
                    true,
                ) {
                    elements.push(element);
                }
                path.clear();
                closed = false;
            }
            "s" => {
                if let Some(element) = painted_path_element(
                    stream_idx,
                    elements.len(),
                    &path,
                    true,
                    &state,
                    false,
                    true,
                ) {
                    elements.push(element);
                }
                path.clear();
                closed = false;
            }
            "B" | "B*" | "b" | "b*" => {
                if let Some(element) = painted_path_element(
                    stream_idx,
                    elements.len(),
                    &path,
                    true,
                    &state,
                    true,
                    true,
                ) {
                    elements.push(element);
                }
                path.clear();
                closed = false;
            }
            _ => {}
        }

        if !matches!(token, "q" | "Q") {
            stack.clear();
        }
    }

    elements
}

/// Parse PostScript path geometry from a content stream.
/// Returns (path_points, is_closed).
pub(crate) fn parse_path_geometry(content: &str) -> (Vec<PathPoint>, bool) {
    let mut points: Vec<PathPoint> = Vec::new();
    let mut closed = false;

    // Match PostScript path operators: m (moveto), l (lineto), c (curveto), h/z (closepath)
    // Word boundary \b ensures single-letter operators are not matched inside identifiers.
    let token_re = match Regex::new(r"(-?\d+\.?\d*(?:[eE][+-]?\d+)?)|\b([mlcCLMhHzZfFbBsS])\b") {
        Ok(re) => re,
        Err(_) => return (points, closed),
    };

    let mut tokens: Vec<String> = token_re
        .find_iter(content)
        .map(|m| m.as_str().to_string())
        .collect();
    tokens.reverse();
    let mut stack: Vec<f64> = Vec::new();

    while let Some(tok) = tokens.pop() {
        if let Ok(n) = tok.parse::<f64>() {
            stack.push(n);
        } else {
            match tok.as_str() {
                "m" | "M" => {
                    if stack.len() >= 2 {
                        let y = stack.pop().unwrap();
                        let x = stack.pop().unwrap();
                        points.push(PathPoint {
                            anchor: [x, y],
                            left_ctrl: [x, y],
                            right_ctrl: [x, y],
                        });
                    }
                    stack.clear();
                }
                "l" | "L" => {
                    if stack.len() >= 2 {
                        let y = stack.pop().unwrap();
                        let x = stack.pop().unwrap();
                        points.push(PathPoint {
                            anchor: [x, y],
                            left_ctrl: [x, y],
                            right_ctrl: [x, y],
                        });
                    }
                    stack.clear();
                }
                "c" | "C" => {
                    // curveto: x1 y1 x2 y2 x3 y3 c
                    if stack.len() >= 6 {
                        let y3 = stack.pop().unwrap();
                        let x3 = stack.pop().unwrap();
                        let y2 = stack.pop().unwrap();
                        let x2 = stack.pop().unwrap();
                        let y1 = stack.pop().unwrap();
                        let x1 = stack.pop().unwrap();
                        // Update the previous point's right control handle
                        if let Some(prev) = points.last_mut() {
                            prev.right_ctrl = [x1, y1];
                        }
                        points.push(PathPoint {
                            anchor: [x3, y3],
                            left_ctrl: [x2, y2],
                            right_ctrl: [x3, y3],
                        });
                    }
                    stack.clear();
                }
                "h" | "H" | "z" | "Z" => {
                    closed = true;
                    stack.clear();
                }
                _ => {
                    stack.clear();
                }
            }
        }
    }
    (points, closed)
}

/// Detect corner radius from an 8-point rounded rectangle Bezier path.
/// Returns the radius in document units, or 0.0 if not a rounded rect.
pub(crate) fn detect_corner_radius(points: &[PathPoint]) -> f64 {
    // A rounded rect has exactly 8 anchor points
    if points.len() != 8 {
        return 0.0;
    }
    // The cubic Bezier approximation constant for a quarter circle
    const KAPPA: f64 = 0.5522847498;
    let mut radii = Vec::new();
    for pt in points {
        let dx_left = pt.anchor[0] - pt.left_ctrl[0];
        let dy_left = pt.anchor[1] - pt.left_ctrl[1];
        let dx_right = pt.right_ctrl[0] - pt.anchor[0];
        let dy_right = pt.right_ctrl[1] - pt.anchor[1];
        let handle_left = (dx_left * dx_left + dy_left * dy_left).sqrt();
        let handle_right = (dx_right * dx_right + dy_right * dy_right).sqrt();
        let handle = handle_left.max(handle_right);
        if handle > 0.001 {
            radii.push(handle / KAPPA);
        }
    }
    if radii.is_empty() {
        return 0.0;
    }
    let mean = radii.iter().sum::<f64>() / radii.len() as f64;
    // Check consistency: all radii within 5% of mean
    let consistent = radii
        .iter()
        .all(|&r| (r - mean).abs() / mean.max(0.001) < 0.05);
    if consistent {
        mean
    } else {
        0.0
    }
}
