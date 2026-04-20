//! DAW control strip demo — demonstrates Knob, Fader, Meter, and StepGrid.

use eframe::egui;
use egui_expressive::widgets::{Fader, Knob, Meter, Orientation, StepGrid};

struct DawApp {
    gain: f64,
    pan: f64,
    volume: f64,
    vu_level: f32,
    peak: f32,
    mute_steps: Vec<Vec<bool>>,
    playhead: usize,
    frame_count: usize,
}

impl Default for DawApp {
    fn default() -> Self {
        Self {
            gain: -6.0,
            pan: 0.0,
            volume: 0.75,
            vu_level: 0.3,
            peak: 0.4,
            mute_steps: vec![vec![false; 16]; 8],
            playhead: 0,
            frame_count: 0,
        }
    }
}

impl eframe::App for DawApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // Simulate VU meter movement
        self.frame_count += 1;
        self.vu_level = 0.3 + 0.4 * (self.frame_count as f32 * 0.05).sin().abs();
        if self.vu_level > self.peak {
            self.peak = self.vu_level;
        }
        if self.frame_count.is_multiple_of(60) {
            self.peak *= 0.95;
        }
        if self.frame_count.is_multiple_of(120) {
            self.playhead = (self.playhead + 1) % 16;
        }

        {
            ui.heading("DAW Control Strip");
            ui.separator();
            ui.horizontal(|ui| {
                // Channel 1
                ui.vertical(|ui| {
                    ui.label("CH 1");
                    ui.add(
                        Knob::new(&mut self.gain, -60.0..=6.0)
                            .size(48.0)
                            .label("GAIN"),
                    );
                    ui.add(
                        Knob::new(&mut self.pan, -1.0..=1.0)
                            .size(32.0)
                            .label("PAN")
                            .default_value(0.0),
                    );
                    ui.add(
                        Meter::new(self.vu_level)
                            .peak(self.peak)
                            .size(egui::vec2(20.0, 120.0)),
                    );
                    ui.add(
                        Fader::new(&mut self.volume, 0.0..=1.0)
                            .size(egui::vec2(40.0, 120.0))
                            .orientation(Orientation::Vertical),
                    );
                });

                ui.separator();

                // Channel 2 (static example values)
                ui.vertical(|ui| {
                    let mut gain2 = -12.0;
                    let mut pan2 = 0.3;
                    let mut vol2 = 0.6;
                    let vu2 = 0.5 + 0.2 * (self.frame_count as f32 * 0.07 + 1.0).sin().abs();
                    let peak2 = 0.6;

                    ui.label("CH 2");
                    ui.add(Knob::new(&mut gain2, -60.0..=6.0).size(48.0).label("GAIN"));
                    ui.add(
                        Knob::new(&mut pan2, -1.0..=1.0)
                            .size(32.0)
                            .label("PAN")
                            .default_value(0.0),
                    );
                    ui.add(Meter::new(vu2).peak(peak2).size(egui::vec2(20.0, 120.0)));
                    ui.add(
                        Fader::new(&mut vol2, 0.0..=1.0)
                            .size(egui::vec2(40.0, 120.0))
                            .orientation(Orientation::Vertical),
                    );
                });
            });

            ui.separator();

            ui.label(format!(
                "Step Sequencer (8x16) — Playhead: {}",
                self.playhead
            ));
            ui.add(
                StepGrid::new(&mut self.mute_steps, 8, 16)
                    .cell_size(egui::vec2(28.0, 28.0))
                    .active_col(self.playhead),
            );

            ui.separator();
            ui.horizontal(|ui| {
                ui.label(format!("Gain: {:.1} dB", self.gain));
                ui.label(format!("Pan: {:.2}", self.pan));
                ui.label(format!("Volume: {:.0}%", self.volume * 100.0));
                ui.label(format!("VU: {:.0}%", self.vu_level * 100.0));
            });
        }

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "DAW Strip",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(DawApp::default()))),
    )
}
