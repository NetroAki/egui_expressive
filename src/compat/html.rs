//! HTML/Electron-style names over `egui_expressive` primitives.

use std::ops::RangeInclusive;

use crate::{layout::FlexContainer, surface::LargeCanvas, swiftui::ScrollList, widgets::*};

pub type HtmlButton<'a> = ToolButton<'a>;
pub type HtmlLabel = egui::Label;
pub type HtmlSpan = egui::Label;
pub type HtmlTextInput<'a> = egui::TextEdit<'a>;
pub type HtmlSelect = egui::ComboBox;
pub type HtmlImage<'a> = egui::Image<'a>;
pub type HtmlLink = egui::Hyperlink;
pub type HtmlWindow<'a> = egui::Window<'a>;
pub type HtmlScrollArea = egui::ScrollArea;
pub type HtmlFrame = egui::Frame;
pub type HtmlInputRange<'a> = Slider<'a>;
pub type HtmlVerticalRange<'a> = Fader<'a>;
pub type HtmlRangeSlider<'a> = RangeSlider<'a>;
pub type HtmlInputColor<'a> = ColorSwatch<'a>;
pub type HtmlInputSearch<'a> = SearchField<'a>;
pub type HtmlCheckbox<'a> = ToggleDot<'a>;
pub type HtmlDialog<'a> = ModalOverlay<'a>;
pub type HtmlProgress<'a> = ProgressOverlay<'a>;
pub type HtmlMeter = Meter;
pub type HtmlTabList<'a> = TabBar<'a>;
pub type HtmlMenu = ContextMenuBuilder;
pub type HtmlCanvas = LargeCanvas;
pub type HtmlFlex = FlexContainer;
pub type HtmlScrollList<T> = ScrollList<T>;
pub type HtmlToolbar<'a> = ToolbarStrip<'a>;
pub type HtmlTree<'a> = TreeView<'a>;
pub type HtmlDataTable<'a> = DataTable<'a>;
pub type HtmlDataGridState = DataGridState;
pub type HtmlDataColumn = DataColumn;
pub type HtmlDataRow = DataRow;
pub type HtmlDataCell = DataCell;
pub type HtmlDataViewStatus = DataViewStatus;
pub type HtmlTreeTable<'a> = TreeTable<'a>;
pub type HtmlPropertyGrid<'a> = PropertyGrid<'a>;
pub type HtmlToastLayer<'a> = ToastLayer<'a>;

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DomProps {
    pub id: Option<String>,
    pub class_name: Option<String>,
    pub title: Option<String>,
    pub role: Option<String>,
    pub aria_label: Option<String>,
    pub data_action: Option<String>,
    pub disabled: bool,
    pub hidden: bool,
}

impl DomProps {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
    pub fn class(mut self, class_name: impl Into<String>) -> Self {
        self.class_name = Some(class_name.into());
        self
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
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
    pub fn data_action(mut self, data_action: impl Into<String>) -> Self {
        self.data_action = Some(data_action.into());
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum HtmlInputType {
    Button,
    Checkbox,
    Color,
    Range,
    Search,
    Text,
}

pub const ON_CLICK: &str = "onclick";
pub const ON_CHANGE: &str = "onchange";
pub const ON_INPUT: &str = "oninput";
pub const ON_KEY_DOWN: &str = "onkeydown";
pub const ON_CONTEXT_MENU: &str = "oncontextmenu";

pub fn button(action_id: impl Into<String>, label: impl Into<String>) -> HtmlButton<'static> {
    ToolButton::new(action_id, label)
}

pub fn input_range<'a>(value: &'a mut f64, range: RangeInclusive<f64>) -> HtmlInputRange<'a> {
    Slider::new(value, range)
}

pub fn input_search<'a>(query: &'a mut String) -> HtmlInputSearch<'a> {
    SearchField::new(query)
}

pub fn input_color<'a>(color: &'a mut egui::Color32) -> HtmlInputColor<'a> {
    ColorSwatch::new(color)
}

pub fn meter(value: f32) -> HtmlMeter {
    Meter::new(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_aliases_construct_existing_primitives() {
        let mut value = 0.5;
        let _slider: HtmlInputRange<'_> = input_range(&mut value, 0.0..=1.0).label("volume");
        let props = DomProps::default()
            .id("volume")
            .class("range")
            .data_action("set-volume");
        assert_eq!(props.data_action.as_deref(), Some("set-volume"));
    }

    #[test]
    fn html_event_names_match_dom_terms() {
        assert_eq!(ON_CLICK, "onclick");
        assert_eq!(ON_CHANGE, "onchange");
    }
}
