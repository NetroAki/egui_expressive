//! Cross-environment compatibility names for users coming from HTML/Electron,
//! SwiftUI, Tkinter, PyQt/PySide, and Kivy.
//!
//! These modules are intentionally thin. They provide familiar names, property
//! structs, constants, and constructor helpers while delegating rendering and
//! interaction to the existing `egui_expressive` primitives.

pub mod html;
pub mod kivy;
pub mod qt;
pub mod swiftui;
pub mod tkinter;

use egui::{Color32, Vec2};
use serde::{Deserialize, Serialize};

/// Shared property vocabulary that mirrors common `text/value/enabled/visible`
/// fields across DOM, SwiftUI, Qt, Tkinter, and Kivy.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CommonProps {
    pub id: Option<String>,
    pub class_name: Option<String>,
    pub text: Option<String>,
    pub value: Option<String>,
    pub disabled: bool,
    pub visible: bool,
    pub tooltip: Option<String>,
    pub placeholder: Option<String>,
    pub role: Option<String>,
    pub aria_label: Option<String>,
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl CommonProps {
    pub fn new() -> Self {
        Self {
            visible: true,
            ..Self::default()
        }
    }
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
    pub fn class(mut self, class_name: impl Into<String>) -> Self {
        self.class_name = Some(class_name.into());
        self
    }
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.disabled = !enabled;
        self
    }
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }
    pub fn role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }
    pub fn aria_label(mut self, aria_label: impl Into<String>) -> Self {
        self.aria_label = Some(aria_label.into());
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }
    pub fn size(mut self, size: Vec2) -> Self {
        self.width = Some(size.x);
        self.height = Some(size.y);
        self
    }
}

/// Common event/callback terminology aliases.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum UiEvent {
    Click,
    DoubleClick,
    SecondaryClick,
    Hover,
    Press,
    Release,
    Change,
    Input,
    Submit,
    Focus,
    Blur,
    KeyDown,
    KeyUp,
    DragStart,
    Drag,
    DragEnd,
}

/// CSS/toolkit box-model vocabulary. It is metadata for builders/examples; egui
/// still owns the real immediate-mode layout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BoxModel {
    pub margin: f32,
    pub padding: f32,
    pub border_width: f32,
    pub border_radius: f32,
}

impl BoxModel {
    pub fn margin(mut self, margin: f32) -> Self {
        self.margin = margin;
        self
    }
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
    pub fn border_width(mut self, border_width: f32) -> Self {
        self.border_width = border_width;
        self
    }
    pub fn border_radius(mut self, border_radius: f32) -> Self {
        self.border_radius = border_radius;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StyleProps {
    pub background: Option<Color32>,
    pub foreground: Option<Color32>,
    pub border: Option<Color32>,
    pub opacity: f32,
    pub box_model: BoxModel,
}

impl Default for StyleProps {
    fn default() -> Self {
        Self {
            background: None,
            foreground: None,
            border: None,
            opacity: 1.0,
            box_model: BoxModel::default(),
        }
    }
}

impl StyleProps {
    pub fn background(mut self, color: Color32) -> Self {
        self.background = Some(color);
        self
    }
    pub fn foreground(mut self, color: Color32) -> Self {
        self.foreground = Some(color);
        self
    }
    pub fn color(self, color: Color32) -> Self {
        self.foreground(color)
    }
    pub fn border(mut self, color: Color32) -> Self {
        self.border = Some(color);
        self
    }
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
    pub fn padding(mut self, padding: f32) -> Self {
        self.box_model.padding = padding;
        self
    }
    pub fn margin(mut self, margin: f32) -> Self {
        self.box_model.margin = margin;
        self
    }
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.box_model.border_radius = radius;
        self
    }
}

/// Familiar orientation names shared by Qt/Tkinter/Kivy/HTML helpers.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum CompatOrientation {
    Horizontal,
    Vertical,
}

impl From<CompatOrientation> for crate::widgets::Orientation {
    fn from(value: CompatOrientation) -> Self {
        match value {
            CompatOrientation::Horizontal => Self::Horizontal,
            CompatOrientation::Vertical => Self::Vertical,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_props_uses_framework_vocabulary() {
        let props = CommonProps::new()
            .id("gain")
            .class("slider")
            .text("Gain")
            .disabled(false)
            .aria_label("Gain slider")
            .width(120.0);
        assert_eq!(props.id.as_deref(), Some("gain"));
        assert_eq!(props.class_name.as_deref(), Some("slider"));
        assert!(props.visible);
        assert_eq!(props.width, Some(120.0));
    }

    #[test]
    fn style_props_clamps_opacity() {
        assert_eq!(StyleProps::default().opacity(2.0).opacity, 1.0);
        assert_eq!(StyleProps::default().padding(8.0).box_model.padding, 8.0);
    }
}
