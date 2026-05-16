use eframe::egui;
use egui_expressive::{
    AccessibilityMeta, AccessibilityRole, FocusRing, MotionPolicy, Tw, TwVariants,
};

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "State + Accessibility Gallery",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(StateAccessibilityGallery))),
    )
}

struct StateAccessibilityGallery;

impl eframe::App for StateAccessibilityGallery {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("Tailwind-style state variants + accessibility baseline");
        ui.label(
            "Click the custom card to focus it. Hover/press/focus resolve like Tailwind variants.",
        );
        ui.add_space(12.0);

        let variants = Tw::new()
            .p(16.0)
            .rounded_lg()
            .bg(egui::Color32::from_rgb(31, 41, 55))
            .text_color(egui::Color32::WHITE)
            .hover(Tw::new().bg(egui::Color32::from_rgb(55, 65, 81)))
            .pressed(Tw::new().bg(egui::Color32::from_rgb(17, 24, 39)))
            .focus(
                Tw::new()
                    .border_2()
                    .border_color(egui::Color32::from_rgb(96, 165, 250)),
            )
            .disabled(Tw::new().bg(egui::Color32::from_rgb(107, 114, 128)));

        state_card(
            ui,
            &variants,
            AccessibilityMeta::new(AccessibilityRole::Button, "Interactive state card")
                .description("Demonstrates hover, pressed, focus, and disabled variants."),
        );

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            let button = ui.button("Focusable egui button");
            FocusRing::themed(ui.ctx()).paint_if(ui, &button);
            if button.clicked() {
                button.request_focus();
            }
        });

        let motion = MotionPolicy::from_ctx(ui.ctx());
        ui.label(format!(
            "Motion policy: {:?}, 250ms transition becomes {:.0}ms",
            motion.preference,
            motion.duration(0.25) * 1000.0
        ));
    }
}

fn state_card(ui: &mut egui::Ui, variants: &TwVariants, meta: AccessibilityMeta) {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(320.0, 96.0), egui::Sense::click());
    if response.clicked() {
        response.request_focus();
    }

    let style = variants.resolve_response(&response, false, meta.disabled);
    let bg = style.bg.unwrap_or(egui::Color32::from_gray(48));
    let radius = egui::CornerRadius::same(style.border_radius.min(255.0) as u8);
    ui.painter().rect_filled(rect, radius, bg);
    if style.border_width > 0.0 {
        ui.painter().rect_stroke(
            rect,
            radius,
            egui::Stroke::new(
                style.border_width,
                style.border_color.unwrap_or(egui::Color32::WHITE),
            ),
            egui::StrokeKind::Outside,
        );
    }
    FocusRing::default().paint_if(ui, &response);

    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        format!("{} ({})", meta.label, meta.role.as_str()),
        egui::FontId::proportional(15.0),
        style.fg.unwrap_or(egui::Color32::WHITE),
    );
}
