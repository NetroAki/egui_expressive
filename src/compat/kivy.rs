//! Kivy-familiar names and property vocabulary.

use std::ops::RangeInclusive;

use crate::{
    animation::*, draw::LayeredPainter, interaction::*, layout::FlexContainer,
    surface::LargeCanvas, swiftui::ScrollList, widgets::*,
};

pub type KivyButton<'a> = ToolButton<'a>;
pub type KivyLabel = egui::Label;
pub type KivyImage<'a> = egui::Image<'a>;
pub type KivyTextEdit<'a> = egui::TextEdit<'a>;
pub type KivyToggleButton<'a> = ToggleDot<'a>;
pub type KivySlider<'a> = Slider<'a>;
pub type KivyTextInput<'a> = SearchField<'a>;
pub type KivyDropDown = ContextMenuBuilder;
pub type KivyTabbedPanel<'a> = TabBar<'a>;
pub type KivyScrollView<T> = ScrollList<T>;
pub type KivyBoxLayout = FlexContainer;
pub type KivyGridLayout = FlexContainer;
pub type KivyStackLayout = FlexContainer;
pub type KivyFloatLayout<'a> = FloatingPanel<'a>;
pub type KivyScatter = PanZoom;
pub type KivyProgressBar<'a> = ProgressOverlay<'a>;
pub type KivyAccordion<'a> = CollapsePanel<'a>;
pub type KivyCarousel<'a> = TabBar<'a>;
pub type KivyPopup<'a> = ModalOverlay<'a>;
pub type KivyCanvas = LargeCanvas;
pub type KivyInstructionGroup<'a> = LayeredPainter<'a>;
pub type KivyClockTween = Tween;
pub type KivyAnimatedState<T> = AnimatedState<T>;
pub type KivyDataTable<'a> = DataTable<'a>;
pub type KivyDataGridState = DataGridState;
pub type KivyDataColumn = DataColumn;
pub type KivyDataRow = DataRow;
pub type KivyDataCell = DataCell;
pub type KivyDataViewStatus = DataViewStatus;
pub type KivyTreeTable<'a> = TreeTable<'a>;
pub type KivyPropertyGrid<'a> = PropertyGrid<'a>;

#[derive(Clone, Debug, PartialEq)]
pub struct KivyProps {
    pub text: Option<String>,
    pub disabled: bool,
    pub opacity: f32,
    pub size_hint: (Option<f32>, Option<f32>),
    pub pos_hint: (Option<f32>, Option<f32>),
    pub background_color: Option<egui::Color32>,
    pub color: Option<egui::Color32>,
}

impl Default for KivyProps {
    fn default() -> Self {
        Self {
            text: None,
            disabled: false,
            opacity: 1.0,
            size_hint: (None, None),
            pos_hint: (None, None),
            background_color: None,
            color: None,
        }
    }
}

impl KivyProps {
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
    pub fn size_hint(mut self, x: Option<f32>, y: Option<f32>) -> Self {
        self.size_hint = (x, y);
        self
    }
    pub fn pos_hint(mut self, x: Option<f32>, y: Option<f32>) -> Self {
        self.pos_hint = (x, y);
        self
    }
    pub fn background_color(mut self, color: egui::Color32) -> Self {
        self.background_color = Some(color);
        self
    }
    pub fn color(mut self, color: egui::Color32) -> Self {
        self.color = Some(color);
        self
    }
}

pub const ON_PRESS: &str = "on_press";
pub const ON_RELEASE: &str = "on_release";
pub const ON_TOUCH_DOWN: &str = "on_touch_down";
pub const ON_TOUCH_MOVE: &str = "on_touch_move";
pub const ON_TOUCH_UP: &str = "on_touch_up";

pub fn slider<'a>(value: &'a mut f64, range: RangeInclusive<f64>) -> KivySlider<'a> {
    Slider::new(value, range)
}

pub fn button(action_id: impl Into<String>, text: impl Into<String>) -> KivyButton<'static> {
    ToolButton::new(action_id, text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kivy_props_support_size_and_pos_hints() {
        let props = KivyProps::default()
            .text("Play")
            .opacity(1.5)
            .size_hint(Some(1.0), None)
            .pos_hint(Some(0.5), Some(0.0));
        assert_eq!(props.opacity, 1.0);
        assert_eq!(props.size_hint, (Some(1.0), None));
        assert_eq!(props.pos_hint.0, Some(0.5));
    }

    #[test]
    fn kivy_slider_alias_reuses_slider() {
        let mut value = 0.25;
        let _slider: KivySlider<'_> = slider(&mut value, 0.0..=1.0);
    }
}
