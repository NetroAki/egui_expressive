//! Tkinter/ttk-familiar names and options.

use std::ops::RangeInclusive;

use crate::{
    layout::FlexContainer, state::StateSlot, surface::LargeCanvas, swiftui::ScrollList, widgets::*,
};

pub type TkButton<'a> = ToolButton<'a>;
pub type TkLabel = egui::Label;
pub type TkText<'a> = egui::TextEdit<'a>;
pub type TkCombobox = egui::ComboBox;
pub type TkWindow<'a> = egui::Window<'a>;
pub type TkFrameWidget = egui::Frame;
pub type TkScale<'a> = Slider<'a>;
pub type TkCheckbutton<'a> = ToggleDot<'a>;
pub type TkEntry<'a> = SearchField<'a>;
pub type TkMenu = ContextMenuBuilder;
pub type TkNotebook<'a> = TabBar<'a>;
pub type TkPanedWindow<'a> = ResizableSplit<'a>;
pub type TkToplevel<'a> = FloatingPanel<'a>;
pub type TkTreeview<'a> = TreeView<'a>;
pub type TkTreeviewItem = TreeNode;
pub type TkProgressbar<'a> = ProgressOverlay<'a>;
pub type TkCanvas = LargeCanvas;
pub type TkSpinbox<'a> = DragNumber<'a>;
pub type TkLabelFrame<'a> = ControlGroup<'a>;
pub type TkScrollbar<T> = ScrollList<T>;
pub type TkFrame = FlexContainer;
pub type StringVar = StateSlot<String>;
pub type IntVar = StateSlot<i64>;
pub type DoubleVar = StateSlot<f64>;
pub type TkDataTable<'a> = DataTable<'a>;
pub type TkDataGridState = DataGridState;
pub type TkDataColumn = DataColumn;
pub type TkDataRow = DataRow;
pub type TkDataCell = DataCell;
pub type TkDataViewStatus = DataViewStatus;
pub type TkTreeTable<'a> = TreeTable<'a>;
pub type TkPropertyGrid<'a> = PropertyGrid<'a>;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum TkSide {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum TkFill {
    None,
    X,
    Y,
    Both,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TkOptions {
    pub text: Option<String>,
    pub state: String,
    pub padx: f32,
    pub pady: f32,
    pub fill: TkFill,
    pub expand: bool,
    pub side: TkSide,
    pub command: Option<String>,
}

impl Default for TkOptions {
    fn default() -> Self {
        Self {
            text: None,
            state: "normal".into(),
            padx: 0.0,
            pady: 0.0,
            fill: TkFill::None,
            expand: false,
            side: TkSide::Top,
            command: None,
        }
    }
}

impl TkOptions {
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }
    pub fn state(mut self, state: impl Into<String>) -> Self {
        self.state = state.into();
        self
    }
    pub fn disabled(self) -> Self {
        self.state("disabled")
    }
    pub fn padx(mut self, padx: f32) -> Self {
        self.padx = padx;
        self
    }
    pub fn pady(mut self, pady: f32) -> Self {
        self.pady = pady;
        self
    }
    pub fn fill(mut self, fill: TkFill) -> Self {
        self.fill = fill;
        self
    }
    pub fn expand(mut self, expand: bool) -> Self {
        self.expand = expand;
        self
    }
    pub fn side(mut self, side: TkSide) -> Self {
        self.side = side;
        self
    }
    pub fn command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }
}

pub const LEFT: TkSide = TkSide::Left;
pub const RIGHT: TkSide = TkSide::Right;
pub const TOP: TkSide = TkSide::Top;
pub const BOTTOM: TkSide = TkSide::Bottom;
pub const BOTH: TkFill = TkFill::Both;
pub const X: TkFill = TkFill::X;
pub const Y: TkFill = TkFill::Y;

pub fn scale<'a>(value: &'a mut f64, from_to: RangeInclusive<f64>) -> TkScale<'a> {
    Slider::new(value, from_to)
}

pub fn button(command: impl Into<String>, text: impl Into<String>) -> TkButton<'static> {
    ToolButton::new(command, text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tkinter_options_match_pack_and_config_vocabulary() {
        let options = TkOptions::default()
            .text("OK")
            .disabled()
            .padx(4.0)
            .pady(2.0)
            .fill(BOTH)
            .expand(true)
            .side(LEFT)
            .command("save");
        assert_eq!(options.state, "disabled");
        assert_eq!(options.fill, TkFill::Both);
        assert_eq!(options.command.as_deref(), Some("save"));
    }

    #[test]
    fn tkinter_scale_alias_reuses_slider() {
        let mut value = 5.0;
        let _scale: TkScale<'_> = scale(&mut value, 0.0..=10.0);
    }
}
