//! SwiftUI-familiar names and modifier vocabulary.
#![allow(non_snake_case)]

use std::ops::RangeInclusive;

use crate::{state::*, swiftui as native_swiftui, widgets::*};

pub type Button<'a> = ToolButton<'a>;
pub type Text = egui::Label;
pub type Image<'a> = egui::Image<'a>;
pub type Link = egui::Hyperlink;
pub type Window<'a> = egui::Window<'a>;
pub type ScrollView = egui::ScrollArea;
pub type Slider<'a> = crate::widgets::Slider<'a>;
pub type Toggle<'a> = ToggleDot<'a>;
pub type TextField<'a> = SearchField<'a>;
pub type ColorPicker<'a> = ColorSwatch<'a>;
pub type ProgressView<'a> = ProgressOverlay<'a>;
pub type Sheet<'a> = ModalOverlay<'a>;
pub type TabView<'a> = TabBar<'a>;
pub type ToolbarItem = crate::widgets::ToolbarItem;
pub type NavigationView = native_swiftui::Navigator;
pub type List<T> = native_swiftui::ScrollList<T>;
pub type GeometryReader = native_swiftui::GeometryProxy;
pub type State<T> = StateSlot<T>;
pub type AppStorage = PersistenceSlot;
pub type DataTable<'a> = crate::widgets::data::DataTable<'a>;
pub type DataGridState = crate::widgets::data::DataGridState;
pub type DataColumn = crate::widgets::data::DataColumn;
pub type DataRow = crate::widgets::data::DataRow;
pub type DataCell = crate::widgets::data::DataCell;
pub type DataViewStatus = crate::widgets::data::DataViewStatus;
pub type TreeTable<'a> = crate::widgets::data::TreeTable<'a>;
pub type PropertyGrid<'a> = crate::widgets::data::PropertyGrid<'a>;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ViewModifiers {
    pub padding: Option<f32>,
    pub background: Option<egui::Color32>,
    pub foreground_color: Option<egui::Color32>,
    pub corner_radius: Option<f32>,
    pub frame_width: Option<f32>,
    pub frame_height: Option<f32>,
    pub disabled: bool,
    pub hidden: bool,
    pub help: Option<String>,
}

impl ViewModifiers {
    pub fn padding(mut self, value: f32) -> Self {
        self.padding = Some(value);
        self
    }
    pub fn background(mut self, color: egui::Color32) -> Self {
        self.background = Some(color);
        self
    }
    pub fn foreground_color(mut self, color: egui::Color32) -> Self {
        self.foreground_color = Some(color);
        self
    }
    pub fn foregroundColor(self, color: egui::Color32) -> Self {
        self.foreground_color(color)
    }
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = Some(radius);
        self
    }
    pub fn cornerRadius(self, radius: f32) -> Self {
        self.corner_radius(radius)
    }
    pub fn frame(mut self, width: impl Into<Option<f32>>, height: impl Into<Option<f32>>) -> Self {
        self.frame_width = width.into();
        self.frame_height = height.into();
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }
    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

pub fn slider<'a>(value: &'a mut f64, range: RangeInclusive<f64>) -> Slider<'a> {
    crate::widgets::Slider::new(value, range)
}

pub fn button(action_id: impl Into<String>, label: impl Into<String>) -> Button<'static> {
    ToolButton::new(action_id, label)
}

pub fn text_field<'a>(text: &'a mut String) -> TextField<'a> {
    SearchField::new(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swiftui_modifiers_use_expected_names() {
        let mods = ViewModifiers::default()
            .padding(8.0)
            .foregroundColor(egui::Color32::WHITE)
            .cornerRadius(6.0)
            .frame(Some(100.0), None::<f32>)
            .help("tip");
        assert_eq!(mods.padding, Some(8.0));
        assert_eq!(mods.frame_width, Some(100.0));
        assert_eq!(mods.help.as_deref(), Some("tip"));
    }

    #[test]
    fn swiftui_slider_alias_reuses_existing_slider() {
        let mut value = 0.25;
        let _slider: Slider<'_> = slider(&mut value, 0.0..=1.0);
    }
}
