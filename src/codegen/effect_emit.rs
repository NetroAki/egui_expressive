use super::{EffectDef, EffectType};

pub(crate) const BOUNDED_CODEGEN_PREFIX: &str = "R100-004A bounded codegen";
pub(crate) const UNSUPPORTED_CODEGEN_PREFIX: &str = "R100-004A unsupported codegen";

pub(crate) fn emit_pre_shape_effect(effect: &EffectDef, opacity: f32, indent: &str) -> String {
    let [r, g, b, a] = effect.color.to_srgba_unmultiplied();
    let alpha = (a as f32 * opacity).clamp(0.0, 255.0) as u8;

    match &effect.effect_type {
        EffectType::DropShadow => format!(
            "{indent}// {BOUNDED_CODEGEN_PREFIX}: DropShadow emits bounded box_shadow; exact generated WGPU callback emission remains R100-004 follow-up.\n\
             {indent}for s in egui_expressive::box_shadow(rect, egui::Color32::from_rgba_unmultiplied({r}, {g}, {b}, {alpha}), {:.1}, {:.1}, egui_expressive::ShadowOffset::new({:.1}, {:.1})) {{ painter.add(s); }}\n",
            effect.blur, effect.spread, effect.x, effect.y
        ),
        EffectType::OuterGlow => format!(
            "{indent}// {BOUNDED_CODEGEN_PREFIX}: OuterGlow emits bounded soft_shadow; exact generated WGPU callback emission remains R100-004 follow-up.\n\
             {indent}for s in egui_expressive::soft_shadow(rect, egui::Color32::from_rgba_unmultiplied({r}, {g}, {b}, {alpha}), {:.1}, 0.0, egui_expressive::ShadowOffset::zero(), egui_expressive::BlurQuality::Medium) {{ painter.add(s); }}\n",
            effect.blur
        ),
        _ => String::new(),
    }
}

pub(crate) fn emit_post_shape_effect(effect: &EffectDef, indent: &str) -> String {
    let r = effect.color.r();
    let g = effect.color.g();
    let b = effect.color.b();
    let a = effect.color.a();

    match &effect.effect_type {
        EffectType::InnerShadow => format!(
            "{indent}// {BOUNDED_CODEGEN_PREFIX}: InnerShadow emits bounded inner_shadow; exact generated WGPU callback emission remains R100-004 follow-up.\n\
             {indent}for s in egui_expressive::inner_shadow(rect, egui::Color32::from_rgba_unmultiplied({r}, {g}, {b}, {a}), {:.1}) {{ painter.add(s); }}\n",
            effect.blur
        ),
        EffectType::Noise => format!(
            "{indent}// {BOUNDED_CODEGEN_PREFIX}: Noise emits bounded noise_rect vector approximation.\n\
             {indent}for s in egui_expressive::noise_rect(rect, {}, {:.2}, {:.2}) {{ painter.add(s); }}\n",
            effect.seed, effect.scale, effect.amount
        ),
        EffectType::Bevel => format!(
            "{indent}// {UNSUPPORTED_CODEGEN_PREFIX}: Bevel has no exported direct bounded helper or exact generated callback in R100-004A.\n"
        ),
        EffectType::GaussianBlur => format!(
            "{indent}// {BOUNDED_CODEGEN_PREFIX}: GaussianBlur emits bounded soft_shadow blur approximation; exact generated WGPU callback emission remains R100-004 follow-up.\n\
             {indent}for s in egui_expressive::blur::soft_shadow(rect, egui::Color32::from_rgba_unmultiplied({r}, {g}, {b}, {a}), {:.1}, 0.0, egui_expressive::ShadowOffset::zero(), egui_expressive::blur::BlurQuality::High) {{ painter.add(s); }}\n",
            effect.radius
        ),
        EffectType::Feather => format!(
            "{indent}// {BOUNDED_CODEGEN_PREFIX}: Feather emits bounded soft_shadow feather approximation; exact generated WGPU callback emission remains R100-004 follow-up.\n\
             {indent}for s in egui_expressive::blur::soft_shadow(rect, egui::Color32::from_rgba_unmultiplied({r}, {g}, {b}, {a}), {:.1}, 0.0, egui_expressive::ShadowOffset::zero(), egui_expressive::blur::BlurQuality::High) {{ painter.add(s); }}\n",
            effect.radius
        ),
        EffectType::InnerGlow => format!(
            "{indent}// {UNSUPPORTED_CODEGEN_PREFIX}: InnerGlow has no direct bounded helper or exact generated callback in R100-004A.\n"
        ),
        EffectType::LiveEffect => format!(
            "{indent}// {UNSUPPORTED_CODEGEN_PREFIX}: LiveEffect has no direct bounded helper or exact generated callback in R100-004A.\n"
        ),
        EffectType::Unknown(name) => format!(
            "{indent}// {UNSUPPORTED_CODEGEN_PREFIX}: unrecognized effect {name:?} has no direct bounded helper or exact generated callback in R100-004A.\n"
        ),
        EffectType::DropShadow | EffectType::OuterGlow => String::new(),
    }
}
