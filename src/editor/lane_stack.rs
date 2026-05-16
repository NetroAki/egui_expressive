//! Lane-stack composition data for timelines, piano rolls, and automation editors.

#[derive(Clone, Debug, PartialEq)]
pub struct LaneDef {
    pub id: String,
    pub label: String,
    pub height: f32,
    pub collapsible: bool,
}

impl LaneDef {
    pub fn new(id: impl Into<String>, label: impl Into<String>, height: f32) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            height,
            collapsible: true,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LaneStack {
    pub lanes: Vec<LaneDef>,
    pub gap: f32,
}

impl LaneStack {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }
    pub fn lane(mut self, lane: LaneDef) -> Self {
        self.lanes.push(lane);
        self
    }
    pub fn total_height(&self) -> f32 {
        self.lanes.iter().map(|lane| lane.height).sum::<f32>()
            + self.gap * self.lanes.len().saturating_sub(1) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lane_stack_total_height_includes_gaps_between_lanes() {
        let stack = LaneStack::new()
            .gap(2.0)
            .lane(LaneDef::new("notes", "Notes", 12.0))
            .lane(LaneDef::new("velocity", "Velocity", 8.0))
            .lane(LaneDef::new("automation", "Automation", 10.0));

        assert_eq!(stack.total_height(), 34.0);
    }

    #[test]
    fn lane_stack_total_height_empty_is_zero() {
        assert_eq!(LaneStack::new().gap(8.0).total_height(), 0.0);
    }
}
