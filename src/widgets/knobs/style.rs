/// Control orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Orientation {
    #[default]
    Vertical,
    Horizontal,
}

/// Visual style for a Knob widget.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum KnobStyle {
    /// Arc track with filled value arc and indicator line (default).
    #[default]
    Default,
    /// Filled circle with indicator line, no arc track.
    Flat,
    /// Thin ring outline with indicator line.
    Ring,
    /// Tick marks around the perimeter.
    Notched,
}

/// Preset size for a Knob widget.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum KnobSize {
    Xs,
    Sm,
    #[default]
    Md,
    Lg,
}

/// Reset gesture used by continuous controls.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ResetGesture {
    /// Do not reset from pointer gestures.
    None,
    /// Reset when the response is double-clicked. Kept as the low-level default
    /// for backwards compatibility with `ContinuousControl`.
    #[default]
    DoubleClick,
    /// Reset on middle click. Used by pro-audio knob/fader primitives to avoid
    /// colliding with double-click inline-edit affordances.
    MiddleClick,
    /// Reset on secondary/right click.
    SecondaryClick,
}

impl KnobSize {
    pub fn to_px(self) -> f32 {
        match self {
            KnobSize::Xs => 24.0,
            KnobSize::Sm => 32.0,
            KnobSize::Md => 48.0,
            KnobSize::Lg => 64.0,
        }
    }
}
