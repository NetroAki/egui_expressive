//! Tailwind-like `hover:` / `focus:` / `disabled:` style variants.

use egui::Response;

use crate::accessibility::MotionPolicy;
use crate::animation::{transition_color, transition_f32, Easing, Transition};
use crate::style::VisualVariant;
use crate::tailwind::builder::Tw;

/// CSS/Tailwind-like interaction variants understood by the styling layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum TwVariant {
    #[default]
    Base,
    Hover,
    Pressed,
    Focus,
    Selected,
    Disabled,
}

impl TwVariant {
    /// Map the crate's generic visual-state enum to Tailwind-style variant names.
    pub fn from_visual_variant(variant: VisualVariant) -> Self {
        match variant {
            VisualVariant::Inactive => Self::Base,
            VisualVariant::Hovered => Self::Hover,
            VisualVariant::Pressed => Self::Pressed,
            VisualVariant::Selected => Self::Selected,
            VisualVariant::Focused => Self::Focus,
            VisualVariant::Disabled => Self::Disabled,
        }
    }

    /// Resolve the active Tailwind-style variant from an egui response.
    pub fn from_response(response: &Response, selected: bool, disabled: bool) -> Self {
        Self::from_visual_variant(VisualVariant::from_response(response, selected, disabled))
    }
}

/// State-aware `Tw` style set.
///
/// This mirrors Tailwind's mental model: a base utility class plus optional
/// `hover:`/`focus:`/`disabled:` overrides. Missing variants fall back to base.
#[derive(Clone, Debug)]
pub struct TwVariants {
    pub base: Tw,
    pub hover: Option<Tw>,
    pub pressed: Option<Tw>,
    pub focus: Option<Tw>,
    pub selected: Option<Tw>,
    pub disabled: Option<Tw>,
}

impl TwVariants {
    pub fn new(base: Tw) -> Self {
        Self {
            base,
            hover: None,
            pressed: None,
            focus: None,
            selected: None,
            disabled: None,
        }
    }

    pub fn hover(mut self, style: Tw) -> Self {
        self.hover = Some(style);
        self
    }

    pub fn pressed(mut self, style: Tw) -> Self {
        self.pressed = Some(style);
        self
    }

    pub fn focus(mut self, style: Tw) -> Self {
        self.focus = Some(style);
        self
    }

    pub fn selected(mut self, style: Tw) -> Self {
        self.selected = Some(style);
        self
    }

    pub fn disabled(mut self, style: Tw) -> Self {
        self.disabled = Some(style);
        self
    }

    /// Return the best style for a variant, falling back to `base` when absent.
    pub fn resolve(&self, variant: TwVariant) -> &Tw {
        match variant {
            TwVariant::Base => &self.base,
            TwVariant::Hover => self.hover.as_ref().unwrap_or(&self.base),
            TwVariant::Pressed => self.pressed.as_ref().unwrap_or(&self.base),
            TwVariant::Focus => self.focus.as_ref().unwrap_or(&self.base),
            TwVariant::Selected => self.selected.as_ref().unwrap_or(&self.base),
            TwVariant::Disabled => self.disabled.as_ref().unwrap_or(&self.base),
        }
    }

    /// Resolve style from an egui response using the same ordering as `VisualVariant`.
    pub fn resolve_response(&self, response: &Response, selected: bool, disabled: bool) -> &Tw {
        self.resolve(TwVariant::from_response(response, selected, disabled))
    }

    pub fn show_variant(
        self,
        ui: &mut egui::Ui,
        variant: TwVariant,
        content: impl FnOnce(&mut egui::Ui),
    ) -> Response {
        self.resolve_animated(ui.ctx(), ui.id().with("tw_variant"), variant)
            .show(ui, content)
    }

    pub fn show_response(
        self,
        ui: &mut egui::Ui,
        response: &Response,
        selected: bool,
        disabled: bool,
        content: impl FnOnce(&mut egui::Ui),
    ) -> Response {
        let variant = TwVariant::from_response(response, selected, disabled);
        self.resolve_animated(ui.ctx(), response.id.with("tw_variant"), variant)
            .show(ui, content)
    }

    /// Resolve a variant and animate a small, deterministic subset of visual
    /// fields when the target style declares `Tw::transition(duration)`.
    ///
    /// This is intentionally bounded: it animates color, opacity, border width,
    /// and radius intent, while layout-affecting utilities still snap to their
    /// target values to avoid reflow churn.
    pub fn resolve_animated(&self, ctx: &egui::Context, id: egui::Id, variant: TwVariant) -> Tw {
        let target = self.resolve(variant);
        let Some(transition) = target.transition.or(self.base.transition) else {
            return target.clone();
        };
        let duration = MotionPolicy::from_ctx(ctx).duration(transition.duration_secs.max(0.0));
        if duration <= f32::EPSILON {
            return target.clone();
        }

        let transition = Transition::new(duration, Easing::EaseInOut);
        let mut resolved = target.clone();
        resolved.opacity = transition_f32(
            ctx,
            id.with("opacity"),
            target.opacity,
            self.base.opacity,
            transition,
        );
        resolved.border_width = transition_f32(
            ctx,
            id.with("border_width"),
            target.border_width,
            self.base.border_width,
            transition,
        );
        resolved.border_radius = transition_f32(
            ctx,
            id.with("border_radius"),
            target.border_radius,
            self.base.border_radius,
            transition,
        );
        if let (Some(target), Some(base)) = (target.bg, self.base.bg) {
            resolved.bg = Some(transition_color(
                ctx,
                id.with("bg"),
                target,
                base,
                transition,
            ));
        }
        if let (Some(target), Some(base)) = (target.fg, self.base.fg) {
            resolved.fg = Some(transition_color(
                ctx,
                id.with("fg"),
                target,
                base,
                transition,
            ));
        }
        if let (Some(target), Some(base)) = (target.border_color, self.base.border_color) {
            resolved.border_color = Some(transition_color(
                ctx,
                id.with("border_color"),
                target,
                base,
                transition,
            ));
        }
        resolved
    }
}

impl Default for TwVariants {
    fn default() -> Self {
        Self::new(Tw::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tailwind::types::Size;

    #[test]
    fn variants_fall_back_to_base() {
        let styles = TwVariants::new(Tw::new().p(8.0));
        assert_eq!(styles.resolve(TwVariant::Hover).padding.top, 8.0);
    }

    #[test]
    fn variants_return_overrides() {
        let styles = Tw::new()
            .p(8.0)
            .hover(Tw::new().p(12.0))
            .focus(Tw::new().border_2());
        assert_eq!(styles.resolve(TwVariant::Hover).padding.top, 12.0);
        assert_eq!(styles.resolve(TwVariant::Focus).border_width, 2.0);
    }

    #[test]
    fn show_variant_entrypoint_selects_override() {
        let styles = Tw::new().p(4.0).hover(Tw::new().p(9.0));
        assert_eq!(styles.resolve(TwVariant::Hover).padding.top, 9.0);
    }

    #[test]
    fn animated_resolve_uses_transition_and_reduced_motion() {
        let ctx = egui::Context::default();
        let styles = Tw::new().bg(egui::Color32::BLACK).opacity(1.0).hover(
            Tw::new()
                .bg(egui::Color32::WHITE)
                .opacity(0.5)
                .transition(0.2),
        );

        let animated =
            styles.resolve_animated(&ctx, egui::Id::new("tw_transition"), TwVariant::Hover);
        assert!(animated.transition.is_some());
        crate::accessibility::motion::set_reduced_motion(&ctx, true);
        let reduced =
            styles.resolve_animated(&ctx, egui::Id::new("tw_transition"), TwVariant::Hover);
        assert_eq!(reduced.bg, Some(egui::Color32::WHITE));
        assert_eq!(reduced.opacity, 0.5);
    }

    #[test]
    fn phase7_state_endpoints_snap_layout_and_interpolate_safe_visual_fields() {
        let ctx = egui::Context::default();
        let styles = Tw::new()
            .p(8.0)
            .w(80.0)
            .h(32.0)
            .rounded(6.0)
            .border(1.0)
            .border_color(egui::Color32::from_rgb(71, 85, 105))
            .bg(egui::Color32::from_rgb(15, 23, 42))
            .opacity(1.0)
            .hover(
                Tw::new()
                    .p(16.0)
                    .w(120.0)
                    .h(40.0)
                    .rounded(12.0)
                    .border(2.0)
                    .border_color(egui::Color32::from_rgb(14, 165, 233))
                    .bg(egui::Color32::from_rgb(37, 99, 235))
                    .opacity(0.72)
                    .transition(0.2),
            );

        let resolved = styles.resolve_animated(
            &ctx,
            egui::Id::new("phase7_state_endpoint"),
            TwVariant::Hover,
        );

        assert_eq!(resolved.padding.top, 16.0);
        assert_eq!(resolved.padding.right, 16.0);
        assert_eq!(resolved.width, Size::Px(120.0));
        assert_eq!(resolved.height, Size::Px(40.0));
        assert!(resolved.transition.is_some());
        assert!(
            (0.0..=1.0).contains(&resolved.opacity),
            "opacity is a safe interpolated field, not a layout reflow trigger"
        );

        crate::accessibility::motion::set_reduced_motion(&ctx, true);
        let reduced = styles.resolve_animated(
            &ctx,
            egui::Id::new("phase7_state_endpoint"),
            TwVariant::Hover,
        );
        assert_eq!(reduced.padding.top, 16.0);
        assert_eq!(reduced.width, Size::Px(120.0));
        assert_eq!(reduced.bg, Some(egui::Color32::from_rgb(37, 99, 235)));
        assert_eq!(reduced.opacity, 0.72);
    }

    #[test]
    fn phase7_state_endpoint_variants_resolve_all_named_states() {
        let styles = Tw::new()
            .p(4.0)
            .hover(Tw::new().p(8.0).bg(egui::Color32::from_rgb(37, 99, 235)))
            .focus(Tw::new().p(10.0).bg(egui::Color32::from_rgb(79, 70, 229)))
            .selected(Tw::new().p(12.0).bg(egui::Color32::from_rgb(20, 184, 166)))
            .disabled(Tw::new().p(2.0).opacity(0.38));

        assert_eq!(styles.resolve(TwVariant::Hover).padding.top, 8.0);
        assert_eq!(
            styles.resolve(TwVariant::Hover).bg,
            Some(egui::Color32::from_rgb(37, 99, 235))
        );
        assert_eq!(styles.resolve(TwVariant::Focus).padding.top, 10.0);
        assert_eq!(
            styles.resolve(TwVariant::Focus).bg,
            Some(egui::Color32::from_rgb(79, 70, 229))
        );
        assert_eq!(styles.resolve(TwVariant::Selected).padding.top, 12.0);
        assert_eq!(
            styles.resolve(TwVariant::Selected).bg,
            Some(egui::Color32::from_rgb(20, 184, 166))
        );
        assert_eq!(styles.resolve(TwVariant::Disabled).padding.top, 2.0);
        assert_eq!(styles.resolve(TwVariant::Disabled).opacity, 0.38);
    }
}
