/// Paint content clipped to a convex polygon using a bounding-box scissor approximation.
pub fn clipped_shape_approx(
    ui: &mut egui::Ui,
    clip_polygon: &[egui::Pos2],
    _clip_closed: bool,
    content: impl FnOnce(&mut egui::Ui),
) {
    if clip_polygon.len() < 3 {
        // Degenerate: just paint without clipping
        content(ui);
        return;
    }
    // Compute bounding box of the clip polygon
    let min_x = clip_polygon
        .iter()
        .map(|p| p.x)
        .fold(f32::INFINITY, f32::min);
    let min_y = clip_polygon
        .iter()
        .map(|p| p.y)
        .fold(f32::INFINITY, f32::min);
    let max_x = clip_polygon
        .iter()
        .map(|p| p.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let max_y = clip_polygon
        .iter()
        .map(|p| p.y)
        .fold(f32::NEG_INFINITY, f32::max);

    if min_x >= max_x || min_y >= max_y {
        return;
    }

    let clip_rect = egui::Rect::from_min_max(egui::pos2(min_x, min_y), egui::pos2(max_x, max_y));

    // Use rectangular scissor for the bounding box
    let painter = ui.painter().with_clip_rect(clip_rect);
    let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(clip_rect));
    child_ui.set_clip_rect(clip_rect);

    // Paint the content (clipped to bbox of polygon)
    content(&mut child_ui);

    // Paint corner-covering triangles in the background color to approximate
    // the polygon clip for convex shapes (e.g. rounded rects, hexagons)
    let bg = ui.visuals().window_fill();
    let n = clip_polygon.len();
    for i in 0..n {
        let a = clip_polygon[i];
        let b = clip_polygon[(i + 1) % n];
        // For each edge, paint the "outside" triangle between the edge and the bbox corner
        // This is a best-effort approximation for convex polygons
        let corner = nearest_bbox_corner(a, b, clip_rect);
        if let Some(corner_pt) = corner {
            painter.add(egui::Shape::convex_polygon(
                vec![a, b, corner_pt],
                bg,
                egui::Stroke::NONE,
            ));
        }
    }
}

pub fn clipped_shape(
    ui: &mut egui::Ui,
    clip_polygon: &[egui::Pos2],
    clip_closed: bool,
    content: impl FnOnce(&mut egui::Ui),
) {
    clipped_shape_approx(ui, clip_polygon, clip_closed, content)
}

/// Paint content clipped to a polygon-shaped region using a background-dependent alpha-mask approximation.
///
/// Requires the `clip-mask` feature flag (`tiny-skia` dependency).
/// When `clip-mask` is not enabled, this function is not compiled; use
/// [`clipped_shape_approx`] for the default bbox approximation path.
///
/// # Limitations
/// This is **not** true arbitrary clipping. It works by painting an inverted mask overlay
/// using the current `ui.visuals().window_fill()` color. It will look incorrect if the
/// background behind the clipped shape is not a solid color matching `window_fill()`,
/// or if placed over layered/non-flat scenes.
///
/// # How it works
/// 1. Builds a tiny-skia alpha mask from the clip polygon
/// 2. Clips content to the bounding box via `set_clip_rect`
/// 3. Overlays the inverted mask (in background color) to hide regions outside the polygon
#[cfg(feature = "clip-mask")]
pub fn clipped_shape_cpu(
    ui: &mut egui::Ui,
    clip_polygon: &[egui::Pos2],
    content: impl FnOnce(&mut egui::Ui),
) {
    use tiny_skia::{FillRule, Paint as SkPaint, PathBuilder, Pixmap, Transform as SkTransform};

    if clip_polygon.len() < 3 {
        content(ui);
        return;
    }

    let min_x = clip_polygon
        .iter()
        .map(|p| p.x)
        .fold(f32::INFINITY, f32::min);
    let min_y = clip_polygon
        .iter()
        .map(|p| p.y)
        .fold(f32::INFINITY, f32::min);
    let max_x = clip_polygon
        .iter()
        .map(|p| p.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let max_y = clip_polygon
        .iter()
        .map(|p| p.y)
        .fold(f32::NEG_INFINITY, f32::max);

    if min_x >= max_x || min_y >= max_y {
        return;
    }

    let w = (max_x - min_x).ceil() as u32;
    let h = (max_y - min_y).ceil() as u32;
    if w == 0 || h == 0 {
        return;
    }

    // Build tiny-skia mask pixmap
    let mut mask_pixmap = match Pixmap::new(w, h) {
        Some(p) => p,
        None => {
            clipped_shape_approx(ui, clip_polygon, true, content);
            return;
        }
    };

    let mut pb = PathBuilder::new();
    let first = clip_polygon[0];
    pb.move_to(first.x - min_x, first.y - min_y);
    for pt in &clip_polygon[1..] {
        pb.line_to(pt.x - min_x, pt.y - min_y);
    }
    pb.close();

    if let Some(path) = pb.finish() {
        let mut paint = SkPaint::default();
        paint.set_color_rgba8(255, 255, 255, 255);
        mask_pixmap.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            SkTransform::identity(),
            None,
        );
    }

    // Paint content clipped to bbox
    let clip_rect = egui::Rect::from_min_max(egui::pos2(min_x, min_y), egui::pos2(max_x, max_y));
    let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(clip_rect));
    child_ui.set_clip_rect(clip_rect);
    content(&mut child_ui);

    // Build inverted mask overlay: background color where polygon is absent
    let bg = ui.visuals().window_fill();
    let mask_pixels: Vec<egui::Color32> = mask_pixmap
        .pixels()
        .iter()
        .map(|px| {
            let a = px.alpha();
            egui::Color32::from_rgba_unmultiplied(bg.r(), bg.g(), bg.b(), 255u8.saturating_sub(a))
        })
        .collect();

    let mask_image = egui::ColorImage {
        size: [w as usize, h as usize],
        pixels: mask_pixels,
        source_size: egui::Vec2::new(w as f32, h as f32),
    };

    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for p in clip_polygon {
        p.x.to_bits().hash(&mut hasher);
        p.y.to_bits().hash(&mut hasher);
    }
    let hash = hasher.finish();
    let texture_name = format!("__egui_expressive_clip_mask_{:x}_{}x{}", hash, w, h);

    let texture = ui
        .ctx()
        .load_texture(texture_name, mask_image, egui::TextureOptions::NEAREST);

    let painter = ui.painter().with_clip_rect(clip_rect);
    painter.image(
        texture.id(),
        clip_rect,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );
}

/// Find the nearest bounding box corner to the midpoint of edge (a, b),
/// on the outside of the polygon. Returns None if the edge is axis-aligned
/// (no corner masking needed).
pub(crate) fn nearest_bbox_corner(
    a: egui::Pos2,
    b: egui::Pos2,
    bbox: egui::Rect,
) -> Option<egui::Pos2> {
    let mid = egui::pos2((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
    // Determine which bbox corner is farthest from the midpoint (outside the polygon)
    let corners = [
        bbox.min,
        egui::pos2(bbox.max.x, bbox.min.y),
        bbox.max,
        egui::pos2(bbox.min.x, bbox.max.y),
    ];
    // Only return a corner if it's clearly outside the edge
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    // Edge normal (pointing outward for CW polygon)
    let nx = dy;
    let ny = -dx;
    corners
        .iter()
        .filter(|&&c| {
            // Corner is on the outside (positive normal side)
            let dot = (c.x - mid.x) * nx + (c.y - mid.y) * ny;
            dot > 1.0
        })
        .copied()
        .next()
}
