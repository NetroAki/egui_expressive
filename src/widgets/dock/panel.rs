use super::DockZone;
use egui::{Pos2, Vec2};
use serde::{Deserialize, Serialize};

const MIN_FLOATING_WIDTH: f32 = 80.0;
const MIN_FLOATING_HEIGHT: f32 = 48.0;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Stable panel id used for persisted layout recovery.
pub struct DockPanelId(String);

impl DockPanelId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for DockPanelId {
    fn from(id: &str) -> Self {
        Self::new(id)
    }
}

impl From<String> for DockPanelId {
    fn from(id: String) -> Self {
        Self::new(id)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// Persisted dock placement.
///
/// Floating panels smaller than 80x48 logical pixels are treated as invalid recovery
/// state and redocked.
pub enum DockPlacement {
    Docked(DockZone),
    Floating { pos: Pos2, size: Vec2 },
}

impl DockPlacement {
    pub fn docked(zone: DockZone) -> Self {
        Self::Docked(zone)
    }

    pub fn floating(pos: Pos2, size: Vec2) -> Self {
        Self::Floating { pos, size }
    }

    /// Recovers persisted placement, redocking invalid floating state.
    pub fn recovered(self, fallback_zone: DockZone) -> Self {
        match self {
            Self::Docked(zone) => Self::Docked(zone),
            Self::Floating { pos, size } if is_valid_floating_geometry(pos, size) => {
                Self::Floating { pos, size }
            }
            Self::Floating { .. } => Self::Docked(fallback_zone),
        }
    }
}

fn is_valid_floating_geometry(pos: Pos2, size: Vec2) -> bool {
    // Tiny persisted floating panels are treated as corrupted layout state: they are
    // too small to expose useful drag/title affordances, so recovery re-docks them.
    pos.x.is_finite()
        && pos.y.is_finite()
        && size.x.is_finite()
        && size.y.is_finite()
        && size.x >= MIN_FLOATING_WIDTH
        && size.y >= MIN_FLOATING_HEIGHT
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DockPanel {
    id: DockPanelId,
    title: String,
    placement: DockPlacement,
    closable: bool,
}

impl DockPanel {
    pub fn new(
        id: impl Into<DockPanelId>,
        title: impl Into<String>,
        placement: DockPlacement,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            placement,
            closable: true,
        }
    }

    pub fn id(&self) -> &DockPanelId {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn placement(&self) -> &DockPlacement {
        &self.placement
    }

    pub fn closable(&self) -> bool {
        self.closable
    }

    pub fn recover_placement(&mut self, fallback_zone: DockZone) {
        self.placement = self.placement.clone().recovered(fallback_zone);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dock_panel_keeps_stable_id() {
        let panel = DockPanel::new(
            "inspector",
            "Inspector",
            DockPlacement::docked(DockZone::Right),
        );
        assert_eq!(panel.id(), &DockPanelId::new("inspector"));
        assert_eq!(panel.id().as_str(), "inspector");
        assert_eq!(panel.title(), "Inspector");
        assert_eq!(panel.placement(), &DockPlacement::docked(DockZone::Right));
        assert!(panel.closable());
    }

    #[test]
    fn floating_placement_recovers_invalid_geometry() {
        let placement = DockPlacement::floating(Pos2::new(f32::NAN, 0.0), Vec2::new(-1.0, 20.0));
        assert_eq!(
            placement.recovered(DockZone::Center),
            DockPlacement::Docked(DockZone::Center)
        );
    }

    #[test]
    fn floating_placement_requires_minimum_size() {
        assert!(is_valid_floating_geometry(
            Pos2::new(0.0, 0.0),
            Vec2::new(80.0, 48.0)
        ));
        assert!(!is_valid_floating_geometry(
            Pos2::new(0.0, 0.0),
            Vec2::new(79.9, 48.0)
        ));
        assert!(!is_valid_floating_geometry(
            Pos2::new(0.0, 0.0),
            Vec2::new(80.0, 47.9)
        ));
    }
}
