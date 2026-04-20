//! Timeline / large canvas demo with viewport culling.

use eframe::egui;
use egui_expressive::surface::LargeCanvas;

struct TimelineApp {
    id: egui::Id,
    beats: usize,
    beat_width: f32,
    tracks: Vec<Vec<Option<(usize, usize)>>>, // track → clip as (start_beat, len_beats)
    frame_count: usize,
    playhead_beat: f32,
    bpm: f64,
}

impl Default for TimelineApp {
    fn default() -> Self {
        let beats = 512;
        let beat_width = 100.0; // 100px per beat = 51200px total
        let track_count = 8;

        // Create some demo clips
        let mut tracks = vec![Vec::new(); track_count];

        // Track 0: Lead vocal — verse sections
        tracks[0].push(Some((0, 16))); // intro
        tracks[0].push(Some((32, 16))); // verse 1
        tracks[0].push(Some((64, 16))); // verse 2
        tracks[0].push(Some((96, 8))); // bridge

        // Track 1: Drums — verse and chorus
        tracks[1].push(Some((0, 64))); // full verse/chorus
        tracks[1].push(Some((80, 32))); // extended chorus

        // Track 2: Bass
        tracks[2].push(Some((0, 48)));
        tracks[2].push(Some((64, 48)));

        // Track 3: Guitars
        tracks[3].push(Some((16, 16)));
        tracks[3].push(Some((48, 16)));
        tracks[3].push(Some((80, 32)));

        // Track 4-7: Synths, pads, etc. (some empty)
        tracks[5].push(Some((0, 96)));

        Self {
            id: egui::Id::new("timeline"),
            beats,
            beat_width,
            tracks,
            frame_count: 0,
            playhead_beat: 0.0,
            bpm: 120.0,
        }
    }
}

impl eframe::App for TimelineApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.frame_count += 1;

        // Advance playhead
        let beats_per_frame = (self.bpm / 60.0 / 60.0) as f32;
        self.playhead_beat += beats_per_frame;
        if self.playhead_beat >= self.beats as f32 {
            self.playhead_beat = 0.0;
        }

        let beat_width = self.beat_width;
        let total_width = self.beats as f32 * beat_width;
        let track_height = 60.0;
        let total_height = self.tracks.len() as f32 * track_height;

        {
            ui.heading("Timeline");
            ui.horizontal(|ui| {
                ui.label(format!("BPM: {:.0}", self.bpm));
                ui.label(format!("Playhead: beat {:.1}", self.playhead_beat));
                ui.label(format!(
                    "Canvas: {:.0} x {:.0} px",
                    total_width, total_height
                ));
                ui.label("(Scroll/pan to navigate — zoom with scroll wheel)");
            });
            ui.separator();

            // The large canvas
            LargeCanvas::new(self.id, egui::vec2(total_width, total_height))
                .zoom_range(0.05, 10.0)
                .show(ui, |ui, origin, pan_zoom, culler| {
                    let painter = ui.painter();

                    // Draw grid and clips
                    let visible_rows = culler.visible_rows(track_height, self.tracks.len());
                    let visible_cols = culler.visible_cols(beat_width, self.beats);

                    // Draw beat markers (only visible ones)
                    for beat in visible_cols.clone() {
                        let x = beat as f32 * beat_width;
                        let screen_x = pan_zoom.to_screen(egui::pos2(x, 0.0), origin).x;

                        // Determine bar color
                        let is_bar = beat % 16 == 0;
                        let is_beat = beat % 4 == 0;
                        let _stroke_width = if is_bar {
                            2.0
                        } else if is_beat {
                            1.0
                        } else {
                            0.5
                        };
                        let color = if is_bar {
                            egui::Color32::from_rgba_unmultiplied(100, 100, 120, 180)
                        } else {
                            egui::Color32::from_rgba_unmultiplied(50, 50, 70, 100)
                        };

                        for row in visible_rows.clone() {
                            let y = row as f32 * track_height;
                            let screen_y_top = pan_zoom.to_screen(egui::pos2(0.0, y), origin).y;
                            let screen_y_bottom = pan_zoom
                                .to_screen(egui::pos2(0.0, y + track_height), origin)
                                .y;

                            let line_rect = egui::Rect::from_min_max(
                                egui::pos2(screen_x, screen_y_top),
                                egui::pos2(screen_x + 1.0, screen_y_bottom),
                            );
                            painter.rect_filled(line_rect, 0.0, color);
                        }
                    }

                    // Draw track separators
                    for row in visible_rows.clone() {
                        let y = row as f32 * track_height;
                        let screen_y = pan_zoom.to_screen(egui::pos2(0.0, y), origin).y;
                        let screen_x_left = origin.x;
                        let screen_x_right = origin.x + total_width * pan_zoom.scale;

                        painter.line_segment(
                            [
                                egui::pos2(screen_x_left, screen_y),
                                egui::pos2(screen_x_right, screen_y),
                            ],
                            egui::Stroke::new(
                                1.0,
                                egui::Color32::from_rgba_unmultiplied(60, 60, 80, 150),
                            ),
                        );
                    }

                    // Draw clips
                    for (track_idx, track) in self.tracks.iter().enumerate() {
                        if !visible_rows.contains(&track_idx) {
                            continue;
                        }

                        let track_y = track_idx as f32 * track_height;

                        for clip in track.iter().flatten() {
                            let (start_beat, len_beats) = *clip;
                            let end_beat = start_beat + len_beats;

                            // Cull: skip if entirely outside visible columns
                            if end_beat < visible_cols.start || start_beat > visible_cols.end {
                                continue;
                            }

                            let clip_x = start_beat as f32 * beat_width;
                            let clip_width = len_beats as f32 * beat_width;

                            let logical_rect = egui::Rect::from_min_size(
                                egui::pos2(clip_x, track_y + 4.0),
                                egui::vec2(clip_width, track_height - 8.0),
                            );

                            let screen_rect = culler.rect_to_screen(logical_rect);

                            // Clip color by track index
                            let colors = [
                                egui::Color32::from_rgb(80, 120, 200),  // blue
                                egui::Color32::from_rgb(200, 100, 80),  // red
                                egui::Color32::from_rgb(80, 180, 120),  // green
                                egui::Color32::from_rgb(180, 160, 60),  // yellow
                                egui::Color32::from_rgb(160, 80, 180),  // purple
                                egui::Color32::from_rgb(80, 180, 180),  // cyan
                                egui::Color32::from_rgb(200, 140, 80),  // orange
                                egui::Color32::from_rgb(140, 140, 160), // gray
                            ];
                            let color = colors[track_idx % colors.len()];

                            painter.rect_filled(screen_rect, 4.0, color.linear_multiply(0.8));
                            painter.rect_stroke(
                                screen_rect,
                                4.0,
                                egui::Stroke::new(1.5, color),
                                egui::StrokeKind::Outside,
                            );
                        }
                    }

                    // Draw playhead
                    let playhead_x = self.playhead_beat * beat_width;
                    let screen_playhead = pan_zoom.to_screen(egui::pos2(playhead_x, 0.0), origin);

                    painter.line_segment(
                        [
                            egui::pos2(screen_playhead.x, origin.y),
                            egui::pos2(screen_playhead.x, origin.y + total_height * pan_zoom.scale),
                        ],
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 60, 60)),
                    );
                });
        }

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Timeline",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(TimelineApp::default()))),
    )
}
