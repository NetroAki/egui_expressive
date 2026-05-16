use super::*;

// ---------------------------------------------------------------------------
// VisualVariant
// ---------------------------------------------------------------------------

/// The six interactive visual states a widget can be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VisualVariant {
    #[default]
    Inactive,
    Hovered,
    Pressed,
    Selected,
    Focused,
    Disabled,
}

impl VisualVariant {
    /// Determine the appropriate variant from an interaction [`Response`].
    #[inline]
    pub fn from_response(r: &Response, selected: bool, disabled: bool) -> Self {
        if disabled {
            Self::Disabled
        } else if r.is_pointer_button_down_on() {
            Self::Pressed
        } else if r.has_focus() {
            Self::Focused
        } else if selected {
            Self::Selected
        } else if r.hovered() {
            Self::Hovered
        } else {
            Self::Inactive
        }
    }
}

// ---------------------------------------------------------------------------
// VisualState
// ---------------------------------------------------------------------------

/// A value parameterized by six visual variants.
#[derive(Debug, Clone)]
pub struct VisualState<T> {
    pub inactive: T,
    pub hovered: T,
    pub pressed: T,
    pub selected: T,
    pub focused: T,
    pub disabled: T,
}

impl<T: Clone> VisualState<T> {
    /// Initialize all variants with the same value.
    #[inline]
    pub fn uniform(value: T) -> Self {
        Self {
            inactive: value.clone(),
            hovered: value.clone(),
            pressed: value.clone(),
            selected: value.clone(),
            focused: value.clone(),
            disabled: value,
        }
    }

    /// Get the value for the given variant.
    #[inline]
    pub fn get(&self, variant: VisualVariant) -> &T {
        match variant {
            VisualVariant::Inactive => &self.inactive,
            VisualVariant::Hovered => &self.hovered,
            VisualVariant::Pressed => &self.pressed,
            VisualVariant::Selected => &self.selected,
            VisualVariant::Focused => &self.focused,
            VisualVariant::Disabled => &self.disabled,
        }
    }

    /// Resolve the correct value for a [`Response`]'s current interaction state.
    #[inline]
    pub fn resolve(&self, r: &Response, selected: bool, disabled: bool) -> &T {
        self.get(VisualVariant::from_response(r, selected, disabled))
    }
}

impl<T: Default> Default for VisualState<T> {
    fn default() -> Self {
        Self {
            inactive: T::default(),
            hovered: T::default(),
            pressed: T::default(),
            selected: T::default(),
            focused: T::default(),
            disabled: T::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Lerp
// ---------------------------------------------------------------------------

/// Trait for linear interpolation between two values.
pub trait Lerp: Sized {
    /// Linearly interpolate between `a` and `b` with parameter `t` in [0, 1].
    fn lerp(a: &Self, b: &Self, t: f32) -> Self;
}

impl Lerp for f32 {
    #[inline]
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        a + (b - a) * t
    }
}

impl Lerp for Color32 {
    #[inline]
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        let a = a.to_tuple();
        let b = b.to_tuple();
        Color32::from_rgba_unmultiplied(
            (a.0 as f32 + (b.0 as f32 - a.0 as f32) * t).round() as u8,
            (a.1 as f32 + (b.1 as f32 - a.1 as f32) * t).round() as u8,
            (a.2 as f32 + (b.2 as f32 - a.2 as f32) * t).round() as u8,
            (a.3 as f32 + (b.3 as f32 - a.3 as f32) * t).round() as u8,
        )
    }
}

impl Lerp for Stroke {
    #[inline]
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        Stroke {
            width: Lerp::lerp(&a.width, &b.width, t),
            color: Lerp::lerp(&a.color, &b.color, t),
        }
    }
}

impl Lerp for egui::CornerRadius {
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        egui::CornerRadius {
            nw: lerp_u8(a.nw, b.nw, t),
            ne: lerp_u8(a.ne, b.ne, t),
            sw: lerp_u8(a.sw, b.sw, t),
            se: lerp_u8(a.se, b.se, t),
        }
    }
}

impl Lerp for egui::Vec2 {
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        egui::Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
    }
}

impl Lerp for egui::Pos2 {
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        egui::Pos2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
    }
}

pub(crate) fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

// ---------------------------------------------------------------------------
// Animated resolution extension
// ---------------------------------------------------------------------------

/// Extension trait providing animated resolution for [`VisualState`].
pub trait VisualStateExt<T: Clone + 'static> {
    /// Resolve the value for the current interaction state, animating smoothly
    /// between the previous and current variant over `duration` seconds.
    fn resolve_animated(
        &self,
        ctx: &Context,
        id: Id,
        r: &Response,
        selected: bool,
        disabled: bool,
        duration: f32,
    ) -> T;
}

impl<T: Lerp + Clone + 'static> VisualStateExt<T> for VisualState<T> {
    fn resolve_animated(
        &self,
        ctx: &Context,
        id: Id,
        r: &Response,
        selected: bool,
        disabled: bool,
        duration: f32,
    ) -> T {
        let current = VisualVariant::from_response(r, selected, disabled);

        // Retrieve or initialize the last variant from egui memory
        let last = ctx.memory(|m| m.data.get_temp::<VisualVariant>(id.with("__exp_vis")));

        // Update stored last variant
        ctx.memory_mut(|m| m.data.insert_temp(id.with("__exp_vis"), current));

        // Get the current target value
        let target = self.get(current).clone();

        // If nothing changed, return the target directly
        if Some(current) == last {
            return target;
        }

        // Compute animation t (0 → 1)
        let animating = last.is_some() && last != Some(current);
        let t = if animating {
            ctx.animate_bool_with_time(id.with("__anim"), true, duration)
        } else {
            1.0
        };

        // Lerp from previous value to current target
        if let Some(last_variant) = last {
            let prev = self.get(last_variant).clone();
            Lerp::lerp(&prev, &target, t)
        } else {
            target
        }
    }
}

// ---------------------------------------------------------------------------
// WidgetTheme
// ---------------------------------------------------------------------------
