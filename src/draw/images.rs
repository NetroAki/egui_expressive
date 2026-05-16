use egui::epaint::*;
use egui::*;

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
