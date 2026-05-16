use super::*;

/// Approximate a CSS box-shadow with multiple semi-transparent rects.
pub fn box_shadow(
    rect: egui::Rect,
    color: egui::Color32,
    blur_radius: f32,
    spread: f32,
    offset: ShadowOffset,
) -> Vec<egui::Shape> {
    let steps = (blur_radius.ceil() as usize).clamp(1, 12);
    let mut shapes = Vec::with_capacity(steps);
    let base_alpha = color.a() as f32 / steps as f32;

    for i in 0..steps {
        let t = i as f32 / steps as f32;
        let expansion = spread + blur_radius * t;
        let alpha = (base_alpha * (1.0 - t * 0.5)) as u8;
        let shadow_color =
            egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha);
        let shadow_rect = egui::Rect::from_min_max(
            egui::Pos2::new(
                rect.min.x - expansion + offset.x,
                rect.min.y - expansion + offset.y,
            ),
            egui::Pos2::new(
                rect.max.x + expansion + offset.x,
                rect.max.y + expansion + offset.y,
            ),
        );
        let rounding = egui::CornerRadius::same((expansion * 0.5).round() as u8);
        shapes.push(egui::Shape::Rect(egui::epaint::RectShape::filled(
            shadow_rect,
            rounding,
            shadow_color,
        )));
    }
    shapes
}

/// Symmetric glow around a rect (no offset, equal spread on all sides).
pub fn glow(rect: egui::Rect, color: egui::Color32, radius: f32) -> Vec<egui::Shape> {
    box_shadow(rect, color, radius, 0.0, ShadowOffset::zero())
}

/// Inner shadow (inset) approximated by drawing a border with gradient-like alpha.
pub fn inner_shadow(rect: egui::Rect, color: egui::Color32, blur_radius: f32) -> Vec<egui::Shape> {
    let steps = (blur_radius.ceil() as usize).clamp(1, 8);
    let mut shapes = Vec::with_capacity(steps * 4);
    let base_alpha = color.a() as f32 / steps as f32;

    for i in 0..steps {
        let t = i as f32 / steps as f32;
        let inset = blur_radius * t;
        let alpha = (base_alpha * (1.0 - t)) as u8;
        let c = egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha);
        let stroke = egui::Stroke::new(1.0, c);
        let inner = egui::Rect::from_min_max(
            egui::Pos2::new(rect.min.x + inset, rect.min.y + inset),
            egui::Pos2::new(rect.max.x - inset, rect.max.y - inset),
        );
        if inner.width() > 0.0 && inner.height() > 0.0 {
            shapes.push(egui::Shape::Rect(egui::epaint::RectShape::stroke(
                inner,
                egui::CornerRadius::ZERO,
                stroke,
                egui::epaint::StrokeKind::Inside,
            )));
        }
    }
    shapes
}

/// Load an image file at runtime and paint it into `rect`.
///
/// This is intended for generated Illustrator preview code where linked raster
/// assets are known only at export time. It returns `false` when the file cannot
/// be read or decoded so generated code can draw a visible fallback instead.
pub fn paint_image_from_path(
    ui: &egui::Ui,
    painter: &egui::Painter,
    rect: egui::Rect,
    path: &str,
    texture_id: &str,
    tint: egui::Color32,
) -> bool {
    let cache_id = egui::Id::new(("egui_expressive_image_texture", texture_id, path));
    let texture = if let Some(texture) = ui
        .ctx()
        .data(|data| data.get_temp::<egui::TextureHandle>(cache_id))
    {
        texture
    } else {
        let path_obj = std::path::Path::new(path);
        let mut bytes = std::fs::read(path_obj);
        if bytes.is_err() {
            if let Some(file_name) = path_obj.file_name() {
                bytes = std::fs::read(file_name);
                if bytes.is_err() {
                    bytes = std::fs::read(std::path::Path::new("generated").join(file_name));
                }
                if bytes.is_err() {
                    bytes = std::fs::read(std::path::Path::new("assets").join(file_name));
                }
                if bytes.is_err() {
                    bytes = std::fs::read(
                        std::path::Path::new("generated")
                            .join("assets")
                            .join(file_name),
                    );
                }
            }
        }
        let Ok(bytes) = bytes else {
            return false;
        };
        let Ok(dynamic_image) = image::load_from_memory(&bytes) else {
            return false;
        };
        let rgba = dynamic_image.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
        let texture = ui
            .ctx()
            .load_texture(texture_id, color_image, egui::TextureOptions::LINEAR);
        ui.ctx()
            .data_mut(|data| data.insert_temp(cache_id, texture.clone()));
        texture
    };

    painter.image(
        texture.id(),
        rect,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        tint,
    );
    true
}

/// Paint a reusable placeholder slot for assets or Illustrator primitives that
/// are intentionally unavailable at runtime.
///
/// This keeps generated exporters and hand-authored egui_expressive code on the
/// same visible fallback primitive instead of duplicating ad-hoc red rectangles
/// in generated files.
pub fn paint_placeholder_slot(
    painter: &egui::Painter,
    rect: egui::Rect,
    fill: egui::Color32,
    stroke: egui::Stroke,
    label: impl AsRef<str>,
) {
    painter.rect_filled(rect, 0.0, fill);
    painter.rect_stroke(rect, 0.0, stroke, egui::StrokeKind::Outside);
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label.as_ref(),
        egui::FontId::proportional(12.0),
        stroke.color,
    );
}

/// Paint an optional image path and draw a shared placeholder when it cannot be
/// loaded.
///
/// Returns `true` when the image was decoded and painted, `false` when the
/// fallback slot was painted. Use this from generated Illustrator code and from
/// code-first egui_expressive UIs that accept user-provided assets.
pub fn paint_image_slot(
    ui: &egui::Ui,
    painter: &egui::Painter,
    rect: egui::Rect,
    path: Option<&str>,
    texture_id: &str,
    tint: egui::Color32,
    fallback_label: &str,
) -> bool {
    if let Some(path) = path.filter(|p| !p.trim().is_empty()) {
        if paint_image_from_path(ui, painter, rect, path, texture_id, tint) {
            return true;
        }
    }

    let alpha = tint.a();
    paint_placeholder_slot(
        painter,
        rect,
        egui::Color32::from_rgba_unmultiplied(255, 0, 0, (30_u16 * alpha as u16 / 255) as u8),
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 0, 0, alpha)),
        fallback_label,
    );
    false
}

// ─── Gradients ────────────────────────────────────────────────────────────────
