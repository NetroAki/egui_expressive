//! Low-level styled primitives for product-specific component systems.
//!
//! These widgets keep interaction handling generic while letting applications own their
//! visual language. They intentionally avoid Material, platform, or product-specific semantics.

use egui;

#[derive(Clone, Copy, Debug)]
pub struct InteractiveFill {
    pub idle: egui::Color32,
    pub hovered: egui::Color32,
    pub pressed: egui::Color32,
}

impl InteractiveFill {
    pub const fn uniform(color: egui::Color32) -> Self {
        Self {
            idle: color,
            hovered: color,
            pressed: color,
        }
    }

    fn resolve(self, response: &egui::Response) -> egui::Color32 {
        if response.is_pointer_button_down_on() {
            self.pressed
        } else if response.hovered() {
            self.hovered
        } else {
            self.idle
        }
    }
}

#[derive(Clone, Debug)]
pub struct SurfaceButtonStyle {
    pub size: egui::Vec2,
    pub fill: InteractiveFill,
    pub stroke: egui::Stroke,
    pub focus_stroke: egui::Stroke,
    pub corner_radius: f32,
    pub text_color: egui::Color32,
    pub font_id: egui::FontId,
}

pub struct SurfaceButton<'a> {
    label: &'a str,
    style: SurfaceButtonStyle,
}

impl<'a> SurfaceButton<'a> {
    pub const fn new(label: &'a str, style: SurfaceButtonStyle) -> Self {
        Self { label, style }
    }
}

impl egui::Widget for SurfaceButton<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(self.style.size, egui::Sense::click());
        let painter = ui.painter();
        painter.rect_filled(
            rect,
            self.style.corner_radius,
            self.style.fill.resolve(&response),
        );
        painter.rect_stroke(
            rect,
            self.style.corner_radius,
            self.style.stroke,
            egui::StrokeKind::Inside,
        );
        if response.has_focus() {
            painter.rect_stroke(
                rect.expand(2.0),
                self.style.corner_radius + 2.0,
                self.style.focus_stroke,
                egui::StrokeKind::Outside,
            );
        }
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            self.label,
            self.style.font_id,
            self.style.text_color,
        );
        response
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PaintedIconButtonStyle {
    pub size: egui::Vec2,
    pub fill: InteractiveFill,
    pub stroke: egui::Stroke,
    pub focus_stroke: egui::Stroke,
    pub corner_radius: f32,
    pub icon_color: egui::Color32,
}

pub struct PaintedIconButton<F> {
    style: PaintedIconButtonStyle,
    paint_icon: F,
}

impl<F> PaintedIconButton<F> {
    pub const fn new(style: PaintedIconButtonStyle, paint_icon: F) -> Self {
        Self { style, paint_icon }
    }
}

impl<F> egui::Widget for PaintedIconButton<F>
where
    F: FnOnce(&egui::Painter, egui::Rect, egui::Color32),
{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(self.style.size, egui::Sense::click());
        let painter = ui.painter();
        painter.rect_filled(
            rect,
            self.style.corner_radius,
            self.style.fill.resolve(&response),
        );
        painter.rect_stroke(
            rect,
            self.style.corner_radius,
            self.style.stroke,
            egui::StrokeKind::Inside,
        );
        if response.has_focus() {
            painter.rect_stroke(
                rect.expand(2.0),
                self.style.corner_radius + 2.0,
                self.style.focus_stroke,
                egui::StrokeKind::Outside,
            );
        }
        (self.paint_icon)(painter, rect, self.style.icon_color);
        response
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SegmentedBarMeter {
    heights: [f32; 3],
    color: egui::Color32,
    size: egui::Vec2,
    bar_width: f32,
    gap: f32,
    outlined: bool,
}

impl SegmentedBarMeter {
    pub const fn new(heights: [f32; 3], color: egui::Color32) -> Self {
        Self {
            heights,
            color,
            size: egui::vec2(17.0, 14.0),
            bar_width: 3.0,
            gap: 2.0,
            outlined: false,
        }
    }

    pub const fn size(mut self, size: egui::Vec2) -> Self {
        self.size = size;
        self
    }

    pub const fn bar_width(mut self, width: f32) -> Self {
        self.bar_width = width;
        self
    }

    pub const fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub const fn outlined(mut self, outlined: bool) -> Self {
        self.outlined = outlined;
        self
    }
}

impl egui::Widget for SegmentedBarMeter {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(self.size, egui::Sense::hover());
        for (index, height) in self.heights.into_iter().enumerate() {
            let x = rect.left() + index as f32 * (self.bar_width + self.gap);
            let bar = egui::Rect::from_min_max(
                egui::pos2(x, rect.bottom() - height.min(rect.height())),
                egui::pos2(x + self.bar_width, rect.bottom()),
            );
            if self.outlined {
                ui.painter().rect_stroke(
                    bar,
                    0.0,
                    egui::Stroke::new(1.0, self.color),
                    egui::StrokeKind::Inside,
                );
            } else {
                ui.painter().rect_filled(bar, 0.0, self.color);
            }
        }
        response
    }
}
