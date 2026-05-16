use super::geometry::point_in_polygon;
use egui::*;

pub(crate) fn apply_polygon_alpha_mask(
    pixels: &mut [Color32],
    width: u32,
    height: u32,
    origin: Pos2,
    polygon: &[Pos2],
) {
    for y in 0..height {
        for x in 0..width {
            let p = pos2(origin.x + x as f32 + 0.5, origin.y + y as f32 + 0.5);
            if !point_in_polygon(p, polygon) {
                pixels[(y * width + x) as usize] = Color32::TRANSPARENT;
            }
        }
    }
}

pub(crate) fn apply_clip_mask(
    pixels: &mut [Color32],
    width: u32,
    height: u32,
    origin: Pos2,
    mask: &super::layout::ClipMask,
) {
    for y in 0..height {
        for x in 0..width {
            let p = pos2(origin.x + x as f32 + 0.5, origin.y + y as f32 + 0.5);
            if !mask.contains(p) {
                pixels[(y * width + x) as usize] = Color32::TRANSPARENT;
            }
        }
    }
}

pub(crate) fn color_with_opacity(color: Color32, opacity: f32) -> Color32 {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    Color32::from_rgba_unmultiplied(r, g, b, (a as f32 * opacity.clamp(0.0, 1.0)).round() as u8)
}
