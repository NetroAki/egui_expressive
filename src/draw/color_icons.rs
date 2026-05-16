pub(crate) fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) * 0.5;
    if (max - min).abs() < 1e-6 {
        return (0.0, 0.0, l);
    }
    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if (max - r).abs() < 1e-6 {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if (max - g).abs() < 1e-6 {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    (h * 60.0, s, l)
}

/// Convert HSL (hue 0.0–360.0, saturation 0.0–1.0, lightness 0.0–1.0) to RGB (0.0–1.0).
pub(crate) fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s < 1e-6 {
        return (l, l, l);
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let hue_to_rgb = |p: f32, q: f32, mut t: f32| -> f32 {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            return p + (q - p) * 6.0 * t;
        }
        if t < 0.5 {
            return q;
        }
        if t < 2.0 / 3.0 {
            return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
        }
        p
    };
    let h = h / 360.0;
    (
        hue_to_rgb(p, q, h + 1.0 / 3.0),
        hue_to_rgb(p, q, h),
        hue_to_rgb(p, q, h - 1.0 / 3.0),
    )
}

/// Blend two colors using the specified blend mode.
pub fn blend_color(
    fg: egui::Color32,
    bg: egui::Color32,
    mode: crate::codegen::BlendMode,
) -> egui::Color32 {
    // Unpack as straight (unmultiplied) RGBA so blend math operates on true color values.
    // Color32 stores premultiplied bytes; to_srgba_unmultiplied() reverses that.
    let fg_arr = fg.to_srgba_unmultiplied();
    let bg_arr = bg.to_srgba_unmultiplied();
    let fg = (
        fg_arr[0] as f32 / 255.0,
        fg_arr[1] as f32 / 255.0,
        fg_arr[2] as f32 / 255.0,
        fg_arr[3] as f32 / 255.0,
    );
    let bg = (
        bg_arr[0] as f32 / 255.0,
        bg_arr[1] as f32 / 255.0,
        bg_arr[2] as f32 / 255.0,
        bg_arr[3] as f32 / 255.0,
    );

    let (r, g, b) = match mode {
        crate::codegen::BlendMode::Normal => (fg.0, fg.1, fg.2),
        crate::codegen::BlendMode::Multiply => (bg.0 * fg.0, bg.1 * fg.1, bg.2 * fg.2),
        crate::codegen::BlendMode::Screen => (
            1.0 - (1.0 - bg.0) * (1.0 - fg.0),
            1.0 - (1.0 - bg.1) * (1.0 - fg.1),
            1.0 - (1.0 - bg.2) * (1.0 - fg.2),
        ),
        crate::codegen::BlendMode::Overlay => {
            let blend = |bg: f32, fg: f32| {
                if bg < 0.5 {
                    2.0 * bg * fg
                } else {
                    1.0 - 2.0 * (1.0 - bg) * (1.0 - fg)
                }
            };
            (blend(bg.0, fg.0), blend(bg.1, fg.1), blend(bg.2, fg.2))
        }
        crate::codegen::BlendMode::Darken => (bg.0.min(fg.0), bg.1.min(fg.1), bg.2.min(fg.2)),
        crate::codegen::BlendMode::Lighten => (bg.0.max(fg.0), bg.1.max(fg.1), bg.2.max(fg.2)),
        // Advanced blend modes
        crate::codegen::BlendMode::ColorDodge => (
            if fg.0 >= 1.0 {
                1.0
            } else {
                (bg.0 / (1.0 - fg.0)).min(1.0)
            },
            if fg.1 >= 1.0 {
                1.0
            } else {
                (bg.1 / (1.0 - fg.1)).min(1.0)
            },
            if fg.2 >= 1.0 {
                1.0
            } else {
                (bg.2 / (1.0 - fg.2)).min(1.0)
            },
        ),
        crate::codegen::BlendMode::ColorBurn => (
            if fg.0 <= 0.0 {
                0.0
            } else {
                1.0 - ((1.0 - bg.0) / fg.0).min(1.0)
            },
            if fg.1 <= 0.0 {
                0.0
            } else {
                1.0 - ((1.0 - bg.1) / fg.1).min(1.0)
            },
            if fg.2 <= 0.0 {
                0.0
            } else {
                1.0 - ((1.0 - bg.2) / fg.2).min(1.0)
            },
        ),
        crate::codegen::BlendMode::HardLight => {
            // HardLight = Overlay with fg and bg swapped
            let blend = |fg: f32, bg: f32| {
                if fg < 0.5 {
                    2.0 * fg * bg
                } else {
                    1.0 - 2.0 * (1.0 - fg) * (1.0 - bg)
                }
            };
            (blend(fg.0, bg.0), blend(fg.1, bg.1), blend(fg.2, bg.2))
        }
        crate::codegen::BlendMode::SoftLight => {
            // W3C SoftLight formula
            let blend = |bg: f32, fg: f32| {
                if fg <= 0.5 {
                    bg - (1.0 - 2.0 * fg) * bg * (1.0 - bg)
                } else {
                    let d = if bg <= 0.25 {
                        ((16.0 * bg - 12.0) * bg + 4.0) * bg
                    } else {
                        bg.sqrt()
                    };
                    bg + (2.0 * fg - 1.0) * (d - bg)
                }
            };
            (blend(bg.0, fg.0), blend(bg.1, fg.1), blend(bg.2, fg.2))
        }
        crate::codegen::BlendMode::Difference => (
            (bg.0 - fg.0).abs(),
            (bg.1 - fg.1).abs(),
            (bg.2 - fg.2).abs(),
        ),
        crate::codegen::BlendMode::Exclusion => (
            bg.0 + fg.0 - 2.0 * bg.0 * fg.0,
            bg.1 + fg.1 - 2.0 * bg.1 * fg.1,
            bg.2 + fg.2 - 2.0 * bg.2 * fg.2,
        ),
        crate::codegen::BlendMode::Hue => {
            // Set hue of bg to hue of fg, keep bg saturation and luminosity
            let (fh, _fs, _fl) = rgb_to_hsl(fg.0, fg.1, fg.2);
            let (_bh, bs, bl) = rgb_to_hsl(bg.0, bg.1, bg.2);
            hsl_to_rgb(fh, bs, bl)
        }
        crate::codegen::BlendMode::Saturation => {
            // Set saturation of bg to saturation of fg, keep bg hue and luminosity
            let (_fh, fs, _fl) = rgb_to_hsl(fg.0, fg.1, fg.2);
            let (bh, _bs, bl) = rgb_to_hsl(bg.0, bg.1, bg.2);
            hsl_to_rgb(bh, fs, bl)
        }
        crate::codegen::BlendMode::Color => {
            // Set hue+saturation of bg to fg, keep bg luminosity
            let (fh, fs, _fl) = rgb_to_hsl(fg.0, fg.1, fg.2);
            let (_bh, _bs, bl) = rgb_to_hsl(bg.0, bg.1, bg.2);
            hsl_to_rgb(fh, fs, bl)
        }
        crate::codegen::BlendMode::Luminosity => {
            // Set luminosity of bg to luminosity of fg, keep bg hue+saturation
            let (_fh, _fs, fl) = rgb_to_hsl(fg.0, fg.1, fg.2);
            let (bh, bs, _bl) = rgb_to_hsl(bg.0, bg.1, bg.2);
            hsl_to_rgb(bh, bs, fl)
        }
    };

    // Full W3C Porter-Duff "source over" compositing in straight-alpha space:
    //   co = cs·αs·(1−αb) + αs·αb·B(cb,cs) + cb·αb·(1−αs)
    // where B(cb,cs) = r/g/b from the blend mode above.
    let out_a = fg.3 + bg.3 * (1.0 - fg.3);
    let (r, g, b) = if out_a > 1e-6 {
        let compose = |cs: f32, blend: f32, cb: f32| {
            (cs * fg.3 * (1.0 - bg.3) + fg.3 * bg.3 * blend + cb * bg.3 * (1.0 - fg.3)) / out_a
        };
        (
            compose(fg.0, r, bg.0),
            compose(fg.1, g, bg.1),
            compose(fg.2, b, bg.2),
        )
    } else {
        (0.0, 0.0, 0.0)
    };

    // Convert back to u8
    let r = (r.clamp(0.0, 1.0) * 255.0) as u8;
    let g = (g.clamp(0.0, 1.0) * 255.0) as u8;
    let b = (b.clamp(0.0, 1.0) * 255.0) as u8;
    let a = (out_a.clamp(0.0, 1.0) * 255.0) as u8;

    egui::Color32::from_rgba_unmultiplied(r, g, b, a)
}

// ─── Icon Rendering ───────────────────────────────────────────────────────────

/// Render a single glyph from an icon font (e.g., Phosphor Icons) at `pos`.
///
/// # Usage
/// 1. Load your icon font via `egui::FontDefinitions` and give it a family name.
/// 2. Call `icon(painter, pos, '\u{E000}', 16.0, color, "PhosphorIcons")`.
pub fn icon(
    painter: &egui::Painter,
    pos: egui::Pos2,
    codepoint: char,
    size: f32,
    color: egui::Color32,
    font_family: &str,
) {
    let font_id = egui::FontId::new(size, egui::FontFamily::Name(font_family.into()));
    painter.text(
        pos,
        egui::Align2::CENTER_CENTER,
        codepoint.to_string(),
        font_id,
        color,
    );
}

/// Render a Phosphor-style icon using a built-in path approximation.
/// This works without loading an icon font — uses PathBuilder to draw common shapes.
pub fn icon_play(painter: &egui::Painter, center: egui::Pos2, size: f32, color: egui::Color32) {
    let r = size * 0.4;
    let points = vec![
        egui::Pos2::new(center.x - r * 0.5, center.y - r),
        egui::Pos2::new(center.x + r, center.y),
        egui::Pos2::new(center.x - r * 0.5, center.y + r),
    ];
    painter.add(egui::Shape::convex_polygon(
        points,
        color,
        egui::Stroke::NONE,
    ));
}

pub fn icon_stop(painter: &egui::Painter, center: egui::Pos2, size: f32, color: egui::Color32) {
    let r = size * 0.35;
    let rect = egui::Rect::from_center_size(center, egui::Vec2::splat(r * 2.0));
    painter.add(egui::Shape::Rect(egui::epaint::RectShape::filled(
        rect,
        egui::CornerRadius::ZERO,
        color,
    )));
}

pub fn icon_record(painter: &egui::Painter, center: egui::Pos2, size: f32, color: egui::Color32) {
    painter.circle_filled(center, size * 0.35, color);
}

pub fn icon_loop(painter: &egui::Painter, center: egui::Pos2, size: f32, color: egui::Color32) {
    // Two arrows forming a loop — simplified as two arcs
    let r = size * 0.35;
    let stroke = egui::Stroke::new(size * 0.1, color);
    painter.circle_stroke(center, r, stroke);
}

// ─── Radial Gradient ─────────────────────────────────────────────────────────

/// Direction for radial gradient — center-out or outside-in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RadialGradientDir {
    /// Color at center, fades to edge color.
    CenterOut,
    /// Color at edge, fades to center color.
    EdgeIn,
}

/// Render a radial gradient as a `Shape::Mesh`.
///
/// Approximates a radial gradient using a triangle fan from the center.
/// `segments` controls smoothness (32 is good, 64 is high quality).
pub fn radial_gradient(
    center: egui::Pos2,
    radius: f32,
    inner_color: egui::Color32,
    outer_color: egui::Color32,
    segments: u32,
) -> egui::Shape {
    use egui::{epaint::Mesh, Vec2};
    let mut mesh = Mesh::default();

    // Center vertex
    mesh.colored_vertex(center, inner_color);

    // Ring vertices
    let n = segments.max(8);
    for i in 0..=n {
        let angle = (i as f32 / n as f32) * std::f32::consts::TAU;
        let pos = center + Vec2::new(angle.cos(), angle.sin()) * radius;
        mesh.colored_vertex(pos, outer_color);
    }

    // Triangles: center (0) + consecutive ring pairs
    for i in 0..n {
        mesh.add_triangle(0, i + 1, i + 2);
    }

    egui::Shape::Mesh(std::sync::Arc::new(mesh))
}

/// Radial gradient clipped to a rectangle (elliptical).
pub fn radial_gradient_rect(
    rect: egui::Rect,
    inner_color: egui::Color32,
    outer_color: egui::Color32,
    segments: u32,
) -> egui::Shape {
    let center = rect.center();
    let rx = rect.width() * 0.5;
    let ry = rect.height() * 0.5;
    use egui::epaint::Mesh;
    let mut mesh = Mesh::default();

    mesh.colored_vertex(center, inner_color);

    let n = segments.max(8);
    for i in 0..=n {
        let angle = (i as f32 / n as f32) * std::f32::consts::TAU;
        let pos = center + egui::Vec2::new(angle.cos() * rx, angle.sin() * ry);
        mesh.colored_vertex(pos, outer_color);
    }

    for i in 0..n {
        mesh.add_triangle(0, i + 1, i + 2);
    }

    egui::Shape::Mesh(std::sync::Arc::new(mesh))
}

/// Multi-stop radial gradient clipped to a rectangle (elliptical).
///
/// Unlike [`radial_gradient_rect`], this preserves all Illustrator radial-gradient stops by
/// emitting concentric mesh rings. Stop positions are clamped to `0.0..=1.0`; missing stops produce
/// [`egui::Shape::Noop`].
pub fn radial_gradient_rect_stops(
    rect: egui::Rect,
    stops: &[(f32, egui::Color32)],
    segments: u32,
) -> egui::Shape {
    use egui::epaint::Mesh;

    if stops.is_empty() {
        return egui::Shape::Noop;
    }

    let mut stops = stops.to_vec();
    stops.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    for (pos, _) in &mut stops {
        *pos = pos.clamp(0.0, 1.0);
    }

    let ring_count = stops.len().max(2);
    let segments = segments.max(8);
    let center = rect.center();
    let rx = rect.width() * 0.5;
    let ry = rect.height() * 0.5;
    let mut mesh = Mesh::default();

    for ring in 0..ring_count {
        let t = if ring_count == 1 {
            0.0
        } else {
            ring as f32 / (ring_count - 1) as f32
        };
        let color = sample_stops(&stops, t);

        if ring == 0 {
            mesh.colored_vertex(center, color);
        } else {
            for i in 0..=segments {
                let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
                mesh.colored_vertex(
                    center + egui::vec2(angle.cos() * rx * t, angle.sin() * ry * t),
                    color,
                );
            }
        }
    }

    // Center fan.
    for i in 0..segments {
        mesh.add_triangle(0, i + 1, i + 2);
    }

    // Ring strips.
    let ring_stride = segments + 1;
    for ring in 1..(ring_count - 1) as u32 {
        let inner_start = 1 + (ring - 1) * ring_stride;
        let outer_start = 1 + ring * ring_stride;
        for i in 0..segments {
            let a = inner_start + i;
            let b = inner_start + i + 1;
            let c = outer_start + i;
            let d = outer_start + i + 1;
            mesh.add_triangle(a, b, c);
            mesh.add_triangle(b, d, c);
        }
    }

    egui::Shape::Mesh(std::sync::Arc::new(mesh))
}

pub(crate) fn sample_stops(stops: &[(f32, egui::Color32)], t: f32) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    if stops.len() == 1 || t <= stops[0].0 {
        return stops[0].1;
    }
    for pair in stops.windows(2) {
        let (a_t, a) = pair[0];
        let (b_t, b) = pair[1];
        if t <= b_t {
            let local = if (b_t - a_t).abs() < f32::EPSILON {
                0.0
            } else {
                (t - a_t) / (b_t - a_t)
            };
            return lerp_color(a, b, local);
        }
    }
    stops
        .last()
        .map(|(_, c)| *c)
        .unwrap_or(egui::Color32::TRANSPARENT)
}

pub(crate) fn lerp_color(a: egui::Color32, b: egui::Color32, t: f32) -> egui::Color32 {
    let a = a.to_srgba_unmultiplied();
    let b = b.to_srgba_unmultiplied();
    let channel = |idx: usize| (a[idx] as f32 + (b[idx] as f32 - a[idx] as f32) * t).round() as u8;
    egui::Color32::from_rgba_unmultiplied(channel(0), channel(1), channel(2), channel(3))
}

// ─── Scan Lines & Overlays ───────────────────────────────────────────────────

/// Render a CRT-style scan line overlay over a rect.
///
/// Draws alternating semi-transparent horizontal lines.
/// `line_height` is the height of each scan line pair (default 2.0).
/// `alpha` controls darkness (0.0 = invisible, 1.0 = fully black lines).
pub fn scan_lines(rect: egui::Rect, line_height: f32, alpha: f32) -> Vec<egui::Shape> {
    let color = egui::Color32::from_black_alpha((alpha * 80.0).clamp(0.0, 255.0) as u8);
    let lh = line_height.max(1.0);
    let mut shapes = Vec::new();
    let mut y = rect.min.y;
    while y < rect.max.y {
        let line_rect = egui::Rect::from_min_max(
            egui::Pos2::new(rect.min.x, y),
            egui::Pos2::new(rect.max.x, (y + lh * 0.5).min(rect.max.y)),
        );
        shapes.push(egui::Shape::rect_filled(line_rect, 0.0, color));
        y += lh;
    }
    shapes
}

/// Render a dot-matrix / halftone overlay over a rect.
///
/// Draws a grid of small semi-transparent dots.
pub fn dot_matrix(
    rect: egui::Rect,
    dot_spacing: f32,
    dot_radius: f32,
    color: egui::Color32,
) -> Vec<egui::Shape> {
    let spacing = dot_spacing.max(2.0);
    let mut shapes = Vec::new();
    let mut y = rect.min.y + spacing * 0.5;
    while y < rect.max.y {
        let mut x = rect.min.x + spacing * 0.5;
        while x < rect.max.x {
            shapes.push(egui::Shape::circle_filled(
                egui::Pos2::new(x, y),
                dot_radius,
                color,
            ));
            x += spacing;
        }
        y += spacing;
    }
    shapes
}

/// Render a vignette effect (dark edges, bright center) over a rect.
pub fn vignette(rect: egui::Rect, color: egui::Color32, strength: f32) -> egui::Shape {
    // Approximate with a radial gradient from transparent center to colored edge
    let alpha = (strength * 200.0).clamp(0.0, 255.0) as u8;
    let edge_color = egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha);
    radial_gradient_rect(rect, egui::Color32::TRANSPARENT, edge_color, 48)
}

// ─── Rich Stroke & Dashed Paths ───────────────────────────────────────────────
