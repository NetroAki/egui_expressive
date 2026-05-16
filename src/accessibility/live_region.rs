//! Live-region semantics for feedback and custom-painted status surfaces.

use crate::accessibility::{AccessibilityMeta, AccessibilityRole};

/// How urgently assistive technology should announce a changing region.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LiveRegionPoliteness {
    Off,
    Polite,
    Assertive,
}

impl LiveRegionPoliteness {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Polite => "polite",
            Self::Assertive => "assertive",
        }
    }
}

/// Which live-region changes are meaningful enough to announce.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LiveRegionRelevant {
    Additions,
    Removals,
    Text,
    All,
}

impl LiveRegionRelevant {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Additions => "additions",
            Self::Removals => "removals",
            Self::Text => "text",
            Self::All => "all",
        }
    }
}

/// Pure live-region descriptor carried beside egui feedback UI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveRegion {
    pub politeness: LiveRegionPoliteness,
    pub atomic: bool,
    pub relevant: LiveRegionRelevant,
    pub label: String,
}

impl LiveRegion {
    pub fn new(politeness: LiveRegionPoliteness, label: impl Into<String>) -> Self {
        Self {
            politeness,
            atomic: true,
            relevant: LiveRegionRelevant::Text,
            label: label.into(),
        }
    }

    pub fn polite(label: impl Into<String>) -> Self {
        Self::new(LiveRegionPoliteness::Polite, label)
    }

    pub fn assertive(label: impl Into<String>) -> Self {
        Self::new(LiveRegionPoliteness::Assertive, label)
    }

    pub fn atomic(mut self, atomic: bool) -> Self {
        self.atomic = atomic;
        self
    }

    pub fn relevant(mut self, relevant: LiveRegionRelevant) -> Self {
        self.relevant = relevant;
        self
    }

    pub fn metadata(&self, role: AccessibilityRole) -> AccessibilityMeta {
        AccessibilityMeta::new(role, self.label.clone()).live_region(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_region_records_politeness_and_metadata() {
        let region = LiveRegion::assertive("Export failed").relevant(LiveRegionRelevant::All);
        let meta = region.metadata(AccessibilityRole::Alert);

        assert_eq!(region.politeness.as_str(), "assertive");
        assert_eq!(region.relevant.as_str(), "all");
        assert_eq!(meta.role.as_str(), "alert");
        assert_eq!(meta.live_region.as_ref(), Some(&region));
    }
}
