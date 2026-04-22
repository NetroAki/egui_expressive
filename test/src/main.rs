mod colors;

use colors::*;
use egui::*;
use egui_expressive::{
    ChannelStrip, ClipKind, DotState, PanZoom,
    Ruler, StepGrid, TabBar, TimelineClip, ToggleDot, TransportButton, TransportKind,
    TreeNode, TreeView, Waveform,
};

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

struct MixerChannel {
    name: String,
    color: Color32,
    volume: f64,
    pan: f64,
    mute_state: DotState,
    solo_state: DotState,
    meter_l: f32,
    meter_r: f32,
}

struct Generator {
    name: String,
    color: Color32,
    steps: Vec<Vec<bool>>,
    mute_state: DotState,
}

struct PlaylistClipData {
    kind: ClipKind,
    title: String,
    track: usize,
    start_beat: f32,
    len_beats: f32,
}

struct App {
    // Panel visibility
    show_browser: bool,
    show_playlist: bool,
    show_channel_rack: bool,
    show_mixer: bool,
    show_devices: bool,
    // Transport
    is_playing: bool,
    is_recording: bool,
    metronome_on: bool,
    loop_on: bool,
    playhead_beat: f32,
    tempo: f64,
    auto_tempo: bool,
    // Browser
    browser_tab: usize,
    browser_search_open: bool,
    browser_search_query: String,
    browser_filter: usize,
    // Playlist
    playlist_tool: usize,
    snap_mode: usize,
    playlist_pan_zoom: PanZoom,
    lane_mute_solo: Vec<DotState>,
    // Channel rack
    swing: f32,
    actions_visible: bool,
    // Mixer
    mixer: Vec<MixerChannel>,
    selected_mixer_channel: usize,
    // Generators & clips
    generators: Vec<Generator>,
    clips: Vec<PlaylistClipData>,
    // Layout
    browser_width: f32,
    channel_rack_height: f32,
    bottom_height: f32,
}

impl App {
    fn new() -> Self {
        let mixer = vec![
            mc("Drums", MIXER_COLORS[0]),
            mc("Bass", MIXER_COLORS[1]),
            mc("Vocals", MIXER_COLORS[2]),
            mc("Synth", MIXER_COLORS[3]),
            mc("Master", MASTER_COLOR),
        ];
        let generators = vec![
            gen("Kick", MIXER_COLORS[0], &[true,false,false,false, true,false,false,false, true,false,false,false, true,false,false,false]),
            gen("Snare", MIXER_COLORS[1], &[false,false,false,false, true,false,false,false, false,false,false,false, true,false,false,false]),
            gen("HiHat", MIXER_COLORS[2], &[true,false,true,false, true,false,true,false, true,false,true,false, true,false,true,false]),
            gen("Clap", MIXER_COLORS[3], &[false,false,false,false, true,false,false,false, false,false,false,false, true,false,false,true]),
        ];
        let clips = vec![
            PlaylistClipData { kind: ClipKind::Pattern, title: "Drums".into(), track: 0, start_beat: 1.0, len_beats: 4.0 },
            PlaylistClipData { kind: ClipKind::Pattern, title: "Synth".into(), track: 0, start_beat: 5.0, len_beats: 3.5 },
            PlaylistClipData { kind: ClipKind::Audio, title: "Vocals Take 3".into(), track: 1, start_beat: 3.5, len_beats: 5.0 },
            PlaylistClipData { kind: ClipKind::Automation, title: "Filter Cutoff".into(), track: 2, start_beat: 1.5, len_beats: 5.0 },
            PlaylistClipData { kind: ClipKind::Automation, title: "Reverb Mix".into(), track: 2, start_beat: 9.0, len_beats: 3.0 },
            PlaylistClipData { kind: ClipKind::Pattern, title: "Bass".into(), track: 3, start_beat: 6.0, len_beats: 3.0 },
        ];
        Self {
            show_browser: true, show_playlist: true, show_channel_rack: true,
            show_mixer: true, show_devices: true,
            is_playing: false, is_recording: false, metronome_on: false,
            loop_on: false, playhead_beat: 4.375, tempo: 120.0, auto_tempo: false,
            browser_tab: 0, browser_search_open: false,
            browser_search_query: String::new(), browser_filter: 0,
            playlist_tool: 0, snap_mode: 7,
            playlist_pan_zoom: PanZoom::new(),
            lane_mute_solo: vec![DotState::Off; 10],
            swing: 0.0, actions_visible: false,
            mixer, selected_mixer_channel: 0,
            generators, clips,
            browser_width: 256.0, channel_rack_height: 208.0, bottom_height: 288.0,
        }
    }
}

fn mc(name: &str, color: Color32) -> MixerChannel {
    MixerChannel {
        name: name.into(), color, volume: 0.75, pan: 0.0,
        mute_state: DotState::Off, solo_state: DotState::Off,
        meter_l: 0.0, meter_r: 0.0,
    }
}

fn gen(name: &str, color: Color32, pattern: &[bool]) -> Generator {
    let steps = vec![pattern.to_vec()];
    Generator { name: name.into(), color, steps, mute_state: DotState::Off }
}

// ---------------------------------------------------------------------------
// main + eframe::App
// ---------------------------------------------------------------------------

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Neutraudio",
        eframe::NativeOptions {
            viewport: ViewportBuilder::default()
                .with_inner_size([1440.0, 900.0])
                .with_title("Neutraudio"),
            ..Default::default()
        },
        Box::new(|_| Ok(Box::new(App::new()))),
    )
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        setup_style(ui.ctx());

        // Topbar
        Panel::top("topbar").exact_size(48.0).frame(topbar_frame()).show_inside(ui, |ui| {
            self.topbar(ui);
        });

        // Bottom dock
        let bh = if self.show_mixer || self.show_devices { self.bottom_height } else { 0.0 };
        Panel::bottom("bottom").exact_size(bh).frame(panel_frame()).show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if self.show_mixer { self.mixer_panel(ui); }
                if self.show_devices { self.devices_panel(ui); }
            });
        });

        // Central workspace
        CentralPanel::default().frame(Frame::new().fill(SURFACE_950)).show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if self.show_browser {
                    let w = self.browser_width;
                    ui.push_id("browser", |ui| {
                        ui.set_min_size(vec2(w, ui.available_height()));
                        ui.set_max_size(vec2(w, ui.available_height()));
                        self.browser_panel(ui);
                    });
                    let (rect, rsp) = ui.allocate_exact_size(vec2(6.0, ui.available_height()), Sense::drag());
                    if rsp.dragged() { self.browser_width = (self.browser_width + rsp.drag_delta().x).clamp(180.0, 400.0); }
                    if rsp.hovered() || rsp.dragged() { ui.painter().rect_filled(rect, 0.0, ACCENT_GLOW.linear_multiply(0.15)); }
                }
                ui.vertical(|ui| {
                    if self.show_playlist {
                        let pl_h = if self.show_channel_rack {
                            (ui.available_height() - self.channel_rack_height - 6.0).max(100.0)
                        } else {
                            ui.available_height()
                        };
                        ui.push_id("playlist", |ui| {
                            ui.set_min_size(vec2(ui.available_width(), pl_h));
                            self.playlist_panel(ui);
                        });
                    }
                    if self.show_channel_rack {
                        let (rect, rsp) = ui.allocate_exact_size(vec2(ui.available_width(), 6.0), Sense::drag());
                        if rsp.dragged() { self.channel_rack_height = (self.channel_rack_height - rsp.drag_delta().y).clamp(120.0, 400.0); }
                        if rsp.hovered() || rsp.dragged() { ui.painter().rect_filled(rect, 0.0, ACCENT_GLOW.linear_multiply(0.15)); }
                        ui.push_id("cr", |ui| {
                            ui.set_min_size(vec2(ui.available_width(), self.channel_rack_height));
                            self.channel_rack_panel(ui);
                        });
                    }
                });
            });
        });
    }
}

// ---------------------------------------------------------------------------
// Shared frames
// ---------------------------------------------------------------------------

fn topbar_frame() -> Frame {
    Frame { fill: SURFACE_900, stroke: Stroke::new(1.0, SURFACE_800), inner_margin: Margin::symmetric(12, 4), ..Frame::default() }
}
fn panel_frame() -> Frame {
    Frame { fill: SURFACE_900, stroke: Stroke::new(1.0, SURFACE_800), ..Frame::default() }
}
fn inset_frame() -> Frame {
    Frame { fill: SURFACE_950, stroke: Stroke::new(1.0, SURFACE_800), inner_margin: Margin::same(4), ..Frame::default() }
}
fn title_bar_frame() -> Frame {
    Frame { fill: TITLE_BAR_BG, stroke: Stroke::new(1.0, SURFACE_800), inner_margin: Margin::symmetric(8, 3), ..Frame::default() }
}

fn setup_style(ctx: &Context) {
    let mut s = (*ctx.global_style()).clone();
    s.visuals = Visuals {
        dark_mode: true,
        panel_fill: SURFACE_900,
        window_fill: SURFACE_900,
        extreme_bg_color: SURFACE_950,
        ..Visuals::dark()
    };
    s.spacing.item_spacing = vec2(4.0, 3.0);
    ctx.set_global_style(s);
}

// ---------------------------------------------------------------------------
// Topbar  (1:1 with HTML mockup)
// ---------------------------------------------------------------------------

impl App {
    fn topbar(&mut self, ui: &mut Ui) {
        ui.horizontal_centered(|ui| {
            // Logo
            ui.label(RichText::new("◆ Neutraudio").color(SURFACE_50).size(14.0).strong());

            // Menus (File/Edit/View/Options in inset)
            inset_frame().show(ui, |ui| {
                for menu in ["File", "Edit", "View", "Options"] {
                    let _ = ui.button(RichText::new(menu).size(10.0).color(SURFACE_400)).on_hover_text(menu);
                }
            });

            // Context hint
            inset_frame().show(ui, |ui| {
                ui.set_min_size(vec2(170.0, 0.0));
                ui.vertical(|ui| {
                    ui.label(RichText::new("READY").size(8.0).color(SURFACE_400).strong());
                    ui.label(RichText::new("").size(10.0).color(SURFACE_200).monospace());
                });
            });

            // Panel toggles (Browser/Rack/Playlist/Mixer/Devices)
            inset_frame().show(ui, |ui| {
                let panels: [(&str, &mut bool); 5] = [
                    ("< Browser", &mut self.show_browser),
                    ("■ Rack", &mut self.show_channel_rack),
                    ("≡ Playlist", &mut self.show_playlist),
                    (" sliders", &mut self.show_mixer),
                    ("+ Devices", &mut self.show_devices),
                ];
                for (label, active) in panels {
                    let color = if *active { ACCENT_GLOW } else { SURFACE_400 };
                    if ui.button(RichText::new(label).size(10.0).color(color)).clicked() {
                        *active = !*active;
                    }
                }
            });

            // Transport (Stop/Play/Record/Metronome)
            inset_frame().show(ui, |ui| {
                // Stop — plain button, clicking stops playback
                let stop_col = if !self.is_playing { ACCENT_GLOW } else { SURFACE_400 };
                if ui.button(RichText::new("■").size(14.0).color(stop_col)).clicked() {
                    self.is_playing = false;
                }
                ui.add(TransportButton::new(TransportKind::Play, &mut self.is_playing).size(24.0));
                ui.add(TransportButton::new(TransportKind::Record, &mut self.is_recording).size(24.0));
                ui.add(TransportButton::new(TransportKind::Metronome, &mut self.metronome_on).size(24.0));
            });

            // Separator
            let (sep, _) = ui.allocate_exact_size(vec2(1.0, 24.0), Sense::hover());
            ui.painter().rect_filled(sep, 0.0, SURFACE_800);

            // Time display
            ui.vertical(|ui| {
                ui.label(RichText::new("001:02:04:960").size(18.0).color(ACCENT_GLOW).monospace().strong());
                ui.label(RichText::new("Bars : Beats : Ticks").size(8.0).color(SURFACE_500).monospace());
                // Loop pill
                if self.loop_on {
                    ui.horizontal(|ui| {
                        let (rect, _) = ui.allocate_exact_size(vec2(60.0, 14.0), Sense::click());
                        ui.painter().rect_filled(rect, 7.0, SURFACE_950);
                        ui.painter().rect_stroke(rect, 7.0, Stroke::new(1.0, ACCENT_GLOW.linear_multiply(0.3)), StrokeKind::Outside);
                        ui.painter().text(rect.center(), Align2::CENTER_CENTER, "Loop 1-1",
                            FontId::proportional(8.0), ACCENT_GLOW.linear_multiply(0.9));
                    });
                }
            });

            // Separator
            let (sep, _) = ui.allocate_exact_size(vec2(1.0, 24.0), Sense::hover());
            ui.painter().rect_filled(sep, 0.0, SURFACE_800);

            // Tempo + AUTO pill + Time sig
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("{:.2}", self.tempo)).size(13.0).color(SURFACE_100).monospace());
                        ui.label(RichText::new("BPM").size(8.0).color(SURFACE_500));
                    });
                });

                // AUTO pill
                let auto_col = if self.auto_tempo { ACCENT_WARN } else { SURFACE_400 };
                let auto_bg = if self.auto_tempo {
                    ACCENT_WARN.linear_multiply(0.15)
                } else {
                    SURFACE_900.linear_multiply(0.6)
                };
                let auto_btn = egui::Button::new(
                    RichText::new("AUTO").size(8.0).color(auto_col).strong()
                ).fill(auto_bg).stroke(Stroke::new(1.0, SURFACE_800)).corner_radius(CornerRadius::same(4));
                if ui.add(auto_btn).clicked() {
                    self.auto_tempo = !self.auto_tempo;
                }

                // Time sig
                ui.horizontal(|ui| {
                    ui.label(RichText::new("4").size(13.0).color(SURFACE_100).monospace());
                    ui.label(RichText::new("/").size(13.0).color(SURFACE_500).monospace());
                    ui.label(RichText::new("4").size(13.0).color(SURFACE_100).monospace());
                });
            });

            // System monitor
            inset_frame().show(ui, |ui| {
                ui.vertical(|ui| {
                    // CPU
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("CPU").size(8.0).color(SURFACE_400).strong());
                        let (rect, _) = ui.allocate_exact_size(vec2(60.0, 4.0), Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, SURFACE_800);
                        let fill = Rect::from_min_max(
                            rect.min + vec2(1.0, 1.0),
                            pos2(rect.min.x + rect.width() * 0.15, rect.max.y - 1.0),
                        );
                        ui.painter().rect_filled(fill, 2.0, ACCENT_GLOW);
                        ui.label(RichText::new("15%").size(8.0).color(SURFACE_300).monospace());
                    });
                    // RAM
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("RAM").size(8.0).color(SURFACE_400).strong());
                        let (rect, _) = ui.allocate_exact_size(vec2(60.0, 4.0), Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, SURFACE_800);
                        let fill = Rect::from_min_max(
                            rect.min + vec2(1.0, 1.0),
                            pos2(rect.min.x + rect.width() * 0.42, rect.max.y - 1.0),
                        );
                        ui.painter().rect_filled(fill, 2.0, ACCENT_WARN);
                        ui.label(RichText::new("42%").size(8.0).color(SURFACE_300).monospace());
                    });
                });
            });
        });
    }
}

// ---------------------------------------------------------------------------
// Browser  (1:1 with HTML mockup)
// ---------------------------------------------------------------------------

impl App {
    fn browser_panel(&mut self, ui: &mut Ui) {
        ui.push_id("br", |ui| {
            ui.vertical(|ui| {
                // Title bar — matches HTML: BROWSER label + Search + Snap + Detach + Close
                title_bar_frame().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("BROWSER").size(9.0).color(SURFACE_400).strong());
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button(RichText::new("x").size(10.0).color(SURFACE_400)).clicked() {
                                self.show_browser = false;
                            }
                            let _ = ui.button(RichText::new("↗").size(10.0).color(SURFACE_400));
                            let _ = ui.button(RichText::new("+").size(10.0).color(SURFACE_400));
                            if ui.button(RichText::new("?").size(10.0).color(SURFACE_400)).clicked() {
                                self.browser_search_open = !self.browser_search_open;
                            }
                        });
                    });
                });

                // Search row (hidden by default, shown when search icon clicked)
                if self.browser_search_open {
                    Frame::new()
                        .fill(SURFACE_950)
                        .stroke(Stroke::new(1.0, SURFACE_800))
                        .inner_margin(Margin::symmetric(6, 2))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("?").size(10.0).color(SURFACE_400));
                                let response = ui.add(
                                    egui::TextEdit::singleline(&mut self.browser_search_query)
                                        .font(TextStyle::Monospace)
                                        .desired_width(ui.available_width() - 20.0)
                                        .hint_text("Search...")
                                );
                                if !self.browser_search_query.is_empty()
                                    && ui.button("x").clicked()
                                {
                                    self.browser_search_query.clear();
                                }
                                response.request_focus();
                            });
                        });
                }

                // Tabs — using TabBar widget
                let new_tab = TabBar::new("browser_tabs", &mut self.browser_tab)
                    .tab("Projects")
                    .tab("Samples")
                    .tab("Instr")
                    .tab("FX")
                    .tab("*")
                    .height(24.0)
                    .show(ui);
                self.browser_tab = new_tab;

                // Filter pills
                Frame::new()
                    .fill(Color32::from_rgba_premultiplied(2, 6, 23, 100))
                    .stroke(Stroke::new(1.0, SURFACE_800))
                    .inner_margin(Margin::same(4))
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            let filters = ["All", "Kicks", "Snares", "Hats", "Perc", "Bass", "Loops", "FX", "MIDI", "Vox", "Plug"];
                            for (fi, f) in filters.iter().enumerate() {
                                let is_active = fi == self.browser_filter;
                                let bg = if is_active { SURFACE_900 } else { SURFACE_950 };
                                let fg = if is_active { SURFACE_200 } else { SURFACE_300 };
                                if ui.add(
                                    egui::Button::new(RichText::new(*f).size(8.0).color(fg).strong())
                                        .fill(bg)
                                        .stroke(Stroke::new(1.0, SURFACE_800))
                                        .corner_radius(CornerRadius::same(16)),
                                ).clicked() {
                                    self.browser_filter = fi;
                                }
                            }
                        });
                    });

                // Tree content — using TreeView widget
                ScrollArea::vertical().show(ui, |ui| {
                    Frame::new().fill(SURFACE_950).inner_margin(Margin::same(4)).show(ui, |ui| {
                        match self.browser_tab {
                            0 => self.browser_projects(ui),
                            1 => self.browser_samples(ui),
                            2 => self.browser_instruments(ui),
                            3 => self.browser_effects(ui),
                            _ => self.browser_favorites(ui),
                        }
                    });
                });

                // Waveform preview at bottom — with metadata + play button
                Frame::new()
                    .fill(SURFACE_950)
                    .stroke(Stroke::new(1.0, SURFACE_800))
                    .inner_margin(Margin::same(4))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            // Header: play btn + metadata + close btn
                            ui.horizontal(|ui| {
                                let _ = ui.button(RichText::new("▶").size(10.0).color(SURFACE_300));
                                ui.label(RichText::new("0:00.000 | WAV | 24-bit | 48kHz | Stereo").size(8.0).color(SURFACE_400).monospace());
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let _ = ui.button(RichText::new("x").size(8.0).color(SURFACE_500));
                                });
                            });
                            // Waveform
                            let sample_data: Vec<f32> = (0..200).map(|i| {
                                (i as f32 * 0.1).sin() * 0.5 * (1.0 + (i as f32 * 0.02).sin())
                            }).collect();
                            Waveform::new(&sample_data)
                                .color(ACCENT_GLOW)
                                .filled(true)
                                .background(SURFACE_900)
                                .show(ui, vec2(ui.available_width(), 48.0));
                        });
                    });
            });
        });
    }

    fn browser_projects(&self, ui: &mut Ui) {
        let tree = TreeNode::new("Current Project").icon('>').child(
            TreeNode::new("Project.neuprj").icon('-')
        ).child(
            TreeNode::new("Bounce_2026-02-23.wav").icon('~')
        ).child(
            TreeNode::new("Vocals_Take_03.wav").icon('~')
        ).child(
            TreeNode::new("ChordProgression.mid").icon('*')
        ).child(
            TreeNode::new("Automation_FilterCut.autoclip").icon('#')
        ).child(
            TreeNode::new("MixBusPreset.nfx").icon('@')
        );
        TreeView::new("proj_tree").node(tree).row_height(20.0).show(ui);

        let recent = TreeNode::new("Recent Projects").icon('>').child(
            TreeNode::new("Synthwave_03").icon('-')
        ).child(
            TreeNode::new("Client_Edit_A").icon('-')
        );
        TreeView::new("recent_tree").node(recent).row_height(20.0).show(ui);

        let plugs = TreeNode::new("Plugin Database").icon('>').child(
            TreeNode::new("Installed").icon('>')
        );
        TreeView::new("plugs_tree").node(plugs).row_height(20.0).show(ui);
    }

    fn browser_samples(&self, ui: &mut Ui) {
        let tree = TreeNode::new("Packs").icon('>').child(
            TreeNode::new("Drums").icon('>').child(
                TreeNode::new("Kick_01.wav").icon('~')
            ).child(
                TreeNode::new("Kick_02_Heavy.wav").icon('~')
            ).child(
                TreeNode::new("Snare_A.wav").icon('~')
            ).child(
                TreeNode::new("Clap_Room.wav").icon('~')
            ).child(
                TreeNode::new("HiHats").icon('>').child(
                    TreeNode::new("Hat_Closed_01.wav").icon('~')
                ).child(
                    TreeNode::new("Hat_Closed_02_Tight.wav").icon('~')
                ).child(
                    TreeNode::new("Hat_Open_01.wav").icon('~')
                ).child(
                    TreeNode::new("Hat_Pedal.wav").icon('~')
                )
            ).child(
                TreeNode::new("Perc").icon('>')
            ).child(
                TreeNode::new("Cymbals").icon('>')
            )
        ).child(
            TreeNode::new("Synths").icon('>')
        ).child(
            TreeNode::new("Loops").icon('>')
        ).child(
            TreeNode::new("FX").icon('>')
        ).child(
            TreeNode::new("MIDI").icon('>')
        );
        TreeView::new("samples_tree").node(tree).row_height(20.0).show(ui);
    }

    fn browser_instruments(&self, ui: &mut Ui) {
        let mut synth_children = Vec::new();
        for s in ["Wavetable Synth", "FM Keys", "Granular Pad", "Multi-Engine Synth", "SoundFont Player"] {
            synth_children.push(TreeNode::new(s).icon('+'));
        }
        let mut node = TreeNode::new("Synths").icon('>');
        for child in synth_children {
            node = node.child(child);
        }
        let tree = node
            .child(TreeNode::new("Samplers").icon('>'))
            .child(TreeNode::new("Drums").icon('>'))
            .child(TreeNode::new("Utilities").icon('>'));
        TreeView::new("instr_tree").node(tree).row_height(20.0).show(ui);
    }

    fn browser_effects(&self, ui: &mut Ui) {
        let tree = TreeNode::new("Dynamics").icon('>')
            .child(TreeNode::new("VCA Comp").icon('@'))
            .child(TreeNode::new("Limiter").icon('@'))
            .child(TreeNode::new("Gate").icon('@'))
            .child(TreeNode::new("EQ & Filters").icon('>')
                .child(TreeNode::new("Parametric EQ").icon('@'))
                .child(TreeNode::new("Low Pass").icon('@'))
                .child(TreeNode::new("High Pass").icon('@'))
            )
            .child(TreeNode::new("Space").icon('>')
                .child(TreeNode::new("Plate Reverb").icon('@'))
                .child(TreeNode::new("Delay").icon('@'))
            )
            .child(TreeNode::new("Modulation").icon('>')
                .child(TreeNode::new("Chorus").icon('@'))
                .child(TreeNode::new("Phaser").icon('@'))
            )
            .child(TreeNode::new("Utility").icon('>')
                .child(TreeNode::new("Utility").icon('@'))
                .child(TreeNode::new("Stereo Tool").icon('@'))
            );
        TreeView::new("fx_tree").node(tree).row_height(20.0).show(ui);
    }

    fn browser_favorites(&self, ui: &mut Ui) {
        ui.label(RichText::new("No favorites yet").size(10.0).color(SURFACE_500));
        ui.label(RichText::new("Right-click items to add to favorites").size(9.0).color(SURFACE_600));
    }
}

// ---------------------------------------------------------------------------
// Playlist  (1:1 with HTML mockup)
// ---------------------------------------------------------------------------

impl App {
    fn playlist_panel(&mut self, ui: &mut Ui) {
        ui.push_id("pl", |ui| {
            ui.vertical(|ui| {
                // Title bar — Playlist + Snap + Detach + Close (matches HTML)
                title_bar_frame().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("PLAYLIST").size(9.0).color(SURFACE_400).strong());
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button(RichText::new("x").size(10.0).color(SURFACE_400)).clicked() {
                                self.show_playlist = false;
                            }
                            let _ = ui.button(RichText::new("↗").size(10.0).color(SURFACE_400));
                            let _ = ui.button(RichText::new("+").size(10.0).color(SURFACE_400));
                        });
                    });
                });

                // Ruler using crate widget
                Frame::new().fill(SURFACE_800).stroke(Stroke::new(1.0, SURFACE_700)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Left stub with CLIP label
                        let (rect, _) = ui.allocate_exact_size(vec2(192.0, 24.0), Sense::hover());
                        ui.painter().rect_filled(rect, 0.0, SURFACE_900);
                        ui.painter().text(rect.center(), Align2::CENTER_CENTER, "CLIP",
                            FontId::proportional(11.0), SURFACE_500);

                        // Ruler widget
                        Ruler::new(&self.playlist_pan_zoom, 4)
                            .playhead(self.playhead_beat)
                            .height(24.0)
                            .bar_color(SURFACE_500)
                            .beat_color(Color32::from_rgba_premultiplied(148, 163, 184, 60))
                            .playhead_color(ACCENT_GLOW)
                            .show(ui);
                    });
                });

                // Main content: lane headers + clip grid
                let available = ui.available_size();
                ui.horizontal(|ui| {
                    // Lane headers (sticky left 192px)
                    let (lane_rect, _) = ui.allocate_exact_size(vec2(192.0, available.y), Sense::hover());
                    let p = ui.painter();
                    p.rect_filled(lane_rect, 0.0, SURFACE_900);

                    // "Lanes" header row with eye toggle
                    let header_h = 40.0;
                    let header_rect = Rect::from_min_size(lane_rect.min, vec2(lane_rect.width(), header_h));
                    p.rect_filled(header_rect, 0.0, SURFACE_900);
                    p.line_segment([header_rect.left_bottom(), header_rect.right_bottom()], Stroke::new(1.0, SURFACE_800));
                    p.text(pos2(header_rect.left() + 12.0, header_rect.center().y),
                        Align2::LEFT_CENTER, "Lanes", FontId::proportional(9.0), SURFACE_400);
                    // Eye icon
                    p.text(pos2(header_rect.right() - 24.0, header_rect.center().y),
                        Align2::CENTER_CENTER, "o", FontId::proportional(11.0), SURFACE_400);

                    // Right border
                    p.line_segment([
                        pos2(lane_rect.right(), lane_rect.top()),
                        pos2(lane_rect.right(), lane_rect.bottom())
                    ], Stroke::new(1.0, SURFACE_800));

                    // 10 lane rows — mute/solo dot + track number + track name + effects button
                    for i in 0..10 {
                        let y = lane_rect.top() + header_h + (i as f32) * 48.0;
                        let row_rect = Rect::from_min_size(pos2(lane_rect.left(), y), vec2(192.0, 48.0));
                        p.rect_filled(row_rect, 0.0, if i % 2 == 0 { SURFACE_900 } else { Color32::from_rgba_premultiplied(15, 23, 42, 200) });
                        p.line_segment([row_rect.left_bottom(), row_rect.right_bottom()], Stroke::new(1.0, SURFACE_800));

                        // Track number
                        p.text(pos2(row_rect.left() + 28.0, row_rect.center().y - 2.0),
                            Align2::LEFT_CENTER, format!("{}", i + 1),
                            FontId::proportional(12.0), SURFACE_200);
                        // Track name
                        p.text(pos2(row_rect.left() + 44.0, row_rect.center().y + 2.0),
                            Align2::LEFT_CENTER, format!("Track {}", i + 1),
                            FontId::proportional(9.0), SURFACE_500);
                        // Effects button (≡)
                        p.text(pos2(row_rect.right() - 20.0, row_rect.center().y),
                            Align2::CENTER_CENTER, "≡", FontId::proportional(11.0), SURFACE_400);
                    }

                    // Draw mute/solo dots using ToggleDot
                    let _ = p;
                    for (i, ms) in self.lane_mute_solo.iter_mut().enumerate() {
                        let dot_y = lane_rect.top() + header_h + (i as f32) * 48.0 + 24.0 - 5.0;
                        let dot_x = lane_rect.left() + 12.0;
                        ui.scope_builder(UiBuilder::new().max_rect(Rect::from_min_size(
                            Pos2::new(dot_x, dot_y), vec2(10.0, 10.0)
                        )), |ui| {
                            ui.add(ToggleDot::new(ms).size(10.0));
                        });
                    }

                    // Clip grid area — tool bar + clips
                    let grid_w = ui.available_width();
                    ui.vertical(|ui| {
                        // Tool bar (matches HTML: Select/Draw/Paint/Slice + Automation + Snap + Minimap + Clip selector)
                        Frame::new()
                            .fill(SURFACE_900.linear_multiply(0.6))
                            .stroke(Stroke::new(1.0, SURFACE_800))
                            .inner_margin(Margin::symmetric(6, 2))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // Tools: Select/Draw/Paint/Slice
                                    let tools = [("<", 0), ("e", 1), ("+", 2), ("c", 3)];
                                    for (icon, idx) in tools {
                                        let bg = if self.playlist_tool == idx { SURFACE_700 } else { SURFACE_800 };
                                        let fg = if self.playlist_tool == idx { Color32::WHITE } else { SURFACE_300 };
                                        if ui.add(
                                            egui::Button::new(RichText::new(icon).size(10.0).color(fg))
                                                .fill(bg).min_size(vec2(24.0, 24.0))
                                        ).clicked() {
                                            self.playlist_tool = idx;
                                        }
                                    }
                                    // Separator
                                    let (sep, _) = ui.allocate_exact_size(vec2(1.0, 18.0), Sense::hover());
                                    ui.painter().rect_filled(sep, 0.0, SURFACE_800);

                                    // Automation mode + draw buttons
                                    ui.add(egui::Button::new(RichText::new("~").size(10.0).color(SURFACE_300)).fill(SURFACE_800).min_size(vec2(24.0, 24.0)));
                                    ui.add(egui::Button::new(RichText::new("e~").size(10.0).color(SURFACE_300)).fill(SURFACE_800).min_size(vec2(24.0, 24.0)));

                                    // Separator
                                    let (sep, _) = ui.allocate_exact_size(vec2(1.0, 18.0), Sense::hover());
                                    ui.painter().rect_filled(sep, 0.0, SURFACE_800);

                                    // Snap dropdown
                                    Frame::new()
                                        .fill(Color32::from_rgba_premultiplied(2, 6, 23, 60))
                                        .stroke(Stroke::new(1.0, SURFACE_800))
                                        .inner_margin(Margin::symmetric(4, 2))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new("⊕").size(9.0).color(SURFACE_400));
                                                ui.label(RichText::new("Snap").size(8.0).color(SURFACE_500).strong());
                                                let snap_options = ["Off", "1/64", "1/32", "1/16", "1/12", "1/8", "1/6", "1/4", "1/3", "1/2", "Beat", "Bar", "2Bar", "4Bar"];
                                                egui::ComboBox::from_id_salt("snap_select")
                                                    .selected_text(snap_options[self.snap_mode])
                                                    .show_ui(ui, |ui| {
                                                        for (si, opt) in snap_options.iter().enumerate() {
                                                            ui.selectable_value(&mut self.snap_mode, si, *opt);
                                                        }
                                                    });
                                            });
                                        });

                                    // Minimap placeholder
                                    let (mm_rect, _) = ui.allocate_exact_size(vec2(60.0, 20.0), Sense::hover());
                                    ui.painter().rect_filled(mm_rect, 2.0, SURFACE_950);
                                    ui.painter().rect_stroke(mm_rect, 2.0, Stroke::new(1.0, SURFACE_800), StrokeKind::Outside);

                                    // Right side: Clip selector
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        // Clip number selector
                                        let (cs_rect, _) = ui.allocate_exact_size(vec2(28.0, 20.0), Sense::click());
                                        ui.painter().rect_filled(cs_rect, 3.0, Color32::from_rgba_premultiplied(2, 6, 23, 60));
                                        ui.painter().rect_stroke(cs_rect, 3.0, Stroke::new(1.0, SURFACE_800), StrokeKind::Outside);
                                        ui.painter().text(cs_rect.center(), Align2::CENTER_CENTER, "01",
                                            FontId::monospace(9.0), SURFACE_200);
                                        // Clip label
                                        ui.label(RichText::new("Clip").size(8.0).color(SURFACE_500).strong());
                                    });
                                });
                            });

                        // Clip grid + clips + playhead
                        let (grid_rect, _) = ui.allocate_exact_size(vec2(grid_w, ui.available_height()), Sense::click_and_drag());
                        let p = ui.painter();
                        p.rect_filled(grid_rect, 0.0, SURFACE_950);

                        // Grid lines
                        let mut x = grid_rect.left();
                        let mut beat = 0;
                        while x < grid_rect.right() {
                            let is_bar = beat % 4 == 0;
                            let col = if is_bar { Color32::from_rgba_premultiplied(239, 68, 68, 30) } else { Color32::from_rgba_premultiplied(255, 255, 255, 10) };
                            p.line_segment([pos2(x, grid_rect.top()), pos2(x, grid_rect.bottom())], Stroke::new(if is_bar { 2.0 } else { 1.0 }, col));
                            x += 80.0; beat += 1;
                        }
                        for i in 0..=10 {
                            let y = grid_rect.top() + 40.0 + (i as f32) * 48.0;
                            p.line_segment([pos2(grid_rect.left(), y), pos2(grid_rect.right(), y)], Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 15)));
                        }

                        // Loop region (semi-transparent overlay)
                        if self.loop_on {
                            let loop_start_x = grid_rect.left() + 0.0 * 80.0;
                            let loop_end_x = grid_rect.left() + 16.0 * 80.0;
                            let region = Rect::from_min_max(
                                pos2(loop_start_x, grid_rect.top() + 40.0),
                                pos2(loop_end_x, grid_rect.bottom())
                            );
                            p.rect_filled(region.intersect(grid_rect), 0.0, ACCENT_GLOW.linear_multiply(0.05));
                        }

                        let _ = p;
                        let pixels_per_beat = 80.0;
                        let origin = grid_rect.min;

                        // Draw clips using TimelineClip widgets
                        for clip in &mut self.clips {
                            let clip_y = origin.y + 40.0 + (clip.track as f32) * 48.0 + 4.0;
                            let color = match clip.kind {
                                ClipKind::Pattern => ACCENT_MIDI.linear_multiply(0.35),
                                ClipKind::Audio => ACCENT_AUDIO.linear_multiply(0.30),
                                ClipKind::Automation => ACCENT_GLOW.linear_multiply(0.25),
                            };
                            ui.scope_builder(UiBuilder::new().max_rect(Rect::from_min_size(
                                Pos2::new(origin.x + clip.start_beat * pixels_per_beat, clip_y),
                                Vec2::new(clip.len_beats * pixels_per_beat, 40.0),
                            )), |ui| {
                                TimelineClip::new(&mut clip.start_beat, &mut clip.len_beats)
                                    .kind(clip.kind)
                                    .label(&clip.title)
                                    .color(color)
                                    .height(40.0)
                                    .pixels_per_unit(pixels_per_beat)
                                    .show(ui);
                            });
                        }

                        // Playhead
                        let ph_x = grid_rect.left() + self.playhead_beat * pixels_per_beat;
                        let p = ui.painter();
                        p.line_segment([pos2(ph_x, grid_rect.top()), pos2(ph_x, grid_rect.bottom())], Stroke::new(2.0, ACCENT_GLOW));
                        let tri = [pos2(ph_x - 5.0, grid_rect.top()), pos2(ph_x + 5.0, grid_rect.top()), pos2(ph_x, grid_rect.top() + 7.0)];
                        p.add(Shape::convex_polygon(tri.to_vec(), ACCENT_GLOW, Stroke::NONE));
                    });
                });
            });
        });
    }
}

// ---------------------------------------------------------------------------
// Channel Rack  (1:1 with HTML mockup)
// ---------------------------------------------------------------------------

impl App {
    fn channel_rack_panel(&mut self, ui: &mut Ui) {
        ui.push_id("cr", |ui| {
            ui.vertical(|ui| {
                // Title bar — Channel Rack + Clip 01 + Add + Snap + Detach + Hide
                title_bar_frame().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("CHANNEL RACK").size(9.0).color(SURFACE_400).strong());
                        // Clip 01 selector
                        let (cs_rect, _) = ui.allocate_exact_size(vec2(44.0, 18.0), Sense::click());
                        ui.painter().rect_filled(cs_rect, 3.0, SURFACE_950);
                        ui.painter().rect_stroke(cs_rect, 3.0, Stroke::new(1.0, SURFACE_700), StrokeKind::Outside);
                        ui.painter().text(cs_rect.center(), Align2::CENTER_CENTER, "Clip 01",
                            FontId::monospace(8.0), SURFACE_200);

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button(RichText::new("▼").size(10.0).color(SURFACE_400)).clicked() {
                                self.show_channel_rack = false;
                            }
                            let _ = ui.button(RichText::new("↗").size(10.0).color(SURFACE_400));
                            let _ = ui.button(RichText::new("+").size(10.0).color(SURFACE_400));
                            if ui.button(RichText::new("+").size(12.0).color(SURFACE_400)).clicked() {
                                // Add generator
                                self.generators.push(gen("New", MIXER_COLORS[self.generators.len() % MIXER_COLORS.len()],
                                    &[false; 16]));
                            }
                        });
                    });
                });

                // Tools row — Actions toggle + Rand/Inv/Rev/Alt + 2/4/8 + ←/→/x | Swing + More
                Frame::new()
                    .fill(Color32::from_rgba_premultiplied(2, 6, 23, 100))
                    .stroke(Stroke::new(1.0, SURFACE_800))
                    .inner_margin(Margin::symmetric(6, 2))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Actions toggle
                            let actions_bg = if self.actions_visible { SURFACE_700 } else { SURFACE_800 };
                            if ui.add(egui::Button::new(RichText::new("▼").size(9.0).color(SURFACE_200)).fill(actions_bg)).clicked() {
                                self.actions_visible = !self.actions_visible;
                            }

                            // Separator
                            let (sep, _) = ui.allocate_exact_size(vec2(1.0, 18.0), Sense::hover());
                            ui.painter().rect_filled(sep, 0.0, SURFACE_800);

                            if self.actions_visible {
                                for t in ["Rand", "Inv", "Rev", "Alt"] {
                                    if ui.add(egui::Button::new(RichText::new(t).size(8.0).color(SURFACE_200).strong()).fill(SURFACE_800).stroke(Stroke::new(1.0, SURFACE_700))).clicked() {
                                        // Apply action to first generator
                                        if let Some(gen) = self.generators.first_mut() {
                                            match t {
                                                "Rand" => {
                                                    for step in gen.steps[0].iter_mut() { *step = rand_bool(); }
                                                }
                                                "Inv" => {
                                                    for step in gen.steps[0].iter_mut() { *step = !*step; }
                                                }
                                                "Rev" => {
                                                    gen.steps[0].reverse();
                                                }
                                                "Alt" => {
                                                    for (i, step) in gen.steps[0].iter_mut().enumerate() { *step = i % 2 == 0; }
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }

                                let (sep, _) = ui.allocate_exact_size(vec2(1.0, 18.0), Sense::hover());
                                ui.painter().rect_filled(sep, 0.0, SURFACE_800);

                                for t in ["2", "4", "8"] {
                                    if ui.add(egui::Button::new(RichText::new(t).size(8.0).color(SURFACE_200).strong()).fill(SURFACE_800).stroke(Stroke::new(1.0, SURFACE_700))).clicked() {
                                        let every = t.parse::<usize>().unwrap_or(2);
                                        if let Some(gen) = self.generators.first_mut() {
                                            for (i, step) in gen.steps[0].iter_mut().enumerate() { *step = i % every == 0; }
                                        }
                                    }
                                }

                                let (sep, _) = ui.allocate_exact_size(vec2(1.0, 18.0), Sense::hover());
                                ui.painter().rect_filled(sep, 0.0, SURFACE_800);

                                // Shift left, shift right, clear
                                if ui.add(egui::Button::new(RichText::new("←").size(9.0).color(SURFACE_200)).fill(SURFACE_800)).clicked() {
                                    if let Some(gen) = self.generators.first_mut() {
                                        gen.steps[0].rotate_left(1);
                                    }
                                }
                                if ui.add(egui::Button::new(RichText::new("→").size(9.0).color(SURFACE_200)).fill(SURFACE_800)).clicked() {
                                    if let Some(gen) = self.generators.first_mut() {
                                        gen.steps[0].rotate_right(1);
                                    }
                                }
                                if ui.add(egui::Button::new(RichText::new("x").size(9.0).color(SURFACE_200)).fill(SURFACE_800)).clicked() {
                                    if let Some(gen) = self.generators.first_mut() {
                                        for step in gen.steps[0].iter_mut() { *step = false; }
                                    }
                                }
                            }

                            // Right side: Swing + More
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                // More menu
                                ui.add(egui::Button::new(RichText::new("⋮").size(12.0).color(SURFACE_400)).fill(Color32::TRANSPARENT));

                                // Swing
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("{}%", self.swing as i32)).size(8.0).color(SURFACE_400).monospace());
                                    let (slider_rect, slider_rsp) = ui.allocate_exact_size(vec2(80.0, 10.0), Sense::click_and_drag());
                                    ui.painter().rect_filled(slider_rect, 3.0, SURFACE_950);
                                    let fill_w = slider_rect.width() * (self.swing / 100.0);
                                    let fill_rect = Rect::from_min_max(slider_rect.min, pos2(slider_rect.min.x + fill_w, slider_rect.max.y));
                                    ui.painter().rect_filled(fill_rect, 3.0, ACCENT_GLOW);
                                    if slider_rsp.dragged() || slider_rsp.clicked() {
                                        if let Some(pos) = ui.input(|i| i.pointer.latest_pos()) {
                                            self.swing = ((pos.x - slider_rect.min.x) / slider_rect.width() * 100.0).clamp(0.0, 100.0);
                                        }
                                    }
                                    ui.label(RichText::new("Swing").size(8.0).color(SURFACE_500).strong());
                                });
                            });
                        });
                    });

                // Step grid area
                ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("16 steps").size(8.0).color(SURFACE_500).monospace());
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(RichText::new("VOL / PAN").size(8.0).color(SURFACE_500).monospace());
                        });
                    });
                    ui.add_space(2.0);

                    for (gi, gen) in self.generators.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            // Mute toggle using ToggleDot
                            ui.add(ToggleDot::new(&mut gen.mute_state).size(10.0));

                            // Generator name button
                            let name_bg = if gi == 0 { SURFACE_800 } else { Color32::TRANSPARENT };
                            let name_col = if gi == 0 { SURFACE_50 } else { SURFACE_400 };
                            let btn = egui::Button::new(RichText::new(&gen.name).size(10.0).color(name_col))
                                .fill(name_bg)
                                .min_size(vec2(80.0, 20.0));
                            ui.add(btn);

                            // StepGrid widget
                            let row_colors = vec![gen.color];
                            ui.add(
                                StepGrid::new(&mut gen.steps, 1, 16)
                                    .cell_size(vec2(20.0, 20.0))
                                    .active_col(0)
                                    .row_colors(row_colors),
                            );
                        });
                        ui.add_space(2.0);
                    }
                });
            });
        });
    }
}

/// Simple pseudo-random bool for step randomization
fn rand_bool() -> bool {
    use std::time::SystemTime;
    let nanos = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().subsec_nanos();
    (nanos as usize).wrapping_mul(2654435761).is_multiple_of(2)
}

// ---------------------------------------------------------------------------
// Mixer  (1:1 with HTML mockup)
// ---------------------------------------------------------------------------

impl App {
    fn mixer_panel(&mut self, ui: &mut Ui) {
        ui.push_id("mx", |ui| {
            ui.vertical(|ui| {
                // Title bar — Menu + FX + Add + Compact/layout | Route + Snap + Detach
                title_bar_frame().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Menu button
                        let _ = ui.button(RichText::new("▼").size(10.0).color(SURFACE_400));
                        // FX button
                        if ui.button(RichText::new("≡").size(10.0).color(SURFACE_400)).clicked() {
                            self.show_devices = !self.show_devices;
                        }
                        // Add channel
                        if ui.button(RichText::new("+").size(12.0).color(SURFACE_400)).clicked() {
                            let idx = self.mixer.len();
                            let name = format!("Ch {}", idx);
                            self.mixer.push(mc(&name, MIXER_COLORS[idx % MIXER_COLORS.len()]));
                        }
                        // Compact style label + layout button
                        Frame::new()
                            .fill(Color32::from_rgba_premultiplied(2, 6, 23, 40))
                            .stroke(Stroke::new(1.0, SURFACE_700))
                            .inner_margin(Margin::symmetric(4, 1))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Compact").size(8.0).color(SURFACE_500).strong());
                                    let _ = ui.button(RichText::new("+").size(8.0).color(SURFACE_400));
                                });
                            });

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let _ = ui.button(RichText::new("↗").size(10.0).color(SURFACE_400));
                            let _ = ui.button(RichText::new("+").size(10.0).color(SURFACE_400));
                            // Route button
                            let _ = ui.button(RichText::new("⊕").size(10.0).color(SURFACE_400));
                        });
                    });
                });

                // Channel strips using ChannelStrip widget
                ScrollArea::horizontal().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let any_solo = self.mixer.iter().any(|c| matches!(c.solo_state, DotState::Solo | DotState::On));

                        for (ci, ch) in self.mixer.iter_mut().enumerate() {
                            let meter_l = if matches!(ch.mute_state, DotState::Muted | DotState::SoloMuted) {
                                0.0
                            } else if any_solo && !matches!(ch.solo_state, DotState::Solo | DotState::On) {
                                ch.meter_l * 0.34
                            } else {
                                ch.meter_l
                            };

                            ChannelStrip::new(
                                ("ch", ci),
                                &mut ch.volume,
                                &mut ch.pan,
                                &mut ch.mute_state,
                                &mut ch.solo_state,
                            )
                            .name(&ch.name)
                            .color(ch.color)
                            .stereo_meter(meter_l, ch.meter_r)
                            .index(ci as u32 + 1)
                            .width(64.0)
                            .show(ui);
                        }

                        // Add channel button (dashed border + "NEW")
                        let (rect, rsp) = ui.allocate_exact_size(vec2(64.0, ui.available_height()), Sense::click());
                        let p = ui.painter();
                        p.rect_stroke(rect, 6.0, Stroke::new(2.0, SURFACE_700), StrokeKind::Outside);
                        p.text(rect.center(), Align2::CENTER_CENTER, "+\nNEW", FontId::proportional(11.0), SURFACE_500);
                        if rsp.clicked() {
                            let idx = self.mixer.len();
                            let name = format!("Ch {}", idx);
                            self.mixer.push(mc(&name, MIXER_COLORS[idx % MIXER_COLORS.len()]));
                        }
                    });
                });
            });
        });
    }
}

// ---------------------------------------------------------------------------
// Devices  (1:1 with HTML mockup)
// ---------------------------------------------------------------------------

impl App {
    fn devices_panel(&mut self, ui: &mut Ui) {
        ui.push_id("dev", |ui| {
            ui.vertical(|ui| {
                // Title bar — DEVICES + channel label + IN/OUT selects | Snap + Detach
                title_bar_frame().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("DEVICES").size(9.0).color(SURFACE_400).strong());
                        // Channel label
                        let ch_name = self.mixer.get(self.selected_mixer_channel).map(|c| c.name.as_str()).unwrap_or("Master");
                        let (lbl_rect, _) = ui.allocate_exact_size(vec2(50.0, 18.0), Sense::hover());
                        ui.painter().rect_filled(lbl_rect, 3.0, SURFACE_950);
                        ui.painter().rect_stroke(lbl_rect, 3.0, Stroke::new(1.0, SURFACE_700), StrokeKind::Outside);
                        ui.painter().text(lbl_rect.center(), Align2::CENTER_CENTER, ch_name,
                            FontId::monospace(8.0), SURFACE_200);

                        // Separator
                        let (sep, _) = ui.allocate_exact_size(vec2(1.0, 16.0), Sense::hover());
                        ui.painter().rect_filled(sep, 0.0, SURFACE_700);

                        // IN select
                        ui.label(RichText::new("IN").size(8.0).color(SURFACE_500).strong());
                        egui::ComboBox::from_id_salt("devices_in")
                            .selected_text("— select —")
                            .show_ui(ui, |ui| {
                                ui.label(RichText::new("(no inputs)").size(9.0).color(SURFACE_500));
                            });

                        // OUT select
                        ui.label(RichText::new("OUT").size(8.0).color(SURFACE_500).strong());
                        egui::ComboBox::from_id_salt("devices_out")
                            .selected_text("— select —")
                            .show_ui(ui, |ui| {
                                ui.label(RichText::new("(no outputs)").size(9.0).color(SURFACE_500));
                            });

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let _ = ui.button(RichText::new("↗").size(10.0).color(SURFACE_400));
                            let _ = ui.button(RichText::new("+").size(10.0).color(SURFACE_400));
                        });
                    });
                });

                // Rack area with dot grid + placeholder device modules
                let (rect, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
                let p = ui.painter();
                p.rect_filled(rect, 0.0, SURFACE_800);

                // Dot grid pattern (10px spacing, 1px radius, 3% opacity)
                let spacing = 10.0;
                let mut y = rect.top();
                while y < rect.bottom() {
                    let mut x = rect.left();
                    while x < rect.right() {
                        p.circle_filled(pos2(x, y), 1.0, Color32::from_rgba_premultiplied(51, 65, 85, 12));
                        x += spacing;
                    }
                    y += spacing;
                }

                // Placeholder device modules (matching mockup's rack content area)
                let module_w = 200.0;
                let module_h = rect.height() - 12.0;
                let start_x = rect.left() + 8.0;
                let start_y = rect.top() + 6.0;

                // Module 1: EQ
                let eq_rect = Rect::from_min_size(pos2(start_x, start_y), vec2(module_w, module_h));
                p.rect_filled(eq_rect, 6.0, SURFACE_900);
                p.rect_stroke(eq_rect, 6.0, Stroke::new(1.0, SURFACE_700), StrokeKind::Outside);
                // Module header
                let hdr_rect = Rect::from_min_size(eq_rect.min, vec2(module_w, 20.0));
                p.rect_filled(hdr_rect, 6.0, ACCENT_AUDIO.linear_multiply(0.15));
                p.text(hdr_rect.center(), Align2::CENTER_CENTER, "Parametric EQ",
                    FontId::proportional(9.0), SURFACE_200);
                // Placeholder knobs
                for i in 0..4 {
                    let kx = eq_rect.left() + 30.0 + (i as f32) * 45.0;
                    let ky = eq_rect.center().y + 5.0;
                    p.circle(pos2(kx, ky), 12.0, Color32::TRANSPARENT, Stroke::new(1.0, SURFACE_600));
                    p.line_segment([pos2(kx, ky), pos2(kx, ky - 8.0)], Stroke::new(1.5, SURFACE_300));
                    p.text(pos2(kx, ky + 18.0), Align2::CENTER_CENTER,
                        ["Gain", "Freq", "Q", "Mix"][i], FontId::proportional(7.0), SURFACE_500);
                }

                // Module 2: Compressor
                let comp_x = start_x + module_w + 8.0;
                let comp_rect = Rect::from_min_size(pos2(comp_x, start_y), vec2(module_w, module_h));
                p.rect_filled(comp_rect, 6.0, SURFACE_900);
                p.rect_stroke(comp_rect, 6.0, Stroke::new(1.0, SURFACE_700), StrokeKind::Outside);
                let hdr2 = Rect::from_min_size(comp_rect.min, vec2(module_w, 20.0));
                p.rect_filled(hdr2, 6.0, ACCENT_GLOW.linear_multiply(0.15));
                p.text(hdr2.center(), Align2::CENTER_CENTER, "VCA Compressor",
                    FontId::proportional(9.0), SURFACE_200);
                for i in 0..4 {
                    let kx = comp_rect.left() + 30.0 + (i as f32) * 45.0;
                    let ky = comp_rect.center().y + 5.0;
                    p.circle(pos2(kx, ky), 12.0, Color32::TRANSPARENT, Stroke::new(1.0, SURFACE_600));
                    p.line_segment([pos2(kx, ky), pos2(kx, ky - 8.0)], Stroke::new(1.5, SURFACE_300));
                    p.text(pos2(kx, ky + 18.0), Align2::CENTER_CENTER,
                        ["Thresh", "Ratio", "Attack", "Release"][i], FontId::proportional(7.0), SURFACE_500);
                }

                // Module 3: Reverb
                let rev_x = comp_x + module_w + 8.0;
                let rev_rect = Rect::from_min_size(pos2(rev_x, start_y), vec2(module_w, module_h));
                p.rect_filled(rev_rect, 6.0, SURFACE_900);
                p.rect_stroke(rev_rect, 6.0, Stroke::new(1.0, SURFACE_700), StrokeKind::Outside);
                let hdr3 = Rect::from_min_size(rev_rect.min, vec2(module_w, 20.0));
                p.rect_filled(hdr3, 6.0, ACCENT_WARN.linear_multiply(0.15));
                p.text(hdr3.center(), Align2::CENTER_CENTER, "Plate Reverb",
                    FontId::proportional(9.0), SURFACE_200);
                for i in 0..4 {
                    let kx = rev_rect.left() + 30.0 + (i as f32) * 45.0;
                    let ky = rev_rect.center().y + 5.0;
                    p.circle(pos2(kx, ky), 12.0, Color32::TRANSPARENT, Stroke::new(1.0, SURFACE_600));
                    p.line_segment([pos2(kx, ky), pos2(kx, ky - 8.0)], Stroke::new(1.5, SURFACE_300));
                    p.text(pos2(kx, ky + 18.0), Align2::CENTER_CENTER,
                        ["Size", "Decay", "Damping", "Mix"][i], FontId::proportional(7.0), SURFACE_500);
                }
            });
        });
    }
}
