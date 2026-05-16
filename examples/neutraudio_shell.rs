use eframe::egui;
use egui_expressive::{
    AccentKind, CheckboxField, Elevation, Fader, GridLayout, Knob, Meter, SelectField,
    SelectOption, SurfaceLevel, TextField, Theme, Tw,
};

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Neutraudio Shell Proof",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(NeutraudioShell::default()))),
    )
}

#[derive(Default)]
struct NeutraudioShell {
    search: String,
    plugin: PluginKind,
    snap: bool,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum PluginKind {
    #[default]
    Synth,
    Sampler,
    Effect,
}

impl eframe::App for NeutraudioShell {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        Theme::neutraudio_dark().store(ui.ctx());
        Tw::new()
            .w_full()
            .min_h_screen()
            .bg_surface(SurfaceLevel::Base)
            .text_surface(SurfaceLevel::On)
            .p(12.0)
            .show(ui, |ui| {
                top_bar(ui);
                ui.add_space(8.0);
                main_workspace(ui, self);
                ui.add_space(8.0);
                bottom_dock(ui);
                snap_line_overlay(ui);
            });
    }
}

fn top_bar(ui: &mut egui::Ui) {
    Tw::new()
        .flex()
        .flex_wrap()
        .gap(8.0)
        .px(12.0)
        .py(8.0)
        .rounded_lg()
        .bg_surface(SurfaceLevel::Container)
        .border_b(1.0)
        .border_b_color(egui::Color32::from_rgb(60, 60, 70))
        .show(ui, |ui| {
            ui.strong("NEUTRAUDIO");
            chip(ui, "Play");
            chip(ui, "Stop");
            chip(ui, "Record");
            chip(ui, "120 BPM");
            chip(ui, "4/4");
        });
}

fn main_workspace(ui: &mut egui::Ui, app: &mut NeutraudioShell) {
    GridLayout::columns(3)
        .gap(10.0)
        .egui_grid("neutraudio_shell_main")
        .show(ui, |ui| {
            browser(ui, app);
            arrangement(ui);
            inspector(ui, app);
            ui.end_row();
        });
}

fn browser(ui: &mut egui::Ui, app: &mut NeutraudioShell) {
    panel(ui, "Browser", |ui| {
        TextField::new("Search", &mut app.search)
            .hint("drums, bass, fx")
            .show(ui);
        ui.separator();
        for item in ["Packs", "Projects", "Samples", "Presets", "Automation"] {
            ui.label(format!("▸ {item}"));
        }
    });
}

fn arrangement(ui: &mut egui::Ui) {
    panel(ui, "Workspace", |ui| {
        for lane in 0..5 {
            Tw::new()
                .w_full()
                .h(42.0)
                .my(4.0)
                .rounded_md()
                .bg_alpha(egui::Color32::from_rgb(35, 39, 48), 0.9)
                .border_l(3.0)
                .border_l_color(egui::Color32::from_rgb(239, 68, 68))
                .show(ui, |ui| {
                    ui.label(format!("Lane {}  ━━━ clip / automation / notes", lane + 1));
                });
        }
    });
}

fn inspector(ui: &mut egui::Ui, app: &mut NeutraudioShell) {
    panel(ui, "Inspector", |ui| {
        SelectField::new("Plugin", &mut app.plugin)
            .options([
                SelectOption::new(PluginKind::Synth, "Synth"),
                SelectOption::new(PluginKind::Sampler, "Sampler"),
                SelectOption::new(PluginKind::Effect, "Effect"),
            ])
            .show(ui);
        CheckboxField::new("Snap enabled", &mut app.snap).show(ui);
        ui.separator();
        GridLayout::columns(2)
            .gap(6.0)
            .egui_grid("inspector_knobs")
            .show(ui, |ui| {
                for name in ["Gain", "Pan", "Cutoff", "Drive"] {
                    knob_stub(ui, name);
                    if name == "Pan" || name == "Drive" {
                        ui.end_row();
                    }
                }
            });
    });
}

fn bottom_dock(ui: &mut egui::Ui) {
    Tw::new()
        .w_full()
        .p(10.0)
        .rounded_xl()
        .shadow(Elevation::Level3)
        .bg_surface(SurfaceLevel::Container)
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for strip in ["Kick", "Snare", "Bass", "Pad", "Vox", "FX"] {
                    Tw::new()
                        .w(96.0)
                        .h(132.0)
                        .m(4.0)
                        .p(8.0)
                        .rounded_lg()
                        .bg_surface(SurfaceLevel::Dim)
                        .border_t(2.0)
                        .border_t_color(egui::Color32::from_rgb(34, 211, 238))
                        .cursor_pointer()
                        .show(ui, |ui| {
                            ui.strong(strip);
                            ui.add(Meter::new(0.62).size(egui::vec2(12.0, 48.0)).segments(8));
                            let mut value = 0.0;
                            ui.add(
                                Fader::new(&mut value, -60.0..=6.0).size(egui::vec2(30.0, 60.0)),
                            );
                        });
                }
            });
        });
}

fn panel(ui: &mut egui::Ui, title: &str, contents: impl FnOnce(&mut egui::Ui)) {
    Tw::new()
        .w_full()
        .min_w(180.0)
        .p(10.0)
        .rounded_lg()
        .bg_surface(SurfaceLevel::Container)
        .border(1.0)
        .border_color(egui::Color32::from_rgb(52, 56, 68))
        .show(ui, |ui| {
            ui.strong(title);
            ui.separator();
            contents(ui);
        });
}

fn chip(ui: &mut egui::Ui, label: &str) {
    Tw::new()
        .px(10.0)
        .py(4.0)
        .rounded_full()
        .bg_accent(AccentKind::Primary)
        .text_accent(AccentKind::OnPrimary)
        .show(ui, |ui| {
            ui.label(label);
        });
}

fn knob_stub(ui: &mut egui::Ui, label: &str) {
    let mut value = 0.5;
    Tw::new()
        .w(74.0)
        .p(6.0)
        .rounded_md()
        .bg_surface(SurfaceLevel::Dim)
        .show(ui, |ui| {
            ui.add(Knob::new(&mut value, 0.0..=1.0).size(36.0));
            ui.small(label);
        });
}

fn snap_line_overlay(ui: &mut egui::Ui) {
    Tw::new()
        .id("neutraudio_shell_snap_line")
        .absolute()
        .top(88.0)
        .left(280.0)
        .z(120)
        .pointer_events_none()
        .w(2.0)
        .h(360.0)
        .bg_accent(AccentKind::Secondary)
        .show(ui, |_ui| {});
}
