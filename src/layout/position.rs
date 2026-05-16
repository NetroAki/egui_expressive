//! CSS-positioning value types for absolute/relative egui composition.

/// CSS-like positioning mode.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum PositionMode {
    #[default]
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

/// CSS-like inset values in pixels.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Insets {
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub left: Option<f32>,
}

impl Insets {
    pub fn all(value: f32) -> Self {
        Self {
            top: Some(value),
            right: Some(value),
            bottom: Some(value),
            left: Some(value),
        }
    }

    pub fn x(value: f32) -> Self {
        Self {
            left: Some(value),
            right: Some(value),
            ..Default::default()
        }
    }

    pub fn y(value: f32) -> Self {
        Self {
            top: Some(value),
            bottom: Some(value),
            ..Default::default()
        }
    }
}

/// Positioning data shared by layout and Tailwind builder APIs.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct PositionStyle {
    pub mode: PositionMode,
    pub inset: Insets,
    pub translate: egui::Vec2,
    pub z_index: Option<i32>,
}

impl PositionStyle {
    pub fn is_positioned(self) -> bool {
        !matches!(self.mode, PositionMode::Static)
    }
}
