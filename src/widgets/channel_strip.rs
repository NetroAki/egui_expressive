use crate::widgets::{controls::ToggleDot, faders::Fader, knobs::Knob, meters::Meter};
use egui::{Color32, Response, Ui, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelStripStyle {
    Compact,
    FlStudio,
    Ableton,
}

pub struct SendControl<'a> {
    pub label: &'a str,
    pub value: &'a mut f64,
}

pub struct ChannelStrip<'a> {
    pub gain: &'a mut f64,
    pub pan: &'a mut f64,
    pub mute: &'a mut crate::widgets::controls::DotState,
    pub solo: Option<&'a mut crate::widgets::controls::DotState>,
    pub record: Option<&'a mut crate::widgets::controls::DotState>,
    pub sends: Vec<SendControl<'a>>,
    pub level: f32,
    pub size: Vec2,
    pub name: String,
    pub label_color: Color32,
    pub style: ChannelStripStyle,
}

impl<'a> ChannelStrip<'a> {
    pub fn new(
        gain: &'a mut f64,
        pan: &'a mut f64,
        mute: &'a mut crate::widgets::controls::DotState,
        level: f32,
    ) -> Self {
        Self {
            gain,
            pan,
            mute,
            solo: None,
            record: None,
            sends: Vec::new(),
            level,
            size: Vec2::new(88.0, 260.0),
            name: "Channel".to_owned(),
            label_color: Color32::from_rgb(120, 170, 220),
            style: ChannelStripStyle::Compact,
        }
    }

    pub fn solo(mut self, state: &'a mut crate::widgets::controls::DotState) -> Self {
        self.solo = Some(state);
        self
    }
    pub fn record(mut self, state: &'a mut crate::widgets::controls::DotState) -> Self {
        self.record = Some(state);
        self
    }
    pub fn send(mut self, label: &'a str, value: &'a mut f64) -> Self {
        self.sends.push(SendControl { label, value });
        self
    }
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
    pub fn label_color(mut self, color: Color32) -> Self {
        self.label_color = color;
        self
    }
    pub fn style(mut self, style: ChannelStripStyle) -> Self {
        self.style = style;
        self
    }
}

impl<'a> egui::Widget for ChannelStrip<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let frame = egui::Frame::group(ui.style())
            .fill(match self.style {
                ChannelStripStyle::Compact => Color32::from_rgb(28, 30, 36),
                ChannelStripStyle::FlStudio => Color32::from_rgb(38, 42, 44),
                ChannelStripStyle::Ableton => Color32::from_rgb(46, 46, 50),
            })
            .inner_margin(egui::Margin::symmetric(6, 6));

        frame
            .show(ui, |ui| {
                ui.set_min_size(self.size);
                ui.vertical_centered(|ui| {
                    ui.colored_label(self.label_color, &self.name);
                    ui.horizontal(|ui| {
                        ui.label("M");
                        ui.add(ToggleDot::new(self.mute).size(10.0));
                        if let Some(solo) = self.solo {
                            ui.label("S");
                            ui.add(ToggleDot::new(solo).size(10.0));
                        }
                        if let Some(record) = self.record {
                            ui.label("R");
                            ui.add(ToggleDot::new(record).size(10.0));
                        }
                    });
                    ui.add(Knob::new(self.pan, -1.0..=1.0).size(36.0).label("PAN"));
                    for send in self.sends {
                        ui.horizontal(|ui| {
                            ui.label(send.label);
                            ui.add(Knob::new(send.value, 0.0..=1.0).size(24.0));
                        });
                    }
                    ui.horizontal(|ui| {
                        ui.add(
                            Fader::new(self.gain, 0.0..=1.0)
                                .size(Vec2::new(30.0, 128.0))
                                .meter_value(self.level),
                        );
                        ui.add(Meter::new(self.level).size(Vec2::new(12.0, 110.0)));
                    });
                });
            })
            .response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_strip_tracks_extended_controls() {
        let (mut gain, mut pan, mut send) = (0.5, 0.0, 0.2);
        let (mut mute, mut solo, mut record) = (
            crate::widgets::controls::DotState::On,
            crate::widgets::controls::DotState::Solo,
            crate::widgets::controls::DotState::Record,
        );
        let strip = ChannelStrip::new(&mut gain, &mut pan, &mut mute, 0.7)
            .solo(&mut solo)
            .record(&mut record)
            .send("A", &mut send)
            .name("Kick")
            .style(ChannelStripStyle::FlStudio);
        assert_eq!(strip.name, "Kick");
        assert_eq!(strip.sends.len(), 1);
        assert_eq!(strip.style, ChannelStripStyle::FlStudio);
        assert!(strip.solo.is_some());
        assert!(strip.record.is_some());
    }
}
