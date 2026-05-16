use super::*;

#[cfg(feature = "wgpu")]
pub(crate) fn pixels_to_rgba(pixels: &[egui::Color32]) -> Vec<u8> {
    pixels
        .iter()
        .flat_map(|p| p.to_srgba_unmultiplied())
        .collect()
}

pub(crate) fn blend_layers_hash(layers: &[BlendLayer], pixels: &[egui::Color32]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    layers.len().hash(&mut hasher);
    for layer in layers {
        layer.blend_mode.hash(&mut hasher);
        layer.opacity.to_bits().hash(&mut hasher);
        layer.shapes.len().hash(&mut hasher);
        layer.clip_polygons.len().hash(&mut hasher);
        for polygon in &layer.clip_polygons {
            for point in polygon {
                point.x.to_bits().hash(&mut hasher);
                point.y.to_bits().hash(&mut hasher);
            }
        }
    }
    for p in pixels {
        p.hash(&mut hasher);
    }
    hasher.finish()
}

pub(crate) fn polygon_hash(points: &[egui::Pos2]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for p in points {
        p.x.to_bits().hash(&mut hasher);
        p.y.to_bits().hash(&mut hasher);
    }
    hasher.finish()
}

pub(crate) fn clip_mask_hash(mask: &ClipMask) -> u64 {
    use std::hash::Hasher;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    mask.hash_into(&mut hasher);
    hasher.finish()
}
