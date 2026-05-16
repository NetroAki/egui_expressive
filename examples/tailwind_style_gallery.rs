use eframe::egui;
use egui_expressive::{BreakpointName, Breakpoints, Elevation, Responsive, Tw, TwVariant};

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Tailwind Style Gallery",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(TailwindStyleGallery))),
    )
}

struct TailwindStyleGallery;

impl eframe::App for TailwindStyleGallery {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Frame::default().show(ui, |ui| {
            let width = ui.available_width();
            let breakpoint = Breakpoints::tailwind().classify(width);
            ui.heading("Tailwind-style egui_expressive surfaces");
            ui.label(format!(
                "Current breakpoint: {breakpoint:?} ({width:.0}px available)"
            ));
            ui.add_space(12.0);

            let card_width = *Responsive::new(180.0)
                .md(240.0)
                .lg(300.0)
                .resolve(breakpoint);

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    card(
                        ui,
                        card_width,
                        "p-4 m-2 rounded-lg shadow-md",
                        "Margin + padding + elevation",
                    );
                    card(
                        ui,
                        card_width,
                        "px-6 py-3 mx-3 rounded-xl",
                        "Axis spacing mirrors Tailwind naming",
                    );
                    card(
                        ui,
                        card_width,
                        "w responsive md/lg",
                        "Width responds to breakpoint",
                    );
                    dark_card(
                        ui,
                        card_width,
                        "bg dark surface",
                        "Dark surface with readable text",
                    );
                    accent_card(
                        ui,
                        card_width,
                        "border-2 accent",
                        "Border and text color utilities",
                    );
                    soft_card(
                        ui,
                        card_width,
                        "rounded-2xl shadow-lg",
                        "Large radius + stronger elevation",
                    );
                    effects_card(ui, card_width);
                    transition_card(ui, card_width);
                });
            });
        });
    }
}

fn card(ui: &mut egui::Ui, width: f32, class_label: &str, body: &str) {
    Tw::new()
        .w(width)
        .m(8.0)
        .p(16.0)
        .rounded_lg()
        .shadow(Elevation::Level2)
        .bg(egui::Color32::from_rgb(245, 247, 250))
        .border_1()
        .border_color(egui::Color32::from_rgb(210, 216, 226))
        .show(ui, |ui| {
            ui.strong(class_label);
            ui.label(body);
        });
}

fn dark_card(ui: &mut egui::Ui, width: f32, class_label: &str, body: &str) {
    Tw::new()
        .w(width)
        .mx(8.0)
        .my(10.0)
        .p(16.0)
        .rounded_xl()
        .shadow(Elevation::Level3)
        .bg(egui::Color32::from_rgb(22, 27, 36))
        .text_color(egui::Color32::from_rgb(235, 240, 248))
        .show(ui, |ui| {
            ui.strong(class_label);
            ui.label(body);
        });
}

fn accent_card(ui: &mut egui::Ui, width: f32, class_label: &str, body: &str) {
    Tw::new()
        .w(width)
        .m(8.0)
        .px(20.0)
        .py(14.0)
        .rounded_md()
        .border_2()
        .border_color(egui::Color32::from_rgb(88, 166, 255))
        .text_color(egui::Color32::from_rgb(42, 92, 150))
        .show(ui, |ui| {
            ui.strong(class_label);
            ui.label(body);
        });
}

fn soft_card(ui: &mut egui::Ui, width: f32, class_label: &str, body: &str) {
    Tw::new()
        .w(width)
        .mt(8.0)
        .mr(12.0)
        .mb(16.0)
        .ml(4.0)
        .p(18.0)
        .rounded_2xl()
        .shadow(Elevation::Level4)
        .bg(egui::Color32::from_rgb(252, 248, 238))
        .show(ui, |ui| {
            ui.strong(class_label);
            ui.label(body);
        });
}

fn effects_card(ui: &mut egui::Ui, width: f32) {
    Tw::new()
        .w(width)
        .m(8.0)
        .p(18.0)
        .rounded_2xl()
        .opacity(0.92)
        .bg_gradient_to_r(
            egui::Color32::from_rgb(70, 110, 255),
            egui::Color32::from_rgb(160, 70, 220),
        )
        .drop_shadow(
            egui::vec2(0.0, 6.0),
            10,
            egui::Color32::from_black_alpha(80),
        )
        .ring(2.0, egui::Color32::from_white_alpha(120))
        .show(ui, |ui| {
            ui.colored_label(
                egui::Color32::WHITE,
                "gradient + opacity + drop-shadow + ring",
            );
            ui.colored_label(
                egui::Color32::WHITE,
                "Bounded, deterministic Stage 7 effect proof",
            );
        });
}

fn transition_card(ui: &mut egui::Ui, width: f32) {
    Tw::new()
        .w(width)
        .m(8.0)
        .p(16.0)
        .rounded_lg()
        .bg(egui::Color32::from_rgb(238, 244, 255))
        .transition(0.18)
        .hover(
            Tw::new()
                .w(width)
                .m(8.0)
                .p(16.0)
                .rounded_lg()
                .bg(egui::Color32::from_rgb(210, 228, 255))
                .opacity(0.85)
                .transition(0.18),
        )
        .show_variant(ui, TwVariant::Hover, |ui| {
            ui.strong("transition + reduced-motion aware variant");
            ui.label("Numeric/color transition intent is bounded and documented.");
        });
}

#[allow(dead_code)]
fn breakpoint_name_for_docs(width: f32) -> BreakpointName {
    Breakpoints::tailwind().classify(width)
}
