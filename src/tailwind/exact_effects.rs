//! WGPU-backed exact Tailwind effect helpers for narrow source-qualified subsets.

use std::hash::{Hash, Hasher};

use egui::{Color32, Rect, Shape};

use crate::render::{OffscreenRequest, RenderCapabilities, RenderFeature, RenderQuality};
use crate::tailwind::types::TwDropShadow;

const TAILWIND_EFFECT_AXIS_LIMIT: u32 = 4096;

#[derive(Clone, Copy, Debug)]
pub(crate) struct TwExactDropShadowInput {
    pub rect: Rect,
    pub fill: Option<Color32>,
    pub shadow: TwDropShadow,
    pub has_rounded_corners: bool,
    pub has_border: bool,
    pub has_ring: bool,
    pub has_gradient: bool,
    pub has_directional_border: bool,
    pub has_divide: bool,
}

pub(crate) fn exact_drop_shadow_shape(
    ctx: &egui::Context,
    input: TwExactDropShadowInput,
) -> Option<Shape> {
    let spec = exact_drop_shadow_spec(ctx, input)?;
    let rgba = source_rect_rgba(
        spec.size[0],
        spec.size[1],
        spec.output_rect,
        spec.source_rect,
        input.shadow.color,
    );
    let callback = egui_wgpu::Callback::new_paint_callback(
        spec.output_rect,
        crate::gpu::GpuSourceLayerEffectCallback::new_blur(
            spec.callback_id,
            spec.size,
            rgba,
            spec.radius,
        ),
    );
    Some(Shape::Callback(callback))
}

#[derive(Clone, Copy, Debug)]
struct TwExactDropShadowSpec {
    output_rect: Rect,
    source_rect: Rect,
    radius: f32,
    size: [u32; 2],
    callback_id: u64,
}

fn exact_drop_shadow_spec(
    ctx: &egui::Context,
    input: TwExactDropShadowInput,
) -> Option<TwExactDropShadowSpec> {
    let fill = input.fill?;
    if fill.a() != 255
        || input.has_rounded_corners
        || input.has_border
        || input.has_ring
        || input.has_gradient
        || input.has_directional_border
        || input.has_divide
        || !crate::gpu::gpu_effects_initialized_for_context(ctx)
    {
        return None;
    }

    let radius = input.shadow.blur as f32;
    if radius < 1.0 || !input.rect.is_finite() || !input.rect.is_positive() {
        return None;
    }

    let source_rect = input.rect.translate(input.shadow.offset);
    let output_rect = source_rect.expand(radius.ceil());
    if !source_rect.is_finite()
        || !source_rect.is_positive()
        || !output_rect.is_finite()
        || !output_rect.is_positive()
    {
        return None;
    }

    let width_px = output_rect.width().ceil();
    let height_px = output_rect.height().ceil();
    if !(1.0..=TAILWIND_EFFECT_AXIS_LIMIT as f32).contains(&width_px)
        || !(1.0..=TAILWIND_EFFECT_AXIS_LIMIT as f32).contains(&height_px)
    {
        return None;
    }
    let size = [width_px as u32, height_px as u32];
    let capabilities = RenderCapabilities::egui_wgpu_callback(
        u64::from(TAILWIND_EFFECT_AXIS_LIMIT) * u64::from(TAILWIND_EFFECT_AXIS_LIMIT),
    );
    let report = crate::gpu::wgpu_source_layer_effect_report(
        &capabilities,
        OffscreenRequest {
            feature: RenderFeature::Shadow,
            width: size[0],
            height: size[1],
            requested_quality: RenderQuality::Exact,
        },
        crate::gpu::GpuEffectSource::LibraryOwnedSourceLayer,
    );
    if !report.is_exact() {
        return None;
    }

    Some(TwExactDropShadowSpec {
        output_rect,
        source_rect,
        radius,
        size,
        callback_id: exact_drop_shadow_callback_id(input, output_rect, source_rect, size),
    })
}

fn exact_drop_shadow_callback_id(
    input: TwExactDropShadowInput,
    output_rect: Rect,
    source_rect: Rect,
    size: [u32; 2],
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    "r100-002-tailwind-exact-drop-shadow".hash(&mut hasher);
    output_rect.min.x.to_bits().hash(&mut hasher);
    output_rect.min.y.to_bits().hash(&mut hasher);
    source_rect.min.x.to_bits().hash(&mut hasher);
    source_rect.min.y.to_bits().hash(&mut hasher);
    input.rect.min.x.to_bits().hash(&mut hasher);
    input.rect.min.y.to_bits().hash(&mut hasher);
    input.rect.max.x.to_bits().hash(&mut hasher);
    input.rect.max.y.to_bits().hash(&mut hasher);
    size.hash(&mut hasher);
    input.shadow.offset.x.to_bits().hash(&mut hasher);
    input.shadow.offset.y.to_bits().hash(&mut hasher);
    input.shadow.blur.hash(&mut hasher);
    input.shadow.color.to_array().hash(&mut hasher);
    input.fill.map(|color| color.to_array()).hash(&mut hasher);
    hasher.finish()
}

fn source_rect_rgba(
    width: u32,
    height: u32,
    output_rect: Rect,
    source_rect: Rect,
    color: Color32,
) -> Vec<u8> {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    let mut rgba = vec![0; (width * height * 4) as usize];
    let min_x = (source_rect.min.x - output_rect.min.x)
        .floor()
        .clamp(0.0, width as f32) as u32;
    let min_y = (source_rect.min.y - output_rect.min.y)
        .floor()
        .clamp(0.0, height as f32) as u32;
    let max_x = (source_rect.max.x - output_rect.min.x)
        .ceil()
        .clamp(0.0, width as f32) as u32;
    let max_y = (source_rect.max.y - output_rect.min.y)
        .ceil()
        .clamp(0.0, height as f32) as u32;
    for y in min_y..max_y {
        for x in min_x..max_x {
            let idx = ((y * width + x) * 4) as usize;
            rgba[idx..idx + 4].copy_from_slice(&[r, g, b, a]);
        }
    }
    rgba
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> TwExactDropShadowInput {
        TwExactDropShadowInput {
            rect: Rect::from_min_size(egui::pos2(10.0, 12.0), egui::vec2(64.0, 32.0)),
            fill: Some(Color32::from_rgb(24, 64, 160)),
            shadow: TwDropShadow {
                offset: egui::vec2(3.0, 5.0),
                blur: 8,
                color: Color32::from_black_alpha(120),
            },
            has_rounded_corners: false,
            has_border: false,
            has_ring: false,
            has_gradient: false,
            has_directional_border: false,
            has_divide: false,
        }
    }

    fn mark_context_ready(ctx: &egui::Context) {
        ctx.data_mut(|data| {
            data.insert_temp(
                egui::Id::new("egui_expressive.gpu_effects.context_ready"),
                true,
            );
        });
    }

    #[test]
    fn exact_drop_shadow_requires_gpu_context_readiness() {
        crate::gpu::set_gpu_effects_initialized_for_tests(false);
        let ctx = egui::Context::default();
        assert!(exact_drop_shadow_shape(&ctx, valid_input()).is_none());
    }

    #[test]
    fn exact_drop_shadow_builds_callback_for_eligible_rect() {
        let ctx = egui::Context::default();
        mark_context_ready(&ctx);
        let shape = exact_drop_shadow_shape(&ctx, valid_input());

        assert!(matches!(shape, Some(Shape::Callback(_))));
    }

    #[test]
    fn exact_drop_shadow_rejects_low_blur_and_unsafe_frame_features() {
        let ctx = egui::Context::default();
        mark_context_ready(&ctx);

        let mut input = valid_input();
        input.shadow.blur = 0;
        assert!(exact_drop_shadow_shape(&ctx, input).is_none());

        let mut rounded = valid_input();
        rounded.has_rounded_corners = true;
        assert!(exact_drop_shadow_shape(&ctx, rounded).is_none());

        let mut bordered = valid_input();
        bordered.has_border = true;
        assert!(exact_drop_shadow_shape(&ctx, bordered).is_none());

        let mut ringed = valid_input();
        ringed.has_ring = true;
        assert!(exact_drop_shadow_shape(&ctx, ringed).is_none());

        let mut gradient = valid_input();
        gradient.has_gradient = true;
        assert!(exact_drop_shadow_shape(&ctx, gradient).is_none());

        let mut missing_fill = valid_input();
        missing_fill.fill = None;
        assert!(exact_drop_shadow_shape(&ctx, missing_fill).is_none());

        let mut translucent_fill = valid_input();
        translucent_fill.fill = Some(Color32::from_rgba_unmultiplied(24, 64, 160, 254));
        assert!(exact_drop_shadow_shape(&ctx, translucent_fill).is_none());
    }

    #[test]
    fn exact_drop_shadow_source_rgba_marks_source_rect_only() {
        let output_rect = Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(6.0, 6.0));
        let source_rect = Rect::from_min_size(egui::pos2(2.0, 1.0), egui::vec2(2.0, 3.0));
        let color = Color32::from_rgba_unmultiplied(10, 20, 30, 40);
        let rgba = source_rect_rgba(6, 6, output_rect, source_rect, color);
        let expected = color.to_srgba_unmultiplied();

        let marked = rgba.chunks_exact(4).filter(|px| *px == expected).count();
        assert_eq!(marked, 6);
    }
}
