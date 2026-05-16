/// The 5 visual states of a mute/solo dot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DotState {
    #[default]
    On,
    Muted,
    Solo,
    Record,
    SoloMuted,
    Off,
}

impl DotState {
    pub fn color(self) -> egui::Color32 {
        match self {
            DotState::On => egui::Color32::from_rgb(80, 180, 120),
            DotState::Muted => egui::Color32::from_rgb(220, 140, 60),
            DotState::Solo => egui::Color32::from_rgb(220, 200, 60),
            DotState::Record => egui::Color32::from_rgb(220, 70, 70),
            DotState::SoloMuted => egui::Color32::from_rgb(80, 80, 90),
            DotState::Off => egui::Color32::from_rgb(45, 45, 52),
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            DotState::On => DotState::Off,
            _ => DotState::On,
        }
    }
}
