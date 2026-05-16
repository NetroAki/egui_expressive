//! Cross-framework compatibility gallery: familiar names, same primitives.

use eframe::egui;
use egui_expressive::compat::{html, kivy, qt, swiftui, tkinter};

struct CompatGalleryApp {
    html_gain: f64,
    swift_gain: f64,
    qt_gain: f64,
    tk_gain: f64,
    kivy_gain: f64,
    search: String,
}

impl Default for CompatGalleryApp {
    fn default() -> Self {
        Self {
            html_gain: 0.25,
            swift_gain: 0.35,
            qt_gain: 0.45,
            tk_gain: 0.55,
            kivy_gain: 0.65,
            search: String::new(),
        }
    }
}

impl eframe::App for CompatGalleryApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("Compatibility aliases");
        ui.label("Different ecosystem names redirect to the same egui_expressive primitives.");

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("HTML/Electron");
                ui.add(html::input_range(&mut self.html_gain, 0.0..=1.0).label("<input range>"));
                ui.add(html::input_search(&mut self.search).hint("<input search>"));
                let _props = html::DomProps::default()
                    .id("gain")
                    .class("control")
                    .data_action("set-gain");
            });
            ui.vertical(|ui| {
                ui.label("SwiftUI");
                ui.add(swiftui::slider(&mut self.swift_gain, 0.0..=1.0).label("Slider"));
                let _mods = swiftui::ViewModifiers::default()
                    .padding(8.0)
                    .cornerRadius(6.0)
                    .help("SwiftUI-style modifiers");
            });
            ui.vertical(|ui| {
                ui.label("PyQt/PySide");
                ui.add(qt::q_slider(&mut self.qt_gain, 0.0..=1.0).label("QSlider"));
                let _props = qt::QtWidgetProps::new()
                    .setObjectName("gainSlider")
                    .setToolTip("Qt-style props");
            });
            ui.vertical(|ui| {
                ui.label("Tkinter");
                ui.add(tkinter::scale(&mut self.tk_gain, 0.0..=1.0).label("Scale"));
                let _opts = tkinter::TkOptions::default()
                    .text("Gain")
                    .fill(tkinter::BOTH)
                    .expand(true);
            });
            ui.vertical(|ui| {
                ui.label("Kivy");
                ui.add(kivy::slider(&mut self.kivy_gain, 0.0..=1.0).label("Slider"));
                let _props = kivy::KivyProps::default()
                    .text("Gain")
                    .size_hint(Some(1.0), None);
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Compatibility Gallery",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(CompatGalleryApp::default()))),
    )
}
