//! Step sequencer demo with BPM control.

use eframe::egui;
use egui_expressive::widgets::{Knob, StepGrid};

struct SeqApp {
    steps: Vec<Vec<bool>>,
    bpm: f64,
    swing: f64,
    playhead: usize,
    frame_count: usize,
    step_interval: usize,
}

impl Default for SeqApp {
    fn default() -> Self {
        // 4 rows, 16 steps — classic 4-on-the-floor kick pattern + hi-hats + snares + toms
        let mut steps = vec![vec![false; 16]; 4];

        // Kick: beats 0, 4, 8, 12
        steps[0][0] = true;
        steps[0][4] = true;
        steps[0][8] = true;
        steps[0][12] = true;

        // Hi-hat: every other beat
        for i in 0..16 {
            if i % 2 == 0 {
                steps[1][i] = true;
            }
        }

        // Snare: beats 4, 12
        steps[2][4] = true;
        steps[2][12] = true;

        // Tom: beat 6 and 14
        steps[3][6] = true;
        steps[3][14] = true;

        Self {
            steps,
            bpm: 120.0,
            swing: 0.0,
            playhead: 0,
            frame_count: 0,
            step_interval: 30, // frames per step at 60fps
        }
    }
}

impl eframe::App for SeqApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.frame_count += 1;

        // Advance playhead based on BPM
        let new_interval = (60.0 / self.bpm * 60.0) as usize;
        if new_interval != self.step_interval {
            self.step_interval = new_interval;
        }

        if self.frame_count % self.step_interval.max(1) == 0 {
            self.playhead = (self.playhead + 1) % 16;
        }

        ui.heading("Step Sequencer");
        ui.separator();

        ui.horizontal(|ui| {
            ui.add(
                Knob::new(&mut self.bpm, 60.0..=200.0)
                    .size(56.0)
                    .label("BPM")
                    .default_value(120.0),
            );
            ui.add(
                Knob::new(&mut self.swing, 0.0..=0.5)
                    .size(40.0)
                    .label("SWING")
                    .default_value(0.0),
            );

            ui.separator();

            ui.vertical(|ui| {
                ui.label(format!("BPM: {:.0}", self.bpm));
                ui.label(format!("Swing: {:.0}%", self.swing * 100.0));
                ui.label(format!("Playhead: {}", self.playhead + 1));
            });
        });

        ui.separator();

        // Row labels
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.add(egui::Label::new("KICK").wrap());
                ui.add(egui::Label::new("HIHAT").wrap());
                ui.add(egui::Label::new("SNARE").wrap());
                ui.add(egui::Label::new("TOM").wrap());
            });
            ui.add(
                StepGrid::new(&mut self.steps, 4, 16)
                    .cell_size(egui::vec2(32.0, 32.0))
                    .active_col(self.playhead)
                    .row_colors(vec![
                        egui::Color32::from_rgb(220, 80, 80),   // kick — red
                        egui::Color32::from_rgb(80, 200, 120),  // hihat — green
                        egui::Color32::from_rgb(220, 180, 60),  // snare — yellow
                        egui::Color32::from_rgb(100, 140, 255), // tom — blue
                    ]),
            );
        });

        ui.separator();

        ui.label("Instructions: Click to toggle steps. Shift+drag for fine control. Double-click to reset BPM.");

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Step Sequencer",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(SeqApp::default()))),
    )
}
