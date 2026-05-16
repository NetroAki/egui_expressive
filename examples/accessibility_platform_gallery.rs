use eframe::egui;
use egui_expressive::{
    AccessibilityMeta, AccessibilityRole, ClipboardCommand, DisplayScale, DroppedFileDescriptor,
    FeedbackMessage, FeedbackProgress, FeedbackSeverity, FocusRing, InputTextContract, LiveRegion,
    PlatformDropBatch, RovingFocusDirection, RovingFocusGroup, RovingFocusItem,
    SystemThemePreference, TextDirection,
};

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Accessibility + Platform Gallery",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(AccessibilityPlatformGallery::default()))),
    )
}

#[derive(Default)]
struct AccessibilityPlatformGallery {
    selected: Option<egui::Id>,
}

impl eframe::App for AccessibilityPlatformGallery {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

impl AccessibilityPlatformGallery {
    fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Stage 8 accessibility, i18n, and platform contracts");
        ui.label("This example keeps platform side effects app-owned while surfacing semantics.");

        let save = egui::Id::new("save_tab");
        let export = egui::Id::new("export_tab");
        let disabled = egui::Id::new("disabled_tab");
        let roving = RovingFocusGroup::new()
            .item(RovingFocusItem::new(save))
            .item(RovingFocusItem::new(disabled).disabled(true))
            .item(RovingFocusItem::new(export));
        if self.selected.is_none() {
            self.selected = Some(save);
        }
        let direction = ui.input(|input| {
            if input.key_pressed(egui::Key::ArrowRight) {
                Some(RovingFocusDirection::Next)
            } else if input.key_pressed(egui::Key::ArrowLeft) {
                Some(RovingFocusDirection::Previous)
            } else if input.key_pressed(egui::Key::Home) {
                Some(RovingFocusDirection::First)
            } else if input.key_pressed(egui::Key::End) {
                Some(RovingFocusDirection::Last)
            } else {
                None
            }
        });
        if let Some(direction) = direction {
            self.selected = roving.resolve(self.selected, direction);
        }
        ui.label(format!(
            "Use ←/→/Home/End: roving focus skips disabled item; selected: {:?}",
            self.selected
        ));

        let button = ui.button("Focusable action with visible ring");
        FocusRing::themed(ui.ctx()).paint_if(ui, &button);

        let error = FeedbackMessage::new("export_failed", "Export failed")
            .severity(FeedbackSeverity::Error);
        let progress = FeedbackProgress::new("sync", "Syncing", Some(0.5));
        let alert_meta = error.accessibility_meta(AccessibilityRole::Alert);
        let progress_meta = progress.accessibility_meta();
        ui.label(format!(
            "Live regions: {} => {}, {} => {}",
            alert_meta.role.as_str(),
            alert_meta.live_region.as_ref().unwrap().politeness.as_str(),
            progress_meta.role.as_str(),
            progress_meta.value.as_deref().unwrap_or("indeterminate")
        ));

        let input_contract =
            InputTextContract::default().text_direction(TextDirection::RightToLeft);
        ui.label(format!(
            "Text direction: {:?}; platform review required: {}",
            input_contract.text_direction,
            input_contract.requires_platform_review()
        ));

        let copy = ClipboardCommand::copy_text("Visible label");
        let drop_batch = PlatformDropBatch::new(
            egui::pos2(24.0, 48.0),
            [DroppedFileDescriptor::new("demo-file", "design.json")],
        );
        let scale = DisplayScale::new(ui.ctx().pixels_per_point());
        ui.label(format!(
            "Clipboard log-safe: {}; dropped files: {}; 12pt => {:.1}px; dark preference: {:?}",
            copy.should_log_value(),
            drop_batch.files.len(),
            scale.logical_to_physical(12.0),
            SystemThemePreference::System.prefers_dark()
        ));

        let meta = AccessibilityMeta::status("Background sync complete")
            .live_region(LiveRegion::polite("Background sync complete"));
        ui.label(format!(
            "Custom status metadata role: {}",
            meta.role.as_str()
        ));
    }
}
