//! Primitive-first gallery showing reusable controls/displays without app-specific DAW composites.

use eframe::egui;
use egui_expressive::widgets::*;

struct PrimitiveGalleryApp {
    gain: f64,
    pan: f64,
    range_start: f64,
    range_end: f64,
    xy_x: f64,
    xy_y: f64,
    steps: Vec<Vec<StepCell>>,
    search: String,
    toasts: Vec<Toast>,
}

impl Default for PrimitiveGalleryApp {
    fn default() -> Self {
        Self {
            gain: 0.65,
            pan: 0.0,
            range_start: 1.0,
            range_end: 4.0,
            xy_x: 0.5,
            xy_y: 0.5,
            steps: vec![vec![StepCell::default(); 16]; 3],
            search: String::new(),
            toasts: vec![Toast::new("Primitive gallery", 3.0)],
        }
    }
}

impl eframe::App for PrimitiveGalleryApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("egui_expressive primitive gallery");
        ui.horizontal(|ui| {
            ui.add(
                Knob::new(&mut self.gain, 0.0..=1.0)
                    .label("GAIN")
                    .bipolar(false)
                    .wheel_step(0.01)
                    .value_popup(true),
            );
            ui.add(
                Slider::new(&mut self.pan, -1.0..=1.0)
                    .label("PAN")
                    .marks(vec![-1.0, 0.0, 1.0])
                    .value_popup(true),
            );
            ui.add(
                RangeSlider::new(&mut self.range_start, &mut self.range_end, 0.0..=8.0)
                    .size(egui::vec2(180.0, 28.0)),
            );
            ui.add(XYPad::new(&mut self.xy_x, &mut self.xy_y, 0.0..=1.0, 0.0..=1.0).label("XY"));
        });

        ui.separator();
        ui.add(StepCellGrid::new(&mut self.steps, 3, 16).active_col(4));
        ui.separator();
        ui.add(SearchField::new(&mut self.search).hint("Search primitives…"));

        let rect = ui.available_rect_before_wrap().shrink(8.0);
        let grid = GridCanvas::new(4, 32.0, 18.0).subdivisions(4);
        grid.paint_grid(
            ui.painter(),
            rect,
            0,
            8,
            4,
            [egui::Color32::from_gray(45), egui::Color32::from_gray(80)],
        );
        NoteRect::new("note", 1.0, 2.0, 1).paint(ui.painter(), &grid, rect.min);
        AutomationCurve::new(vec![
            AutomationPoint {
                beat: 0.0,
                value: 0.2,
                segment: AutomationSegment::Linear,
            },
            AutomationPoint {
                beat: 2.0,
                value: 0.8,
                segment: AutomationSegment::Smooth,
            },
            AutomationPoint {
                beat: 4.0,
                value: 0.4,
                segment: AutomationSegment::Linear,
            },
        ])
        .paint(ui.painter(), &grid, rect, egui::Color32::LIGHT_BLUE);
        ui.allocate_space(rect.size());
        ToastLayer::new(&mut self.toasts).show(ui.ctx());
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Primitive Gallery",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(PrimitiveGalleryApp::default()))),
    )
}
