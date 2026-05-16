//! PyQt/PySide/Qt-familiar names over existing primitives.
#![allow(non_snake_case)]

use std::ops::RangeInclusive;

use crate::{
    interaction::*, state::StateMachine, surface::LargeCanvas, swiftui::ScrollList, widgets::*,
};

pub type QPushButton<'a> = ToolButton<'a>;
pub type QLabel = egui::Label;
pub type QTextEdit<'a> = egui::TextEdit<'a>;
pub type QComboBox = egui::ComboBox;
pub type QImage<'a> = egui::Image<'a>;
pub type QMainWindow<'a> = egui::Window<'a>;
pub type QFrame = egui::Frame;
pub type QSlider<'a> = Slider<'a>;
pub type QDial<'a> = Knob<'a>;
pub type QCheckBox<'a> = ToggleDot<'a>;
pub type QLineEdit<'a> = SearchField<'a>;
pub type QToolBar<'a> = ToolbarStrip<'a>;
pub type QMenuBar<'a> = TopMenuBar<'a>;
pub type QMenu = ContextMenuBuilder;
pub type QTabWidget<'a> = TabBar<'a>;
pub type QTreeWidget<'a> = TreeView<'a>;
pub type QTreeWidgetItem = TreeNode;
pub type QSplitter<'a> = ResizableSplit<'a>;
pub type QDockWidget<'a> = FloatingPanel<'a>;
pub type QDialog<'a> = ModalOverlay<'a>;
pub type QProgressBar<'a> = ProgressOverlay<'a>;
pub type QGraphicsView = LargeCanvas;
pub type QScrollArea<T> = ScrollList<T>;
pub type QAction = ActionDef;
pub type QShortcut = ShortcutBinding;
pub type QStateMachine<S> = StateMachine<S>;
pub type QSpinBox<'a> = DragNumber<'a>;
pub type QColorDialog<'a> = ColorSwatch<'a>;
pub type QLevelMeter = Meter;
pub type QDataTable<'a> = DataTable<'a>;
pub type QDataGridState = DataGridState;
pub type QDataColumn = DataColumn;
pub type QDataRow = DataRow;
pub type QDataCell = DataCell;
pub type QDataViewStatus = DataViewStatus;
pub type QTreeTable<'a> = TreeTable<'a>;
pub type QPropertyGrid<'a> = PropertyGrid<'a>;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum QtOrientation {
    Horizontal,
    Vertical,
}

impl From<QtOrientation> for Orientation {
    fn from(value: QtOrientation) -> Self {
        match value {
            QtOrientation::Horizontal => Self::Horizontal,
            QtOrientation::Vertical => Self::Vertical,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct QtWidgetProps {
    pub object_name: Option<String>,
    pub enabled: bool,
    pub visible: bool,
    pub tooltip: Option<String>,
    pub style_sheet: Option<String>,
    pub minimum_size: Option<egui::Vec2>,
    pub maximum_size: Option<egui::Vec2>,
}

impl QtWidgetProps {
    pub fn new() -> Self {
        Self {
            enabled: true,
            visible: true,
            ..Self::default()
        }
    }
    pub fn set_object_name(mut self, name: impl Into<String>) -> Self {
        self.object_name = Some(name.into());
        self
    }
    pub fn setObjectName(self, name: impl Into<String>) -> Self {
        self.set_object_name(name)
    }
    pub fn set_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn setEnabled(self, enabled: bool) -> Self {
        self.set_enabled(enabled)
    }
    pub fn set_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
    pub fn setVisible(self, visible: bool) -> Self {
        self.set_visible(visible)
    }
    pub fn set_tool_tip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }
    pub fn setToolTip(self, tooltip: impl Into<String>) -> Self {
        self.set_tool_tip(tooltip)
    }
    pub fn set_style_sheet(mut self, style_sheet: impl Into<String>) -> Self {
        self.style_sheet = Some(style_sheet.into());
        self
    }
    pub fn setStyleSheet(self, style_sheet: impl Into<String>) -> Self {
        self.set_style_sheet(style_sheet)
    }
}

pub fn q_slider<'a>(value: &'a mut f64, range: RangeInclusive<f64>) -> QSlider<'a> {
    Slider::new(value, range)
}

pub fn q_dial<'a>(value: &'a mut f64, range: RangeInclusive<f64>) -> QDial<'a> {
    Knob::new(value, range)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qt_properties_support_camel_case_redirects() {
        let props = QtWidgetProps::new()
            .setObjectName("gain")
            .setEnabled(false)
            .setToolTip("Gain");
        assert_eq!(props.object_name.as_deref(), Some("gain"));
        assert!(!props.enabled);
        assert_eq!(props.tooltip.as_deref(), Some("Gain"));
    }

    #[test]
    fn qt_aliases_construct_existing_widgets() {
        let mut value = 0.5;
        let _slider: QSlider<'_> = q_slider(&mut value, 0.0..=1.0);
    }
}
