use eframe::egui;
use egui_expressive::{Elevation, GridLayout, PositionMode, Tw};

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Layout Grid Position Gallery",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(LayoutGridPositionGallery))),
    )
}

struct LayoutGridPositionGallery;

impl eframe::App for LayoutGridPositionGallery {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("Tailwind-like layout utilities");
        ui.label("Grid, position, directional borders, flex wrapping, overflow, cursor, and arbitrary sizing are represented as readable Tw utilities.");
        ui.add_space(12.0);

        Tw::new()
            .w_pct(92.0)
            .max_w_vw(92.0)
            .p(16.0)
            .rounded_xl()
            .shadow(Elevation::Level2)
            .bg(egui::Color32::from_rgb(24, 28, 36))
            .text_color(egui::Color32::from_rgb(232, 237, 245))
            .show(ui, |ui| {
                ui.strong("w-[92%] max-w-[92vw] p-4 rounded-xl shadow");
                ui.label("Arbitrary percentage and viewport sizing without raw CSS strings.");
            });

        ui.add_space(12.0);

        let grid_spec = GridLayout::columns(3).gap(8.0);
        grid_spec
            .egui_grid("layout_grid_position_gallery")
            .show(ui, |ui| {
                for index in 0..6 {
                    Tw::new()
                        .p(10.0)
                        .rounded_md()
                        .border_l(2.0)
                        .border_l_color(egui::Color32::from_rgb(88, 166, 255))
                        .bg_alpha(egui::Color32::from_rgb(40, 52, 70), 0.9)
                        .cursor_pointer()
                        .show(ui, |ui| {
                            ui.label(format!("grid cell {index}"));
                        });
                    if (index + 1) % grid_spec.columns == 0 {
                        ui.end_row();
                    }
                }
            });

        ui.add_space(12.0);
        let overlay_style = Tw::new()
            .absolute()
            .inset(8.0)
            .translate_x(12.0)
            .z(20)
            .pointer_events_none()
            .overflow_clip();
        ui.label(format!(
            "position={:?}, inset={:?}, translate={:?}, z={:?}",
            overlay_style.position.mode,
            overlay_style.position.inset,
            overlay_style.position.translate,
            overlay_style.position.z_index
        ));
        assert_eq!(overlay_style.position.mode, PositionMode::Absolute);

        ui.horizontal_wrapped(|ui| {
            for label in ["flex-wrap", "flex-1", "shrink-0", "gap-x", "gap-y"] {
                Tw::new()
                    .flex()
                    .flex_wrap()
                    .gap_x(8.0)
                    .gap_y(4.0)
                    .px(12.0)
                    .py(6.0)
                    .rounded_full()
                    .bg(egui::Color32::from_rgb(235, 241, 255))
                    .text_color(egui::Color32::from_rgb(30, 66, 120))
                    .show(ui, |ui| {
                        ui.label(label);
                    });
            }
        });
    }
}
